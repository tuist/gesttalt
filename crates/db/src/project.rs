use std::path::Path;

use sqlx::FromRow;

use crate::error::Error;

/// A project attached to a [`crate::Workspace`]. The `path` is an absolute
/// filesystem path (usually a git repository); `name` is derived from the
/// path's basename at insertion time.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Project {
    pub id: i64,
    pub workspace_id: i64,
    pub path: String,
    pub name: String,
}

impl Project {
    pub fn path(&self) -> &Path {
        Path::new(&self.path)
    }
}

pub(crate) fn derive_name(path: &Path) -> Result<String, Error> {
    let path_str = path
        .to_str()
        .ok_or_else(|| Error::PathNotUtf8(path.to_path_buf()))?;
    if !path.is_absolute() {
        return Err(Error::PathNotAbsolute(path_str.to_string()));
    }
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| Error::PathHasNoBasename(path_str.to_string()))?;
    Ok(name.to_string())
}
