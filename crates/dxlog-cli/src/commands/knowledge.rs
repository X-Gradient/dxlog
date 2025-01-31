// crates/dxlog-cli/src/commands/knowledge.rs
use anyhow::Result;
use dxlog::{create_knowledge, list_knowledge, update_knowledge_status, KnowledgeStatus};

#[derive(clap::Subcommand, Clone)]
pub enum KnowledgeCommands {
    /// Create a new knowledge base entry
    ///
    /// Creates a new entry in the knowledge base for documenting
    /// established findings, techniques, or concepts.
    ///
    /// Examples:
    ///   dxlog knowledge new "Quantum Error Correction Guide" --tags quantum,guide
    ///   dxlog knowledge new "ML Model Evaluation Methods" -t ml,evaluation
    New {
        /// Title of the knowledge entry
        #[arg(help = "The main title of your knowledge entry")]
        title: String,

        /// Tags for categorization
        #[arg(
            short,
            long,
            value_delimiter = ',',
            help_heading = "ORGANIZATION",
            help = "Comma-separated list of tags"
        )]
        tags: Option<Vec<String>>,
    },

    /// Publish a knowledge entry
    ///
    /// Marks a knowledge entry as reviewed and ready for use.
    /// Moves it to the published section of the knowledge base.
    ///
    /// Example:
    ///   dxlog knowledge publish 8i3j5jkl
    Publish {
        /// ID of the knowledge entry (can be partial)
        #[arg(help = "Unique identifier or first few characters of the entry ID")]
        id: String,
    },

    /// Archive a knowledge entry
    ///
    /// Moves a knowledge entry to the archive when it's no longer current.
    ///
    /// Example:
    ///   dxlog knowledge archive 9k4l6mno
    Archive {
        /// ID of the knowledge entry (can be partial)
        #[arg(help = "Unique identifier or first few characters of the entry ID")]
        id: String,
    },

    /// List knowledge entries with optional filters
    ///
    /// Display all knowledge entries, optionally filtered by status and/or tags.
    ///
    /// Examples:
    ///   dxlog knowledge list
    ///   dxlog knowledge list --status published
    ///   dxlog knowledge list --tags guide
    ///   dxlog knowledge list -s draft -t quantum
    List {
        /// Filter by entry status
        #[arg(
            short,
            long,
            help_heading = "FILTERS",
            help = "Show only entries with specified status"
        )]
        status: Option<KnowledgeStatus>,

        /// Filter by tags
        #[arg(
            short,
            long,
            value_delimiter = ',',
            help_heading = "FILTERS",
            help = "Show only entries with specified tags"
        )]
        tags: Option<Vec<String>>,
    },
}

impl KnowledgeCommands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::New { title, tags } => {
                let knowledge = create_knowledge(title, tags.clone())?;
                println!(
                    "New Knowledge \"{}\" created with id: {}",
                    knowledge.base.title, knowledge.base.id
                );
                Ok(())
            }
            Self::Publish { id } => {
                update_knowledge_status(id, KnowledgeStatus::Published)?;
                println!("Update Knowledge {}; Status => Published", id);
                Ok(())
            }
            Self::Archive { id } => {
                update_knowledge_status(id, KnowledgeStatus::Archived)?;
                println!("Update Knowledge {}; Status => Archived", id);
                Ok(())
            }
            Self::List { status, tags } => {
                println!(
                    "{:<18} {:<20} {:<12} {:<18} {:<18} TAGS",
                    "KNOWLEDGE ID", "TITLE", "STATUS", "CREATED", "AUTHOR"
                );

                let entries = list_knowledge(status.clone(), tags.clone())?;
                for entry in entries {
                    let id = entry.base.id.to_string();
                    let short_id = &id[..12];
                    let title = if entry.base.title.len() > 20 {
                        format!("{}...", &entry.base.title[..17])
                    } else {
                        entry.base.title.clone()
                    };
                    let author = if entry.base.created_by.name.len() > 12 {
                        format!("{}...", &entry.base.created_by.name[..9])
                    } else {
                        entry.base.created_by.name.clone()
                    };

                    let tags = entry.base.tags.into_iter().collect::<Vec<_>>().join(", ");

                    println!(
                        "{:<18} {:<20} {:<12} {:<18} {:<18} {}",
                        short_id,
                        title,
                        entry.status.to_string(),
                        entry.base.date,
                        author,
                        tags
                    );
                }
                Ok(())
            }
        }
    }
}
