use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use uuid::Uuid;

// Re-export all functionality for backward compatibility
pub use file_ops::*;
pub use git_ops::*; 
pub use graph_ops::*;
pub use editor_ops::*;
pub use filename::*;

// Sub-modules
pub mod file_ops;
pub mod git_ops;
pub mod graph_ops;
pub mod editor_ops;
pub mod filename;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseLog {
    pub id: Uuid,
    pub date: String,
    pub title: String,
    pub tags: HashSet<String>,
    pub created_by: Author,
    pub references: HashSet<Uuid>,
}

pub struct StatusChange {
    pub from: String,
    pub to: String,
    pub reason: String,
}