//! Test-only helpers.
//!
//! Every test gets its own private in-memory SQLite database, which is the
//! SQLite-friendly equivalent of Ecto's sandbox: each test runs in isolation
//! and they can execute in parallel without contending on shared state.

use std::path::PathBuf;

use crate::database::Database;

pub(crate) async fn test_db() -> Database {
    Database::in_memory()
        .await
        .expect("open in-memory test database")
}

/// Build an absolute, UTF-8 filesystem path with the given basename for use in
/// tests. We don't touch the filesystem — we just need a syntactically valid
/// absolute path that satisfies `Path::is_absolute()` on every supported OS
/// (Unix's `/tmp/...` isn't absolute on Windows).
pub(crate) fn project_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(name)
}
