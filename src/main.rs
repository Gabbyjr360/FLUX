// Main entry point for the Flux CLI.
// Parses command-line arguments and dispatches to the appropriate CLI functions.

use anyhow::Result;
use clap::Parser;

mod cli;
mod storage;
mod commit;
mod diff;

// If you want shorter calls later, you can add:
// use cli::{init, add, commit, diff, sync, branch, checkout};

#[derive(Parser)]
#[command(name = "flux", about = "Flux V1 - Version control for creative files")]
enum Commands {
    /// Initialize a new Flux repository
    Init,

    /// Add a file to staging
    Add {
        /// The file to add
        file: String,
    },

    /// Commit staged changes
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
    },

    /// Show differences for a file (object-level for .blend)
    Diff {
        /// The file to diff
        file: String,
    },

    /// Sync changes locally (remote optional, local only in V1)
    Sync,

    /// Create a new branch
    Branch {
        /// Name of the new branch
        name: String,
    },

    /// Checkout a branch
    Checkout {
        /// Name of the branch to checkout
        name: String,
    },
}

fn main() -> Result<()> {
    let cmd = Commands::parse();

    match cmd {
        Commands::Init              => cli::init()?,
        Commands::Add { file }      => cli::add(&file)?,
        Commands::Commit { message } => cli::commit(&message)?,
        Commands::Diff { file }      => cli::diff(&file)?,
        Commands::Sync              => cli::sync()?,
        Commands::Branch { name }    => cli::branch(&name)?,
        Commands::Checkout { name }  => cli::checkout(&name)?,
    }

    Ok(())
}