//! calc-store — the Storage capability and persisted schema.
//!
//! One logical schema; physical backends differ per target — native filesystem
//! (CLI/TUI/desktop), OPFS (web/PWA), and the File System Access API bridge for
//! the desktop PWA (ADR-0002, ADR-0003). Writes are atomic with last-write-wins
//! across co-running frontends.

use std::error::Error;

/// A Store operation result. Real backends are async; this is the seam.
pub type StoreResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

/// Key-value Storage capability over raw bytes. Concrete entity shapes
/// (Function, Script, Setting, History) layer on top as the schema firms up.
pub trait Storage {
    fn get(&self, key: &str) -> StoreResult<Option<Vec<u8>>>;
    fn put(&self, key: &str, value: &[u8]) -> StoreResult<()>;
    fn list(&self, prefix: &str) -> StoreResult<Vec<String>>;
    fn remove(&self, key: &str) -> StoreResult<()>;
}
