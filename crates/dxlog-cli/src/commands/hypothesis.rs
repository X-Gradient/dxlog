use anyhow::Result;
use dxlog::{create_hypothesis, list_hypotheses, update_hypothesis_status, HypothesisStatus};

// crates/dxlog-cli/src/commands/hypothesis.rs
#[derive(clap::Subcommand, Clone)]
pub enum HypothesisCommands {
    /// Create a new research hypothesis
    ///
    /// Creates a new hypothesis entry with specified title and optional tags.
    /// The hypothesis will be initialized in 'active' status.
    ///
    /// Examples:
    ///   dxlog hypothesis new "Quantum error correction impact" --tags quantum,error-correction
    ///   dxlog hypothesis new "FPGA optimization patterns" -t hardware,performance
    New {
        /// Title of the hypothesis (wrap in quotes if it contains spaces)
        #[arg(help = "The main title of your hypothesis")]
        title: String,

        /// Tags for categorizing the hypothesis
        #[arg(
            short,
            long,
            value_delimiter = ',',
            help_heading = "ORGANIZATION",
            help = "Comma-separated list of tags (e.g., quantum,physics)"
        )]
        tags: Option<Vec<String>>,
    },

    /// Mark hypothesis as proven
    ///
    /// Updates the status of a hypothesis to 'proven' when evidence confirms it.
    /// This will move the hypothesis to the knowledge base.
    ///
    /// Example:
    ///   dxlog hypothesis proven 1f418cae
    Proven {
        /// ID of the hypothesis (can be partial)
        #[arg(help = "Unique identifier or first few characters of the hypothesis ID")]
        id: String,
    },

    /// Mark hypothesis as disproven
    ///
    /// Updates the status of a hypothesis to 'disproven' when evidence refutes it.
    /// This will move the hypothesis to the knowledge base.
    ///
    /// Example:
    ///   dxlog hypothesis disproven 2a7b9def
    Disproven {
        /// ID of the hypothesis (can be partial)
        #[arg(help = "Unique identifier or first few characters of the hypothesis ID")]
        id: String,
    },

    /// Mark hypothesis as inconclusive
    ///
    /// Updates the status to 'inconclusive' when there's insufficient evidence.
    /// This will move the hypothesis to the knowledge base.
    ///
    /// Example:
    ///   dxlog hypothesis inconclusive 3c8d0fed
    Inconclusive {
        /// ID of the hypothesis (can be partial)
        #[arg(help = "Unique identifier or first few characters of the hypothesis ID")]
        id: String,
    },

    /// Temporarily suspend research on a hypothesis
    ///
    /// Marks a hypothesis as 'suspended' when work needs to be paused.
    /// The hypothesis remains in the active directory.
    ///
    /// Example:
    ///   dxlog hypothesis suspend 4d9e1ghi
    Suspend {
        /// ID of the hypothesis (can be partial)
        #[arg(help = "Unique identifier or first few characters of the hypothesis ID")]
        id: String,
    },

    /// List hypotheses with optional filters
    ///
    /// Display all hypotheses, optionally filtered by status and/or tags.
    ///
    /// Examples:
    ///   dxlog hypothesis list
    ///   dxlog hypothesis list --status active
    ///   dxlog hypothesis list --tags quantum,physics
    ///   dxlog hypothesis list -s proven -t quantum
    List {
        /// Filter by hypothesis status
        #[arg(
            short,
            long,
            help_heading = "FILTERS",
            help = "Show only hypotheses with specified status"
        )]
        status: Option<HypothesisStatus>,

        /// Filter by tags
        #[arg(
            short,
            long,
            value_delimiter = ',',
            help_heading = "FILTERS",
            help = "Show only hypotheses with specified tags"
        )]
        tags: Option<Vec<String>>,
    },
}

impl HypothesisCommands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::New { title, tags } => {
                let new_hypothesis = create_hypothesis(title, tags.clone())?;
                println!(
                    "New Hypothesis \"{}\" created with id: {}",
                    new_hypothesis.base.title, new_hypothesis.base.id
                );
                Ok(())
            }
            Self::Proven { id } => {
                update_hypothesis_status(id, HypothesisStatus::Proven)?;
                println!("Update Hypothesis {}; Status => Proven", id);
                Ok(())
            }
            Self::Disproven { id } => {
                update_hypothesis_status(id, HypothesisStatus::Disproven)?;
                println!("Update Hypothesis {}; Status => Disproven", id);
                Ok(())
            }
            Self::Inconclusive { id } => {
                update_hypothesis_status(id, HypothesisStatus::Inconclusive)?;
                println!("Update Hypothesis {}; Status => Inconclusive", id);
                Ok(())
            }
            Self::Suspend { id } => {
                update_hypothesis_status(id, HypothesisStatus::Suspended)?;
                println!("Update Hypothesis {}; Status => Suspended", id);
                Ok(())
            }
            Self::List { status, tags } => {
                let hypotheses = list_hypotheses(status.clone(), tags.clone())?;
                println!(
                    "{:<18} {:<20} {:<12} {:<18} {:<18} TAGS",
                    "HYPOTHESIS ID", "TITLE", "STATUS", "CREATED", "AUTHOR"
                );

                for hypothesis in hypotheses {
                    let id = hypothesis.base.id.to_string();
                    let short_id = &id[..12];
                    let title = if hypothesis.base.title.len() > 20 {
                        format!("{}...", &hypothesis.base.title[..17])
                    } else {
                        hypothesis.base.title.clone()
                    };
                    let author = if hypothesis.base.created_by.name.len() > 12 {
                        format!("{}...", &hypothesis.base.created_by.name[..9])
                    } else {
                        hypothesis.base.created_by.name.clone()
                    };

                    let tags = hypothesis
                        .base
                        .tags
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(", ");

                    println!(
                        "{:<18} {:<20} {:<12} {:<18} {:<18} {}",
                        short_id,
                        title,
                        hypothesis.status.to_string(),
                        hypothesis.base.date,
                        author,
                        tags
                    );
                }

                Ok(())
            }
        }
    }
}
