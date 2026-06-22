//! AES-256-GCM per-chunk memory encryption with Argon2id KDF and OS keychain.

use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, AeadCore, KeyInit, OsRng},
};
use argon2::{Argon2, PasswordHasher, password_hash::SaltString};

// ── EncryptError ──────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum EncryptError {
    Aead(String),
    Kdf(String),
    Keychain(String),
    InvalidCiphertext,
    Utf8,
}

impl std::fmt::Display for EncryptError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptError::Aead(msg) => write!(f, "AEAD error: {msg}"),
            EncryptError::Kdf(msg) => write!(f, "KDF error: {msg}"),
            EncryptError::Keychain(msg) => write!(f, "keychain error: {msg}"),
            EncryptError::InvalidCiphertext => write!(f, "ciphertext too short (missing nonce)"),
            EncryptError::Utf8 => write!(f, "decrypted bytes are not valid UTF-8"),
        }
    }
}

impl std::error::Error for EncryptError {}

pub type EncryptResult<T> = std::result::Result<T, EncryptError>;

// ── MemoryKey ─────────────────────────────────────────────────────────────────

const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

/// A 256-bit AES-GCM key. Zeroised on drop via the contained array.
pub struct MemoryKey([u8; KEY_LEN]);

impl MemoryKey {
    pub fn from_bytes(bytes: [u8; KEY_LEN]) -> Self {
        MemoryKey(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.0
    }
}

impl Drop for MemoryKey {
    fn drop(&mut self) {
        // Zeroise key material on drop
        self.0.iter_mut().for_each(|b| *b = 0);
    }
}

// ── KDF ───────────────────────────────────────────────────────────────────────

/// Derive a 256-bit key from a passphrase and a 16-byte salt using Argon2id.
///
/// The salt must be stored alongside the database (never inside encrypted rows).
pub fn derive_key(passphrase: &str, salt: &[u8; 16]) -> EncryptResult<MemoryKey> {
    let salt_str = SaltString::encode_b64(salt)
        .map_err(|e| EncryptError::Kdf(e.to_string()))?;

    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(passphrase.as_bytes(), &salt_str)
        .map_err(|e| EncryptError::Kdf(e.to_string()))?;

    // Extract the raw hash bytes (first KEY_LEN bytes of the output hash)
    let hash_output = hash
        .hash
        .ok_or_else(|| EncryptError::Kdf("Argon2 produced no hash output".into()))?;

    let raw = hash_output.as_bytes();
    if raw.len() < KEY_LEN {
        return Err(EncryptError::Kdf(format!(
            "hash output too short: {} bytes (need {KEY_LEN})",
            raw.len()
        )));
    }

    let mut key = [0u8; KEY_LEN];
    key.copy_from_slice(&raw[..KEY_LEN]);
    Ok(MemoryKey(key))
}

// ── Encrypt / Decrypt ─────────────────────────────────────────────────────────

/// Encrypt `plaintext` with AES-256-GCM.
///
/// Output layout: `[12-byte nonce] ++ [ciphertext + 16-byte auth tag]`
pub fn encrypt(plaintext: &str, mem_key: &MemoryKey) -> EncryptResult<Vec<u8>> {
    let key = Key::<Aes256Gcm>::from_slice(mem_key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|e| EncryptError::Aead(e.to_string()))?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt a blob produced by [`encrypt`].
pub fn decrypt(blob: &[u8], mem_key: &MemoryKey) -> EncryptResult<String> {
    if blob.len() < NONCE_LEN {
        return Err(EncryptError::InvalidCiphertext);
    }

    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let key = Key::<Aes256Gcm>::from_slice(mem_key.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| EncryptError::Aead(e.to_string()))?;

    String::from_utf8(plaintext).map_err(|_| EncryptError::Utf8)
}

// ── KeyVault ──────────────────────────────────────────────────────────────────

/// Manages the active key for a workspace via the OS keychain.
///
/// Keys are stored as hex strings to avoid raw-binary issues in some keychains.
pub struct KeyVault {
    workspace_id: String,
}

impl KeyVault {
    pub fn new(workspace_id: impl Into<String>) -> Self {
        KeyVault { workspace_id: workspace_id.into() }
    }

    fn entry(&self) -> EncryptResult<keyring::Entry> {
        keyring::Entry::new("cronus", &self.workspace_id)
            .map_err(|e| EncryptError::Keychain(e.to_string()))
    }

    /// Store the key in the OS keychain.
    pub fn store(&self, key: &MemoryKey) -> EncryptResult<()> {
        let hex = hex_encode(key.as_bytes());
        self.entry()?.set_password(&hex)
            .map_err(|e| EncryptError::Keychain(e.to_string()))
    }

    /// Load the key from the OS keychain.
    pub fn load(&self) -> EncryptResult<MemoryKey> {
        let hex = self.entry()?.get_password()
            .map_err(|e| EncryptError::Keychain(e.to_string()))?;
        let bytes = hex_decode(&hex)
            .map_err(EncryptError::Keychain)?;
        if bytes.len() != KEY_LEN {
            return Err(EncryptError::Keychain(format!(
                "stored key has wrong length: {} (expected {KEY_LEN})",
                bytes.len()
            )));
        }
        let mut arr = [0u8; KEY_LEN];
        arr.copy_from_slice(&bytes);
        Ok(MemoryKey(arr))
    }

    /// Evict the key from the OS keychain (lock).
    pub fn evict(&self) -> EncryptResult<()> {
        self.entry()?.delete_credential()
            .map_err(|e| EncryptError::Keychain(e.to_string()))
    }
}

// ── Hex helpers ───────────────────────────────────────────────────────────────

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2) {
        return Err("odd-length hex string".into());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .map_err(|e| format!("invalid hex at {i}: {e}"))
        })
        .collect()
}

