use std::path::PathBuf;

use dxlog::init_repository;

use crate::commands::{
    HypothesisCommands, KnowledgeCommands, LiteratureCommands, ReferenceCommands,
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
}

impl Cli {
    pub fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Commands::Init { path } => init_repository(path),
            Commands::Hypothesis { command } => command.execute(),
            Commands::Literature { command } => command.execute(),
            Commands::Knowledge { command } => command.execute(),
            Commands::Reference { command } => command.execute(),
        }
    }
}
