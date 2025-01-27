//! Command line interface for the application.

use {
    crate::{DocMetadata, FileFormat, Library},
    anyhow::{bail, Context},
    clap::{Parser, Subcommand},
    std::{
        fmt::Display,
        fs,
        io::{self, Write},
        path::PathBuf,
        process,
        str::FromStr,
    },
};

/// Run the command line application.
///
/// This function is the entry point for the command line application. It parses the command line
/// arguments and runs the appropriate command.
///
/// # Errors
///
/// If an error occurs, an error message is printed to standard error and the process exits with a
/// non-zero exit code.
pub fn run() {
    let cli = Cli::parse();
    if let Err(error) = cli.run() {
        eprintln!("Error: {:#}", error);
        process::exit(1);
    }
}

fn stdin_read_input<T>(prompt: &str) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    loop {
        print!("{}: ", prompt);
        io::stdout()
            .flush()
            .context("IO error while writing to stdout")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("IO error while reading from stdin")?;

        match input.trim().parse() {
            Ok(value) => return Ok(value),
            Err(error) => eprintln!("Invalid input: {}", error),
        }
    }
}

fn stdin_confirm(prompt: &str) -> anyhow::Result<bool> {
    loop {
        print!("{} (y/n): ", prompt);
        io::stdout()
            .flush()
            .context("IO error while writing to stdout")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("IO error while reading from stdin")?;
        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => eprintln!("Please enter 'y'/'yes' or 'n'/'no'."),
        }
    }
}

/// Command line interface for the application.
#[derive(Debug, Parser)]
struct Cli {
    /// Path to the document library
    #[clap(long, short)]
    library: Option<PathBuf>,
    /// Operation to perform on the library
    #[command(subcommand)]
    command: Command,
}

impl Cli {
    /// Get the path to the document library.
    fn library_path(&self) -> anyhow::Result<PathBuf> {
        match &self.library {
            Some(path) => Ok(path.clone()),
            None => crate::default_library_dir(),
        }
    }

    /// Run the command.
    fn run(&self) -> anyhow::Result<()> {
        match &self.command {
            Command::Add { path } => {
                // Validate the path

                let exists = fs::exists(path).context("IO error while checking if file exists")?;
                if !exists {
                    bail!("File does not exist: {}", path.display());
                }
                let file_format = FileFormat::from_path(path)?;

                //--------------------------------------------------------------------------------//

                // For user experience, we load the library before asking the user for metadata
                // about the document. This way, if for example the library is not readable, the
                // user is not asked for metadata that cannot be saved.

                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;

                //--------------------------------------------------------------------------------//

                // Ask the user for metadata about the document

                let title = stdin_read_input("Title")?;

                let mut authors = Vec::new();
                loop {
                    let another_author = stdin_confirm("Add another author?")?;
                    if !another_author {
                        break;
                    }
                    let author = stdin_read_input("Author")?;
                    authors.push(author);
                }

                let mut isbns = Vec::new();
                loop {
                    let another_isbn = stdin_confirm("Add another ISBN?")?;
                    if !another_isbn {
                        break;
                    }
                    let isbn = stdin_read_input("ISBN")?;
                    isbns.push(isbn);
                }

                let metadata = DocMetadata {
                    title,
                    authors,
                    isbns,
                    file_format,
                };

                //--------------------------------------------------------------------------------//

                library.add_document(path, metadata)
            }
            Command::Get {
                hash_prefix,
                output,
            } => {
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;
                library.retrieve_document(hash_prefix, output.as_ref())?;
                Ok(())
            }
            Command::List => {
                // TODO: This should be replaced with a more robust implementation
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;
                for doc in library.documents()? {
                    print!("{}: {}", doc.hash().to_short_string(), doc.title());
                    let mut authors = doc.authors();
                    if let Some(author) = authors.next() {
                        print!(" - {}", author);
                        for author in authors {
                            print!(", {}", author);
                        }
                    }
                    println!();
                }
                Ok(())
            }
            Command::New => {
                let library_path = self.library_path()?;
                Library::new(library_path)?;
                Ok(())
            }
            Command::Remove { hash_prefixes } => {
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;
                let hash_prefixes = hash_prefixes.iter().map(String::as_str);
                let results = library.remove_all(hash_prefixes)?;

                let mut printed = false;

                if !results.removed().is_empty() {
                    println!("Removed documents:");
                    for doc in results.removed() {
                        println!("{}: {}", doc.hash().to_short_string(), doc.title());
                    }
                    printed = true;
                }

                if !results.not_found().is_empty() {
                    if printed {
                        println!();
                    }
                    println!("Documents not found:");
                    for hash_prefix in results.not_found() {
                        println!("{}", hash_prefix);
                    }
                    printed = true;
                }

                if !results.ambiguous().is_empty() {
                    if printed {
                        println!();
                    }
                    println!("Ambiguous hash prefixes:");
                    for ambiguous in results.ambiguous() {
                        println!("{}", ambiguous.hash_prefix());
                    }
                    printed = true;
                }

                if !results.errors().is_empty() {
                    if printed {
                        eprintln!();
                    }
                    eprintln!("Errors:");
                    for error in results.errors() {
                        eprintln!(
                            "{}: {}",
                            error.entry().hash().to_short_string(),
                            error.error()
                        );
                    }
                }

                Ok(())
            }
        }
    }
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Add a new document to the library
    Add {
        /// The path to the document to add
        path: PathBuf,
    },
    /// List all documents in the library
    List,
    /// Retrieve a document from the library
    Get {
        /// Hash prefix of the document to retrieve
        hash_prefix: String,
        /// Path to save the document to
        #[clap(long, short)]
        output: Option<PathBuf>,
    },
    /// Create a new library
    New,
    /// Remove documents from the library
    Remove {
        /// Hash prefixes of the documents to remove
        ///
        /// All documents with a hash that starts with one of the given prefixes will be removed.
        /// If a document matches multiple prefixes, it will not be removed and instead a message
        /// will be printed to standard error.
        // This ensures that the user must provide at least one hash prefix.
        #[arg(required = true, num_args = 1..)]
        hash_prefixes: Vec<String>,
    },
}
