//! Command line interface for the application.

use {
    crate::{DocMetadata, FileFormat, Library},
    anyhow::{bail, Context},
    clap::{Parser, Subcommand},
    std::{
        fmt::{self, Display, Formatter},
        fs,
        io::{self, Write},
        path::PathBuf,
        process::ExitCode,
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
#[must_use]
pub fn run() -> ExitCode {
    let cli = Cli::parse();
    match cli.run() {
        Ok(exit_code) => exit_code,
        Err(error) => {
            eprintln!("Error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn stdin_read_input<T>(prompt: &str) -> anyhow::Result<T>
where
    T: FromStr,
    T::Err: Display,
{
    loop {
        print!("{prompt}: ");
        io::stdout()
            .flush()
            .context("IO error while writing to stdout")?;

        let mut input = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut input)
            .context("IO error while reading from stdin")?;
        if bytes_read == 0 {
            bail!("Unexpected end of input");
        }

        match input.trim().parse() {
            Ok(value) => return Ok(value),
            Err(error) => eprintln!("Invalid input: {error}"),
        }
    }
}

fn stdin_confirm(prompt: &str) -> anyhow::Result<bool> {
    loop {
        print!("{prompt} (y/n): ");
        io::stdout()
            .flush()
            .context("IO error while writing to stdout")?;

        let mut input = String::new();
        let bytes_read = io::stdin()
            .read_line(&mut input)
            .context("IO error while reading from stdin")?;
        if bytes_read == 0 {
            bail!("Unexpected end of input");
        }

        match input.trim().to_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => eprintln!("Please enter 'y'/'yes' or 'n'/'no'."),
        }
    }
}

/// burette is a document management system.
///
/// See the README at <https://github.com/darkfireZZ/burette> for more information.
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
    fn run(&self) -> anyhow::Result<ExitCode> {
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

                let mut doi = None;
                let read_doi = stdin_confirm("Add a DOI?")?;
                if read_doi {
                    doi = Some(stdin_read_input("DOI")?);
                }

                let metadata = DocMetadata {
                    title,
                    authors,
                    isbns,
                    file_format,
                    doi,
                };

                //--------------------------------------------------------------------------------//

                library.add_document(path, metadata)?;

                Ok(ExitCode::SUCCESS)
            }
            Command::Edit { hash_prefix, field } => {
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;

                match field {
                    EditField::Title => {
                        library.edit_metadata(hash_prefix, |index_entry| {
                            println!("Current title:\n{}", index_entry.title());
                            let title = stdin_read_input("New title")?;
                            index_entry.set_title(title);
                            Ok(())
                        })?;
                    }
                    EditField::Authors => {
                        library.edit_metadata(hash_prefix, |index_entry| {
                            println!("Current authors:");
                            for author in index_entry.authors() {
                                println!("{author}");
                            }
                            let mut authors = Vec::new();
                            loop {
                                let another_author = stdin_confirm("Add another author?")?;
                                if !another_author {
                                    break;
                                }
                                let author = stdin_read_input("Author")?;
                                authors.push(author);
                            }
                            index_entry.set_authors(authors);
                            Ok(())
                        })?;
                    }
                    EditField::Isbns => {
                        library.edit_metadata(hash_prefix, |index_entry| {
                            println!("Current ISBNs:");
                            for isbn in index_entry.isbns() {
                                println!("{isbn}");
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
                            index_entry.set_isbns(isbns);
                            Ok(())
                        })?;
                    }
                    EditField::Doi => {
                        library.edit_metadata(hash_prefix, |index_entry| {
                            match index_entry.doi() {
                                Some(doi) => println!("Current DOI:\n{doi}"),
                                None => println!("No DOI currently set."),
                            }
                            let read_doi = stdin_confirm("Add a DOI?")?;
                            let doi = if read_doi {
                                Some(stdin_read_input("DOI")?)
                            } else {
                                None
                            };
                            index_entry.set_doi(doi);
                            Ok(())
                        })?;
                    }
                }

                Ok(ExitCode::SUCCESS)
            }
            Command::Get {
                hash_prefix,
                output,
            } => {
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;
                library.retrieve_document(hash_prefix, output.as_ref())?;
                Ok(ExitCode::SUCCESS)
            }
            Command::List => {
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;
                for doc in library.documents()? {
                    print!("{}: {}", doc.hash().to_short_string(), doc.title());
                    let mut authors = doc.authors();
                    if let Some(author) = authors.next() {
                        print!(" - {author}");
                        for author in authors {
                            print!(", {author}");
                        }
                    }
                    println!();
                }
                Ok(ExitCode::SUCCESS)
            }
            Command::New => {
                let library_path = self.library_path()?;
                Library::new(library_path)?;
                Ok(ExitCode::SUCCESS)
            }
            Command::Remove { hash_prefixes } => {
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;
                let hash_prefixes = hash_prefixes.iter().map(String::as_str);
                let results = library.remove_all(hash_prefixes)?;

                let mut printed = false;

                let mut removed: Vec<_> = results.removed().iter().collect();
                removed.sort_unstable_by_key(|entry| *entry.hash());
                if !removed.is_empty() {
                    println!("Removed documents:");
                    for doc in removed {
                        println!("{}: {}", doc.hash().to_short_string(), doc.title());
                    }
                    printed = true;
                }

                let mut not_found: Vec<_> = results.not_found().iter().collect();
                not_found.sort_unstable();
                if !not_found.is_empty() {
                    if printed {
                        println!();
                    }
                    println!("Documents not found:");
                    for hash_prefix in not_found {
                        println!("{hash_prefix}");
                    }
                    printed = true;
                }

                let mut ambiguous: Vec<_> = results.ambiguous().iter().collect();
                ambiguous.sort_unstable_by_key(|prefix| prefix.hash_prefix());
                if !ambiguous.is_empty() {
                    if printed {
                        println!();
                    }
                    println!("Ambiguous hash prefixes:");
                    for ambiguous_prefix in ambiguous {
                        println!("{}", ambiguous_prefix.hash_prefix());
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

                Ok(if results.success() {
                    ExitCode::SUCCESS
                } else {
                    ExitCode::FAILURE
                })
            }
            Command::Validate => {
                let library_path = self.library_path()?;
                let library = Library::open(library_path)?;
                let results = library.validate()?;
                if results.is_valid() {
                    println!("Library is valid.");
                    Ok(ExitCode::SUCCESS)
                } else {
                    let mut printed = false;

                    let mut missing_files = results.missing_files();
                    if let Some(missing_file) = missing_files.next() {
                        eprintln!("Files present in the index but not in the document store:");
                        eprintln!("{missing_file}");
                        for missing_file in missing_files {
                            eprintln!("{missing_file}");
                        }
                        printed = true;
                    }

                    let mut missing_index_entries = results.missing_index_entries();
                    if let Some(missing_index_entry) = missing_index_entries.next() {
                        if printed {
                            eprintln!();
                        }
                        eprintln!("Files present in the document store but not in the index:");
                        eprintln!("{missing_index_entry}");
                        for missing_index_entry in missing_index_entries {
                            eprintln!("{missing_index_entry}");
                        }
                        printed = true;
                    }

                    let mut hash_mismatches = results.hash_mismatches();
                    if let Some(hash_mismatch) = hash_mismatches.next() {
                        if printed {
                            eprintln!();
                        }
                        eprintln!("Files with names that do not match their hashes:");
                        eprintln!("{hash_mismatch}");
                        for hash_mismatch in hash_mismatches {
                            eprintln!("{hash_mismatch}");
                        }
                        printed = true;
                    }

                    let mut invalid_file_types = results.invalid_file_types();
                    if let Some(invalid_file_type) = invalid_file_types.next() {
                        if printed {
                            eprintln!();
                        }
                        eprintln!("Files with invalid file types:");
                        eprintln!("{invalid_file_type}");
                        for invalid_file_type in invalid_file_types {
                            eprintln!("{invalid_file_type}");
                        }
                    }
                    Ok(ExitCode::FAILURE)
                }
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
    /// Edit the metadata of a document in the library
    Edit {
        /// Hash prefix of the document to edit
        hash_prefix: String,
        /// Field of the document to edit
        field: EditField,
    },
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
    /// Validate the library
    ///
    /// This command checks the integrity of the library and prints any errors found.
    /// If the library is in a valid state, the command prints "Library is valid." and exits with a
    /// status code of 0. If the library is not valid, the command prints the errors found and
    /// exits with a non-zero status code.
    Validate,
}

/// Field of a document to edit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditField {
    /// Edit the title of the document
    Title,
    /// Edit the authors of the document
    Authors,
    /// Edit the ISBNs of the document
    Isbns,
    /// Edit the DOI of the document
    Doi,
}

impl Display for EditField {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            EditField::Title => write!(f, "title"),
            EditField::Authors => write!(f, "authors"),
            EditField::Isbns => write!(f, "isbns"),
            EditField::Doi => write!(f, "doi"),
        }
    }
}

impl FromStr for EditField {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> anyhow::Result<Self> {
        match s {
            "title" => Ok(EditField::Title),
            "authors" => Ok(EditField::Authors),
            "isbns" => Ok(EditField::Isbns),
            "doi" => Ok(EditField::Doi),
            _ => bail!("Invalid field: {}", s),
        }
    }
}