// ── Key rotation ──────────────────────────────────────────────────────────────

/// Rotate encryption key for all memory bodies in the given SQLite connection.
///
/// Decrypts all rows with `old_key`, re-encrypts with `new_key` in a single
/// transaction. Commits only when every row succeeds; leaves the database
/// unchanged on any error.
pub fn rotate_keys(
    conn: &rusqlite::Connection,
    old_key: &MemoryKey,
    new_key: &MemoryKey,
) -> EncryptResult<usize> {
    // Load all encrypted bodies
    let rows: Vec<(i64, Vec<u8>)> = {
        let mut stmt = conn
            .prepare("SELECT rowid, body_enc FROM memories WHERE body_enc IS NOT NULL")
            .map_err(|e| EncryptError::Kdf(e.to_string()))?;
        stmt.query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?)))
            .map_err(|e| EncryptError::Kdf(e.to_string()))?
            .collect::<rusqlite::Result<_>>()
            .map_err(|e| EncryptError::Kdf(e.to_string()))?
    };

    // Re-encrypt under new key
    let reencrypted: Vec<(i64, Vec<u8>)> = rows
        .iter()
        .map(|(rowid, blob)| {
            let plaintext = decrypt(blob, old_key)?;
            let new_blob = encrypt(&plaintext, new_key)?;
            Ok((*rowid, new_blob))
        })
        .collect::<EncryptResult<_>>()?;

    // Write in a single transaction
    conn.execute("BEGIN", [])
        .map_err(|e| EncryptError::Kdf(e.to_string()))?;

    let result = (|| {
        for (rowid, blob) in &reencrypted {
            conn.execute(
                "UPDATE memories SET body_enc = ?1 WHERE rowid = ?2",
                rusqlite::params![blob, rowid],
            )
            .map_err(|e| EncryptError::Kdf(e.to_string()))?;
        }
        Ok(reencrypted.len())
    })();

    match result {
        Ok(n) => {
            conn.execute("COMMIT", [])
                .map_err(|e| EncryptError::Kdf(e.to_string()))?;
            Ok(n)
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", []);
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> MemoryKey {
        let mut arr = [0u8; KEY_LEN];
        // Deterministic test key (not cryptographically generated)
        for (i, b) in arr.iter_mut().enumerate() {
            *b = i as u8;
        }
        MemoryKey(arr)
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = "the quick brown fox jumps over the lazy dog";
        let blob = encrypt(plaintext, &key).unwrap();
        let recovered = decrypt(&blob, &key).unwrap();
        assert_eq!(recovered, plaintext);
    }

    #[test]
    fn wrong_key_returns_error() {
        let key = test_key();
        let blob = encrypt("secret", &key).unwrap();

        let mut wrong_bytes = [0u8; KEY_LEN];
        wrong_bytes[0] = 0xff;
        let wrong_key = MemoryKey(wrong_bytes);
        assert!(decrypt(&blob, &wrong_key).is_err(), "wrong key must not decrypt");
    }

    #[test]
    fn encrypt_different_nonces_each_call() {
        let key = test_key();
        let blob1 = encrypt("same text", &key).unwrap();
        let blob2 = encrypt("same text", &key).unwrap();
        // Nonces differ even for same plaintext
        assert_ne!(blob1[..NONCE_LEN], blob2[..NONCE_LEN], "nonces must be unique");
    }

    #[test]
    fn kdf_is_deterministic() {
        let passphrase = "hunter2";
        let salt = [0xab_u8; 16];
        let k1 = derive_key(passphrase, &salt).unwrap();
        let k2 = derive_key(passphrase, &salt).unwrap();
        assert_eq!(k1.as_bytes(), k2.as_bytes(), "same passphrase+salt must yield same key");
    }

    #[test]
    fn kdf_different_salts_yield_different_keys() {
        let passphrase = "same-pass";
        let salt1 = [0x11_u8; 16];
        let salt2 = [0x22_u8; 16];
        let k1 = derive_key(passphrase, &salt1).unwrap();
        let k2 = derive_key(passphrase, &salt2).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes(), "different salts must yield different keys");
    }

    #[test]
    fn short_blob_returns_invalid_ciphertext_error() {
        let key = test_key();
        let too_short = [0u8; NONCE_LEN - 1];
        assert!(matches!(decrypt(&too_short, &key), Err(EncryptError::InvalidCiphertext)));
    }

    #[test]
    fn hex_encode_decode_roundtrip() {
        let bytes = [0xde, 0xad, 0xbe, 0xef];
        let hex = hex_encode(&bytes);
        let decoded = hex_decode(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }
}
