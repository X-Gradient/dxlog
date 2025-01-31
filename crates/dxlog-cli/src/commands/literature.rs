use anyhow::Result;
use dxlog::{
    create_literature, delete_literature, list_literature, update_literature_status,
    LiteratureStatus,
};

#[derive(clap::Subcommand, Clone)]
pub enum LiteratureCommands {
    /// Create a new literature review entry
    ///
    /// Creates a new literature review from an arXiv paper, GitHub repository,
    /// or DOI. Automatically extracts metadata from the source.
    ///
    /// Examples:
    ///   dxlog literature new --url https://arxiv.org/abs/2401.12345 --tags quantum,ml
    ///   dxlog literature new --url https://github.com/username/repo -t software
    ///   dxlog literature new --url 10.1234/journal.paper -t biology
    New {
        /// URL or DOI of the source material
        #[arg(long, help = "arXiv URL, GitHub repository URL, or DOI")]
        url: String,

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

    /// Delete a literature review entry
    ///
    /// Permanently removes a literature review entry.
    ///
    /// Example:
    ///   dxlog literature delete 5e0f2abc
    Delete {
        /// ID of the literature entry (can be partial)
        #[arg(help = "Unique identifier or first few characters of the entry ID")]
        id: String,
    },

    /// Mark literature review as completed
    ///
    /// Updates status to 'completed' when review is finished.
    /// Moves the entry to the knowledge base.
    ///
    /// Example:
    ///   dxlog literature complete 6f1g3def
    Complete {
        /// ID of the literature entry (can be partial)
        #[arg(help = "Unique identifier or first few characters of the entry ID")]
        id: String,
    },

    /// Archive a literature review
    ///
    /// Moves the literature review to the archive directory.
    ///
    /// Example:
    ///   dxlog literature archive 7h2i4ghi
    Archive {
        /// ID of the literature entry (can be partial)
        #[arg(help = "Unique identifier or first few characters of the entry ID")]
        id: String,
    },

    /// List literature reviews with optional filters
    ///
    /// Display all literature reviews, optionally filtered by status and/or tags.
    ///
    /// Examples:
    ///   dxlog literature list
    ///   dxlog literature list --status completed
    ///   dxlog literature list --tags quantum,physics
    ///   dxlog literature list -s in_progress -t ml
    List {
        /// Filter by review status
        #[arg(
            short,
            long,
            help_heading = "FILTERS",
            help = "Show only reviews with specified status"
        )]
        status: Option<LiteratureStatus>,

        /// Filter by tags
        #[arg(
            short,
            long,
            value_delimiter = ',',
            help_heading = "FILTERS",
            help = "Show only reviews with specified tags"
        )]
        tags: Option<Vec<String>>,
    },
}

impl LiteratureCommands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::New { url, tags } => {
                let new_literature = create_literature(url, tags.clone())?;
                println!(
                    "New Literture  \"{}\" created with id: {}",
                    new_literature.base.title, new_literature.base.id
                );

                Ok(())
            }
            Self::Delete { id } => delete_literature(id),
            Self::Complete { id } => update_literature_status(id, LiteratureStatus::Completed),
            Self::Archive { id } => update_literature_status(id, LiteratureStatus::Archived),
            Self::List { status, tags } => {
                println!(
                    "{:<18} {:<20} {:<12} {:<18} {:<18} TAGS",
                    "LITERATURE ID", "TITLE", "STATUS", "CREATED", "AUTHOR"
                );

                let literature_entries = list_literature(status.clone(), tags.clone())?;

                for literature in literature_entries {
                    let id = literature.base.id.to_string();
                    let short_id = &id[..12];
                    let title = if literature.base.title.len() > 20 {
                        format!("{}...", &literature.base.title[..17])
                    } else {
                        literature.base.title.clone()
                    };
                    let author = if literature.base.created_by.name.len() > 12 {
                        format!("{}...", &literature.base.created_by.name[..9])
                    } else {
                        literature.base.created_by.name.clone()
                    };

                    let tags = literature
                        .base
                        .tags
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!(
                        "{:<18} {:<20} {:<12} {:<18} {:<18} {}",
                        short_id,
                        title,
                        literature.status.to_string(),
                        literature.base.date,
                        author,
                        tags
                    );
                }
                Ok(())
            }
        }
    }
}
