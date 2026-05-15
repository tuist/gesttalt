use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("SQL error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("project path must be absolute: {0}")]
    PathNotAbsolute(String),

    #[error("project path is not valid UTF-8: {0:?}")]
    PathNotUtf8(PathBuf),

    #[error("project path has no basename: {0}")]
    PathHasNoBasename(String),

    #[error("not found")]
    NotFound,
}
