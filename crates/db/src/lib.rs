//! Persistent data layer for gesttalt, built on SeaORM.
//!
//! Stores user-defined workspaces (named groupings) and the projects (absolute
//! filesystem paths, usually git repositories) attached to them.

mod database;
mod entities;
mod error;
mod migration;
mod path;

#[cfg(test)]
mod test_support;

pub use database::Database;
pub use entities::project::Model as Project;
pub use entities::workspace::Model as Workspace;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
