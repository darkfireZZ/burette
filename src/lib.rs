//! Rust library for `burette`, a document library manager.

#![warn(
    clippy::pedantic,
    clippy::absolute_paths,
    clippy::allow_attributes_without_reason,
    clippy::dbg_macro,
    clippy::exit,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used,
    missing_debug_implementations,
    missing_docs
)]
// The following lints are enable by default in clippy::pedantic, but are disabled here because
// they are too aggressive.
#![allow(clippy::module_name_repetitions, reason = "Occasionally useful")]
#![allow(clippy::too_many_lines, reason = "This is not bad in my opinion")]

use {
    anyhow::{anyhow, Context},
    std::{env, path::PathBuf},
};

pub mod cli;
pub mod sha256;

mod library;
pub use library::*;

mod file_format;
pub use file_format::FileFormat;

mod isbn;
pub use isbn::Isbn13;

/// Format a string into a format suitable for use as a file name.
#[must_use]
pub fn format_as_file_name(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut prev_was_whitespace = true;
    for c in s.chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' => {
                result.push(c.to_ascii_lowercase());
                prev_was_whitespace = false;
            }
            c if c == '-' || c == '_' || c.is_whitespace() => {
                if !prev_was_whitespace {
                    result.push('_');
                }
                prev_was_whitespace = true;
            }
            _ => {}
        }
    }
    result
}

/// Return the home directory of the current user.
fn home_dir() -> anyhow::Result<PathBuf> {
    let home_dir =
        env::var_os("HOME").ok_or_else(|| anyhow!("Failed to read HOME environment variable"))?;
    Ok(PathBuf::from(home_dir))
}

/// Return the location of the default library directory.
///
/// The default library directory is `$HOME/.book-store`.
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined.
pub fn default_library_dir() -> anyhow::Result<PathBuf> {
    Ok(home_dir()
        .context("Failed to determine library directory")?
        .join(".book-store"))
}
