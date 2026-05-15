use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("database error: {0}")]
    SeaOrm(#[from] sea_orm::DbErr),

    #[error("could not determine OS data directory")]
    NoDataDir,

    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("project path must be absolute: {0}")]
    PathNotAbsolute(String),

    #[error("project path is not valid UTF-8: {0:?}")]
    PathNotUtf8(PathBuf),

    #[error("project path has no basename: {0}")]
    PathHasNoBasename(String),

    #[error("not found")]
    NotFound,
}
