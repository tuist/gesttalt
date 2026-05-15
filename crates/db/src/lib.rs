//! Persistent data layer for gesttalt.
//!
//! Stores user-defined workspaces (named groupings) and the projects (absolute
//! filesystem paths, usually git repositories) attached to them.

mod database;
mod error;
mod project;
mod workspace;

pub use database::Database;
pub use error::Error;
pub use project::Project;
pub use workspace::Workspace;

pub type Result<T> = std::result::Result<T, Error>;
