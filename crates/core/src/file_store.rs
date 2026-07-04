//! File store — content-addressed blob storage with reference-tracking GC.
//!
//! Identical content deduplicates to a single immutable blob addressed by its
//! content hash; file records reference blobs, and a blob with zero references
//! becomes GC-eligible. Metadata is decoupled from blob bytes. Blobs are immutable
//! — there is no mutate-in-place API.
//!
//! Production content addressing is SHA-256 over the blob bytes (a seam); here a
//! deterministic std hash stands in so the dedup/GC/immutability algebra is testable
//! without a crypto dependency. The StorageBackend (local FS / S3) is also a seam.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// A content address (SHA-256 hex in production; a std-hash hex stand-in here).
pub type ContentHash = String;

/// Compute the content address of a blob. Deterministic: identical bytes hash to
/// the same address (the content-addressing property). SHA-256 is the production seam.
pub fn content_hash(data: &[u8]) -> ContentHash {
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

/// An immutable stored blob and its live reference count.
#[derive(Debug, Clone)]
struct Blob {
    data: Vec<u8>,
    ref_count: usize,
}

/// The content-addressed file store. Blobs are shared by content; file records map
/// a file id to the blob it references (metadata decoupled from bytes).
#[derive(Debug, Default)]
pub struct FileStore {
    blobs: HashMap<ContentHash, Blob>,
    /// file_id -> content hash (the metadata layer, decoupled from blob bytes).
    files: HashMap<String, ContentHash>,
}

impl FileStore {
    pub fn new() -> Self {
        FileStore::default()
    }

    /// Store a file's content. Identical content deduplicates to one blob (its
    /// reference count rises); distinct content creates a new immutable blob.
    /// Returns the content address. Re-adding the same file id rebinds it (its old
    /// blob loses a reference).
    pub fn add_file(&mut self, file_id: &str, data: &[u8]) -> ContentHash {
        let hash = content_hash(data);
        // Rebinding an existing file id: release its previous blob reference first.
        if let Some(old) = self.files.get(file_id).cloned()
            && old != hash
        {
            self.release(&old);
        }
        self.blobs
            .entry(hash.clone())
            .and_modify(|b| b.ref_count += 1)
            .or_insert_with(|| Blob {
                data: data.to_vec(),
                ref_count: 1,
            });
        // If the file id already pointed at this same hash, don't double-count.
        if self
            .files
            .insert(file_id.to_string(), hash.clone())
            .as_deref()
            == Some(hash.as_str())
        {
            self.release(&hash); // it was already counted; undo the increment above
        }
        hash
    }

    /// Read a blob's immutable bytes by content address.
    pub fn read(&self, hash: &ContentHash) -> Option<&[u8]> {
        self.blobs.get(hash).map(|b| b.data.as_slice())
    }

    /// The content address a file id currently references.
    pub fn hash_of(&self, file_id: &str) -> Option<&ContentHash> {
        self.files.get(file_id)
    }

    /// The live reference count of a blob (0 if absent).
    pub fn ref_count(&self, hash: &ContentHash) -> usize {
        self.blobs.get(hash).map(|b| b.ref_count).unwrap_or(0)
    }

    /// Distinct blob count — the dedup measure.
    pub fn blob_count(&self) -> usize {
        self.blobs.len()
    }

    fn release(&mut self, hash: &ContentHash) {
        if let Some(b) = self.blobs.get_mut(hash) {
            b.ref_count = b.ref_count.saturating_sub(1);
        }
    }

    /// Remove a file record, releasing its blob reference. The blob remains until GC.
    pub fn remove_file(&mut self, file_id: &str) {
        if let Some(hash) = self.files.remove(file_id) {
            self.release(&hash);
        }
    }

    /// Collect blobs with zero references, removing them. Returns the reclaimed
    /// addresses. This is the reference-tracking GC.
    pub fn gc(&mut self) -> Vec<ContentHash> {
        let dead: Vec<ContentHash> = self
            .blobs
            .iter()
            .filter(|(_, b)| b.ref_count == 0)
            .map(|(h, _)| h.clone())
            .collect();
        for h in &dead {
            self.blobs.remove(h);
        }
        dead
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_content_deduplicates_to_one_blob() {
        let mut store = FileStore::new();
        let h1 = store.add_file("f1", b"same bytes");
        let h2 = store.add_file("f2", b"same bytes");
        assert_eq!(h1, h2); // content addressing: same bytes -> same address
        assert_eq!(store.blob_count(), 1); // deduped
        assert_eq!(store.ref_count(&h1), 2); // two files hold it
    }

    #[test]
    fn distinct_content_creates_distinct_blobs() {
        let mut store = FileStore::new();
        let a = store.add_file("f1", b"alpha");
        let b = store.add_file("f2", b"beta");
        assert_ne!(a, b);
        assert_eq!(store.blob_count(), 2);
    }

    #[test]
    fn removing_one_holder_keeps_shared_blob() {
        let mut store = FileStore::new();
        let h = store.add_file("f1", b"shared");
        store.add_file("f2", b"shared");
        store.remove_file("f1");
        assert_eq!(store.ref_count(&h), 1);
        assert!(store.gc().is_empty()); // still referenced by f2
        assert_eq!(store.read(&h), Some(b"shared".as_slice()));
    }

    #[test]
    fn zero_reference_blob_is_gc_eligible() {
        let mut store = FileStore::new();
        let h = store.add_file("f1", b"orphan");
        store.remove_file("f1");
        assert_eq!(store.ref_count(&h), 0);
        let reclaimed = store.gc();
        assert_eq!(reclaimed, vec![h.clone()]);
        assert_eq!(store.blob_count(), 0);
        assert_eq!(store.read(&h), None);
    }

    #[test]
    fn rebinding_a_file_releases_its_old_blob() {
        let mut store = FileStore::new();
        let old = store.add_file("f1", b"v1");
        let new = store.add_file("f1", b"v2"); // same file id, new content
        assert_eq!(store.ref_count(&old), 0); // old blob orphaned
        assert_eq!(store.ref_count(&new), 1);
        assert_eq!(store.hash_of("f1"), Some(&new));
    }

    #[test]
    fn re_adding_identical_file_id_and_content_is_stable() {
        let mut store = FileStore::new();
        let h = store.add_file("f1", b"x");
        store.add_file("f1", b"x"); // idempotent re-add
        assert_eq!(store.ref_count(&h), 1); // not double-counted
    }

    #[test]
    fn metadata_is_decoupled_from_blob_bytes() {
        // Two file ids, one blob: the metadata layer maps ids independently.
        let mut store = FileStore::new();
        let h = store.add_file("report.pdf", b"PDFDATA");
        store.add_file("copy.pdf", b"PDFDATA");
        assert_eq!(store.hash_of("report.pdf"), Some(&h));
        assert_eq!(store.hash_of("copy.pdf"), Some(&h));
        assert_eq!(store.blob_count(), 1);
    }
}
