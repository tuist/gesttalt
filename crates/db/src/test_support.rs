//! Test-only helpers.
//!
//! Every test gets its own private in-memory SQLite database, which is the
//! SQLite-friendly equivalent of Ecto's sandbox: each test runs in isolation
//! and they can execute in parallel without contending on shared state.

use crate::database::Database;

pub(crate) async fn test_db() -> Database {
    Database::in_memory()
        .await
        .expect("open in-memory test database")
}
