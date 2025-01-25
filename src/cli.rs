//! Command line interface for the application.

use clap::{Parser, Subcommand};

/// Command line interface for the application.
#[derive(Debug, Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {}
