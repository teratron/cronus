use cronus_store_local::memory::encryption::{
    EncryptError, MemoryKey, decrypt, derive_key, encrypt, rotate_keys,
};
use rusqlite::Connection;

const NONCE_LEN: usize = 12;

fn test_key() -> MemoryKey {
    let mut arr = [0u8; 32];
    for (i, b) in arr.iter_mut().enumerate() {
        *b = i as u8;
    }
    MemoryKey::from_bytes(arr)
}

// ── Round-trip ─────────────────────────────────────────────────────────────────

#[test]
fn aes_gcm_roundtrip_recovers_plaintext() {
    let key = test_key();
    let plain = "the quick brown fox";
    let blob = encrypt(plain, &key).unwrap();
    assert_eq!(decrypt(&blob, &key).unwrap(), plain);
}

#[test]
fn wrong_key_yields_decryption_error() {
    let key = test_key();
    let blob = encrypt("secret data", &key).unwrap();

    let mut wrong = [0u8; 32];
    wrong[0] = 0xff;
    let wrong_key = MemoryKey::from_bytes(wrong);
    assert!(
        decrypt(&blob, &wrong_key).is_err(),
        "wrong key must not decrypt"
    );
}

#[test]
fn truncated_blob_yields_invalid_ciphertext() {
    let key = test_key();
    let short = [0u8; NONCE_LEN - 1];
    assert!(matches!(
        decrypt(&short, &key),
        Err(EncryptError::InvalidCiphertext)
    ));
}

// ── KDF ────────────────────────────────────────────────────────────────────────

#[test]
fn argon2id_kdf_produces_32_byte_key() {
    let key = derive_key("my passphrase", &[0u8; 16]).unwrap();
    assert_eq!(key.as_bytes().len(), 32);
}

#[test]
fn argon2id_kdf_is_deterministic() {
    let salt = [0x55_u8; 16];
    let k1 = derive_key("pass", &salt).unwrap();
    let k2 = derive_key("pass", &salt).unwrap();
    assert_eq!(k1.as_bytes(), k2.as_bytes());
}

#[test]
fn argon2id_kdf_differs_by_salt() {
    let k1 = derive_key("pass", &[0x11_u8; 16]).unwrap();
    let k2 = derive_key("pass", &[0x22_u8; 16]).unwrap();
    assert_ne!(k1.as_bytes(), k2.as_bytes());
}

#[test]
fn argon2id_kdf_differs_by_passphrase() {
    let salt = [0xaa_u8; 16];
    let k1 = derive_key("pass1", &salt).unwrap();
    let k2 = derive_key("pass2", &salt).unwrap();
    assert_ne!(k1.as_bytes(), k2.as_bytes());
}

// ── Key rotation ──────────────────────────────────────────────────────────────

fn setup_db_with_encrypted_rows(key: &MemoryKey) -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE memories (id TEXT PRIMARY KEY, body TEXT NOT NULL, body_enc BLOB)",
    )
    .unwrap();

    for i in 0..3 {
        let plaintext = format!("memory body {i}");
        let blob = encrypt(&plaintext, key).unwrap();
        conn.execute(
            "INSERT INTO memories (id, body, body_enc) VALUES (?1, ?2, ?3)",
            rusqlite::params![format!("id-{i}"), &plaintext, &blob],
        )
        .unwrap();
    }
    conn
}

#[test]
fn key_rotation_re_encrypts_all_rows() {
    let old_key = test_key();
    let mut new_bytes = [0u8; 32];
    new_bytes[0] = 0xff;
    let new_key = MemoryKey::from_bytes(new_bytes);

    let conn = setup_db_with_encrypted_rows(&old_key);
    let count = rotate_keys(&conn, &old_key, &new_key).unwrap();
    assert_eq!(count, 3, "rotation must touch all 3 rows");

    // Verify each row decrypts with new key
    let mut stmt = conn.prepare("SELECT body, body_enc FROM memories").unwrap();
    let rows: Vec<(String, Vec<u8>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    for (plain, blob) in rows {
        let recovered = decrypt(&blob, &new_key).unwrap();
        assert_eq!(recovered, plain);
        // Old key must no longer work
        assert!(
            decrypt(&blob, &old_key).is_err(),
            "old key must not decrypt after rotation"
        );
    }
}

#[test]
fn key_rotation_leaves_db_clean_on_empty_table() {
    let key = test_key();
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE memories (id TEXT PRIMARY KEY, body TEXT NOT NULL, body_enc BLOB)",
    )
    .unwrap();
    let count = rotate_keys(&conn, &key, &key).unwrap();
    assert_eq!(count, 0);
}

// ── Nonce uniqueness ──────────────────────────────────────────────────────────

#[test]
fn each_encrypt_call_uses_unique_nonce() {
    let key = test_key();
    let b1 = encrypt("same", &key).unwrap();
    let b2 = encrypt("same", &key).unwrap();
    assert_ne!(
        &b1[..NONCE_LEN],
        &b2[..NONCE_LEN],
        "nonces must differ across calls"
    );
}
