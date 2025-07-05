use std::path::PathBuf;

use dxlog::{init_repository, list_latest_active_logs};

use crate::commands::{
    GraphCommands, HypothesisCommands, KnowledgeCommands, LiteratureCommands, ReferenceCommands,
};

#[derive(clap::Parser)]
#[command(author, version, about = "A research log management tool for tracking hypotheses, literature, and knowledge", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Initialize a new research log repository
    ///
    /// Creates a new dxlog repository with default configuration and directory structure.
    /// This command will set up all necessary folders and template files.
    ///
    /// Examples:
    ///   dxlog init ./my-research
    ///   dxlog init ~/projects/quantum-research
    Init {
        /// Path where the repository should be initialized
        #[arg(help = "Directory path for the new repository")]
        path: PathBuf,
    },

    /// Manage research hypotheses
    Hypothesis {
        #[command(subcommand)]
        command: HypothesisCommands,
    },

    /// Manage literature reviews
    Literature {
        #[command(subcommand)]
        command: LiteratureCommands,
    },

    /// Manage knowledge base entries
    Knowledge {
        #[command(subcommand)]
        command: KnowledgeCommands,
    },

    /// Manage references between entries
    Reference {
        #[command(subcommand)]
        command: ReferenceCommands,
    },

    /// Visualize and analyze the research graph
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },

    /// Show all latest active logs
    ///
    /// Display a summary of all active research logs from the research-logs directory.
    /// This includes hypotheses, literature reviews, and knowledge entries with
    /// status 'active' or 'suspended'.
    ///
    /// Examples:
    ///   dxlog latest
    ///   dxlog latest --limit 10
    Latest {
        /// Limit the number of entries shown
        #[arg(
            short,
            long,
            default_value = "20",
            help = "Maximum number of entries to display"
        )]
        limit: usize,
    },
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Init { path } => init_repository(path),
            Commands::Hypothesis { command } => command.execute(),
            Commands::Literature { command } => command.execute(),
            Commands::Knowledge { command } => command.execute(),
            Commands::Reference { command } => command.execute(),
            Commands::Graph { command } => command.execute(),
            Commands::Latest { limit } => list_latest_active_logs(*limit),
        }
    }
}
