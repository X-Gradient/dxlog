use std::io::Write;

use anyhow::Result;
use dxlog::{add_reference, force_add_reference, list_references, remove_reference};

#[derive(clap::Subcommand, Clone)]
pub enum ReferenceCommands {
    /// Add a reference between two entries
    ///
    /// Creates a directional reference from one entry to another.
    /// References should typically point to completed/published entries.
    ///
    /// Example:
    ///   dxlog reference add 1a2b3c4d 5e6f7g8h
    Add {
        /// ID of the source entry (can be partial)
        #[arg(help = "ID of the entry that will contain the reference")]
        source_id: String,

        /// ID of the target entry (can be partial)
        #[arg(help = "ID of the entry being referenced")]
        target_id: String,
    },

    /// Remove a reference between entries
    ///
    /// Removes a directional reference from one entry to another.
    ///
    /// Example:
    ///   dxlog reference remove 1a2b3c4d 5e6f7g8h
    Remove {
        /// ID of the source entry (can be partial)
        #[arg(help = "ID of the entry containing the reference")]
        source_id: String,

        /// ID of the target entry (can be partial)
        #[arg(help = "ID of the referenced entry to remove")]
        target_id: String,
    },

    /// List all references for an entry
    ///
    /// Shows all entries referenced by the specified entry.
    ///
    /// Example:
    ///   dxlog reference list 1a2b3c4d
    List {
        /// ID of the entry (can be partial)
        #[arg(help = "Show references for this entry ID")]
        id: String,
    },
}

impl ReferenceCommands {
    pub fn execute(&self) -> Result<()> {
        match self {
            Self::Add {
                source_id,
                target_id,
            } => match add_reference(source_id, target_id) {
                Ok(_) => {
                    println!("Added reference from {} to {}", source_id, target_id);
                    Ok(())
                }
                Err(e) if e.to_string().starts_with("Warning:") => {
                    eprintln!("{}", e);
                    if confirm_action("Do you want to add the reference anyway? [y/N]: ")? {
                        force_add_reference(source_id, target_id)?;
                        println!("Added reference from {} to {}", source_id, target_id);
                        Ok(())
                    } else {
                        Err(anyhow::anyhow!("Reference addition cancelled"))
                    }
                }
                Err(e) => Err(e),
            },
            Self::Remove {
                source_id,
                target_id,
            } => {
                remove_reference(source_id, target_id)?;
                println!("Removed reference from {} to {}", source_id, target_id);
                Ok(())
            }
            Self::List { id } => {
                println!("{:<12} {:<12} {:<20} {:<30}", "ID", "TYPE", "TITLE", "TAGS");
                let references = list_references(id)?;
                for reference in references {
                    let short_id = &reference.id[..8];
                    let tags_str = reference
                        .tags
                        .iter()
                        .cloned()
                        .collect::<Vec<String>>()
                        .join(", ");

                    println!(
                        "{:<12} {:<12} {:<20} {:<30}",
                        short_id, reference.type_, reference.title, tags_str
                    );
                }
                Ok(())
            }
        }
    }
}

fn confirm_action(prompt: &str) -> Result<bool> {
    print!("{}", prompt);
    std::io::stdout().flush()?;

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_lowercase() == "y")
}
