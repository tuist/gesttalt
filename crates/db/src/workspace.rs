use sqlx::FromRow;

/// A named grouping of projects.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
}
