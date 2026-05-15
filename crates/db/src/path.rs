use std::path::Path;

use crate::error::Error;

/// Validate that `path` is an absolute, UTF-8 path and return its basename.
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
