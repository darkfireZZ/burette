use {
    crate::{sha256, FileFormat, Isbn13},
    anyhow::{anyhow, bail, Context},
    serde::{Deserialize, Serialize},
    std::{
        collections::HashSet,
        env,
        ffi::{OsStr, OsString},
        fmt::{self, Display, Formatter},
        fs::{self, File, FileType},
        iter,
        path::{Path, PathBuf},
    },
};

/// The location of the document store directory within the library directory.
const DOCUMENT_STORE_DIR: &str = "documents";

/// The location of the index file within the library directory.
const INDEX_FILE: &str = "index.json";

/// The location of the version file within the library directory.
const VERSION_FILE: &str = "burette_version";

/// Handle to a document library.
///
/// The [`Library`] is the main interface to the document store. It provides methods to add,
/// retrieve and remove documents from the library.
///
/// The library is stored on disk in a directory structure. This struct is essentially a reference
/// to that directory that provides methods to interact with the document store at that location.
#[derive(Debug)]
pub struct Library {
    path: PathBuf,
    #[allow(
        dead_code,
        reason = "
        For now, we don't use the version field.
        However, it may be useful in the future if we want to implement backwards compatibility or
        other features that depend on the version of the library."
    )]
    version: String,
}

impl Library {
    /// Return the path to the document store directory of the library.
    fn document_store_dir(&self) -> PathBuf {
        self.path.join(DOCUMENT_STORE_DIR)
    }

    /// Return the path to the index file of the library.
    fn index_path(&self) -> PathBuf {
        self.path.join(INDEX_FILE)
    }

    /// Create a new library at the specified path.
    ///
    /// # Errors
    ///
    /// This function will return an error if the library directory already exists or if there is
    /// an error when initializing the library.
    pub fn new<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let context = "Failed to initialize new library";

        let exists = path
            .try_exists()
            .with_context(|| {
                format!(
                    "Could not determine if library directory exists at {}",
                    path.display()
                )
            })
            .context(context)?;
        if exists {
            return Err(anyhow!("Directory already exists").context(context));
        }

        match Self::try_create(path) {
            Ok(library) => Ok(library),
            Err(error) => {
                // If we failed to create the library, clean up any files that were created.
                let _ = fs::remove_dir_all(path);
                Err(error.context(context))
            }
        }
    }

    /// Helper function to create a new library at the specified path.
    ///
    /// This function should only be called by [`Library::new()`].
    fn try_create(path: &Path) -> anyhow::Result<Self> {
        fs::create_dir_all(path)
            .with_context(|| format!("Failed to create library directory at {}", path.display()))?;

        let index_path = path.join(INDEX_FILE);
        LibraryIndex::new().save(&index_path)?;

        let version = env!("CARGO_PKG_VERSION");
        let version_path = path.join(VERSION_FILE);
        fs::write(&version_path, version).with_context(|| {
            format!("Failed to write version file to {}", version_path.display())
        })?;
        Ok(Self {
            path: path.to_owned(),
            version: version.to_owned(),
        })
    }

    /// Open an existing library at the specified path.
    ///
    /// This function validates that the version of the library is compatible with the software
    /// version and that the index file is correctly formatted. If either of these checks fail, an
    /// error is returned.
    ///
    /// Note that this function does fully validate that the library directory is intact. If the
    /// library directory is corrupted, this function may still succeed and subsequent operations
    /// may fail. If the user wants to validate the integrity of the library directory, they should
    /// use [`Library::validate()`].
    ///
    /// # Errors
    ///
    /// An error will be returned in any of the following cases:
    /// - The library directory does not exist.
    /// - The version file is missing or cannot be read.
    /// - The index file is missing, cannot be read or contains invalid data.
    /// - The version of the library is incompatible with the software version.
    pub fn open<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        // This is only a small wrapper around `open_impl` to provide a better error message.
        Self::open_impl(path.as_ref()).context("Failed to open library")
    }

    /// Internal implementation of [`Library::open()`].
    ///
    /// See [`Library::open()`] for details.
    fn open_impl(path: &Path) -> anyhow::Result<Self> {
        let exists = path.try_exists().with_context(|| {
            format!(
                "Could not determine if library directory exists at {}",
                path.display()
            )
        })?;
        if !exists {
            bail!("Directory {} does not exist", path.display());
        }

        let version_path = path.join(VERSION_FILE);
        let library_version = fs::read_to_string(&version_path).with_context(|| {
            format!(
                "Failed to read version file from {}",
                version_path.display()
            )
        })?;
        let software_version = env!("CARGO_PKG_VERSION");
        if library_version != software_version {
            bail!(
                "Document library version ({}) is incompatible with software version ({})",
                library_version,
                software_version
            );
        }

        let index_path = path.join(INDEX_FILE);
        // We open the index file here to validate that it is correctly formatted.
        // No need to keep the index around, as we only need to validate it once.
        LibraryIndex::open(&index_path)?;

        Ok(Self {
            path: path.to_owned(),
            version: library_version,
        })
    }

    /// Add a document to the library.
    ///
    /// This function inserts the document at the specified path into the library. If the document
    /// is already in the library, an error is returned and the library is not modified.
    ///
    /// # Errors
    ///
    /// An error will be returned in any of the following cases:
    /// - The document is already in the library.
    /// - The document file cannot be read.
    /// - The document file cannot be copied to the document store.
    /// - The index file cannot be read or written.
    pub fn add_document<P: AsRef<Path>>(
        &self,
        path: P,
        metadata: DocMetadata,
    ) -> anyhow::Result<()> {
        let doc_file = File::open(&path)
            .with_context(|| format!("Failed to open file at {}", path.as_ref().display()))?;
        let hash = sha256::hash_reader(doc_file)?;

        let index_path = self.index_path();
        let mut index = LibraryIndex::open(&index_path)?;

        // Check if the document is already in the library.
        for doc in &index.documents {
            if let Some(isbn) = doc
                .metadata
                .isbns
                .iter()
                .find(|isbn| metadata.isbns.contains(isbn))
            {
                bail!(
                    "Document with ISBN {} already exists ({})",
                    isbn,
                    doc.hash.to_short_string()
                );
            }
            if doc.hash == hash {
                bail!(
                    "Document is already in the library ({})",
                    doc.hash.to_short_string()
                );
            }
        }

        // Add the document to the library.
        let document_store_dir = self.document_store_dir();
        fs::create_dir_all(&document_store_dir).with_context(|| {
            format!(
                "Failed to create document store directory at {}",
                document_store_dir.display()
            )
        })?;
        let store_path = document_store_dir.join(hash.to_string());
        fs::copy(&path, &store_path).with_context(|| {
            format!(
                "Failed to copy file from {} to document store at {}",
                path.as_ref().display(),
                store_path.display()
            )
        })?;
        let index_entry = IndexEntry { hash, metadata };
        index.documents.push(index_entry);

        // Save the updated index.
        if let Err(error) = index.save(&index_path) {
            // If we can't save the index, remove the document we just added.
            // This is a best-effort approach to avoid having a document in the library without an
            // index entry.
            // We ignore any errors that occur when removing the document, as we want to propagate the
            // original error.
            let _ = fs::remove_file(&store_path);
            return Err(error);
        }

        Ok(())
    }

    /// Retrieve a document from the library.
    ///
    /// The document matching the given hash prefix is copied to the specified output path.
    /// If the output path is `None`, the document is copied to the current working directory.
    /// If multiple or no documents match the hash prefix, an error is returned.
    ///
    /// # Errors
    ///
    /// An error will be returned in any of the following cases:
    /// - The file at the output path already exists.
    /// - Multiple documents match the hash prefix.
    /// - No documents match the hash prefix.
    /// - The index file cannot be read.
    /// - The document cannot be copied to the output path.
    pub fn retrieve_document<P: AsRef<Path>>(
        &self,
        hash_prefix: &str,
        out_path: Option<P>,
    ) -> anyhow::Result<IndexEntry> {
        let index_path = self.index_path();
        let index = LibraryIndex::open(&index_path)?;

        let mut matches = index.find_all_hashes(iter::once(hash_prefix));

        let found = matches.found.pop();
        let ambiguous = matches.ambiguous.pop();
        let not_found = matches.not_found.pop();

        debug_assert_eq!(matches.found.len(), 0);
        debug_assert_eq!(matches.ambiguous.len(), 0);
        debug_assert_eq!(matches.not_found.len(), 0);

        match (found, ambiguous, not_found) {
            (Some(entry), None, None) => {
                let out_path = match out_path {
                    Some(p) => p.as_ref().to_owned(),
                    None => PathBuf::from(entry.default_file_name()),
                };
                let exists = out_path.try_exists().with_context(|| {
                    format!(
                        "Could not determine if output file exists at {}",
                        out_path.display()
                    )
                })?;
                if exists {
                    bail!("Output file {} already exists", out_path.display());
                }
                let store_path = self.document_store_dir().join(entry.hash().to_string());
                fs::copy(&store_path, &out_path).with_context(|| {
                    format!(
                        "Failed to copy document from {} to {}",
                        store_path.display(),
                        out_path.display()
                    )
                })?;
                Ok(entry.clone())
            }
            (None, Some(ambiguous), None) => {
                bail!(
                    "Multiple documents match hash prefix {}",
                    ambiguous.hash_prefix,
                );
            }
            (None, None, Some(hash_prefix)) => {
                bail!("No documents match hash prefix {}", hash_prefix);
            }
            _ => unreachable!(),
        }
    }

    /// Iterate over the metadata of all documents in the library.
    ///
    /// # Errors
    ///
    /// This function returns an error if the index file cannot be read.
    pub fn documents(&self) -> anyhow::Result<impl Iterator<Item = IndexEntry>> {
        let index_path = self.index_path();
        Ok(LibraryIndex::open(&index_path)?.documents.into_iter())
    }

    /// Remove all documents that match the specified hash prefixes.
    ///
    /// Documents are removed from the library if their hash starts with one of the specified hash
    /// prefixes. If a hash prefix matches multiple documents, none of the matched documents are
    /// removed.
    ///
    /// The returned [`RemovalResults`] object provides information about which documents were
    /// - successfully removed,
    /// - not found in the library,
    /// - ambiguous (i.e. multiple documents matched the hash prefix), and
    /// - could not be removed due to an error.
    ///
    /// # Errors
    ///
    /// This function returns an error if the index file cannot be read or written. If an error
    /// occurs when trying to remove a document, the error is included in the [`RemovalResults`]
    /// object.
    pub fn remove_all<'a, H>(&self, hash_prefixes: H) -> anyhow::Result<RemovalResults<'a>>
    where
        H: Iterator<Item = &'a str>,
    {
        let index_path = self.index_path();
        let index = LibraryIndex::open(&index_path)?;

        let matches = index.find_all_hashes(hash_prefixes);

        let not_found = matches.not_found;
        let ambiguous = matches
            .ambiguous
            .iter()
            .map(AmbiguousHashMatch::to_owned)
            .collect();

        // This could be a HashSet, but we expect the number of documents to be small, so a Vec is
        // fine.
        let mut to_be_removed = Vec::new();
        let mut errors = Vec::new();

        let document_store_dir = self.document_store_dir();
        for entry in matches.found {
            let hash = entry.hash();
            let path = document_store_dir.join(hash.to_string());
            match fs::remove_file(&path) {
                Ok(()) => to_be_removed.push(*hash),
                Err(error) => {
                    let error = anyhow::Error::from(error)
                        .context(format!("Failed to remove document at {}", path.display()));
                    errors.push(RemovalError {
                        entry: entry.clone(),
                        error,
                    });
                }
            }
        }

        let mut documents = Vec::with_capacity(index.documents.len() - to_be_removed.len());
        let mut removed = Vec::with_capacity(to_be_removed.len());

        for entry in index.documents {
            if to_be_removed.contains(entry.hash()) {
                removed.push(entry);
            } else {
                documents.push(entry);
            }
        }

        // If this fails, the library is in an inconsistent state.
        LibraryIndex { documents }.save(&index_path)?;

        Ok(RemovalResults {
            ambiguous,
            errors,
            not_found,
            removed,
        })
    }

    /// Check if the library is in a consistent state.
    ///
    /// This function performs the following checks:
    /// - The document store contains only files and no directories or other types of files.
    /// - The names of all files in the document store match their SHA-256 hash.
    /// - All entries in the index file have a corresponding file in the document store.
    /// - All files in the document store have an entry in the index file.
    ///
    /// # Errors
    ///
    /// If there is an IO error when validating the library, an error is returned.
    //
    // Note that we don't need to check
    // - existence of the document store directory
    // - validity of the index file
    // - validity of the version file
    // as these are checked when opening the library.
    pub fn validate(&self) -> anyhow::Result<ValidationResults> {
        let document_store_dir = self.document_store_dir();

        let mut hash_mismatches = Vec::new();
        let mut invalid_file_types = Vec::new();
        let mut existing_files = HashSet::new();

        let dir = fs::read_dir(&document_store_dir).with_context(|| {
            format!(
                "Failed to read document store directory at {}",
                document_store_dir.display()
            )
        })?;
        for entry in dir {
            let entry = entry.context("Failed to read directory entry of document store")?;

            let file_type = entry.file_type().with_context(|| {
                format!(
                    "Failed to determine file type of {}",
                    entry.path().display()
                )
            })?;
            let file_name = entry.file_name();
            if !file_type.is_file() {
                invalid_file_types.push(NotAFile {
                    file_name,
                    file_type,
                });
                continue;
            }

            let path = entry.path();
            let file = File::open(&path)
                .with_context(|| format!("Failed to open file {}", path.display()))?;
            let hash = sha256::hash_reader(file)
                .with_context(|| format!("Failed to hash file {}", path.display()))?;
            let hash_str = hash.to_string();
            if *file_name != *hash_str {
                hash_mismatches.push(HashMismatch {
                    expected: hash,
                    actual: file_name,
                });
            }
            existing_files.insert(hash);
        }

        let index_path = self.index_path();
        let index = LibraryIndex::open(&index_path)?;

        let existing_entries = index
            .documents
            .iter()
            .map(|entry| *entry.hash())
            .collect::<HashSet<_>>();

        let missing_files = existing_entries
            .difference(&existing_files)
            .copied()
            .collect();
        let missing_index_entries = existing_files
            .difference(&existing_entries)
            .copied()
            .collect();

        Ok(ValidationResults {
            missing_files,
            missing_index_entries,
            hash_mismatches,
            invalid_file_types,
        })
    }
}

/// Results from [`Library::validate()`].
///
/// See [`Library::validate()`] for details.
#[derive(Debug)]
pub struct ValidationResults {
    missing_files: HashSet<sha256::Hash>,
    missing_index_entries: HashSet<sha256::Hash>,
    hash_mismatches: Vec<HashMismatch>,
    invalid_file_types: Vec<NotAFile>,
}

impl ValidationResults {
    /// Return true if the library is in a consistent state.
    ///
    /// If this returns true, then
    /// - [`Self::missing_files()`] is empty,
    /// - [`Self::missing_index_entries()`] is empty,
    /// - [`Self::hash_mismatches()`] is empty, and
    /// - [`Self::invalid_file_types()`] is empty.
    ///
    /// If this returns false, then at least one of the above conditions is not met.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.missing_files.is_empty()
            && self.missing_index_entries.is_empty()
            && self.hash_mismatches.is_empty()
            && self.invalid_file_types.is_empty()
    }

    /// Return the SHA-256 hashes of files that are in the document store but not in the index.
    pub fn missing_files(&self) -> impl Iterator<Item = &sha256::Hash> {
        self.missing_files.iter()
    }

    /// Return the SHA-256 hashes of files that are in the index but not in the document store.
    pub fn missing_index_entries(&self) -> impl Iterator<Item = &sha256::Hash> {
        self.missing_index_entries.iter()
    }

    /// Return information about files with names that do not match their SHA-256 hash.
    pub fn hash_mismatches(&self) -> impl Iterator<Item = &HashMismatch> {
        self.hash_mismatches.iter()
    }

    /// Return information about files with invalid file types.
    pub fn invalid_file_types(&self) -> impl Iterator<Item = &NotAFile> {
        self.invalid_file_types.iter()
    }
}

/// Indicates that the name of a file does not match its SHA-256 hash.
#[derive(Debug)]
pub struct HashMismatch {
    expected: sha256::Hash,
    actual: OsString,
}

impl HashMismatch {
    /// The SHA-256 hash of the file. This is the expected name of the file.
    #[must_use]
    pub fn expected(&self) -> &sha256::Hash {
        &self.expected
    }

    /// The actual name of the file.
    #[must_use]
    pub fn actual(&self) -> &OsStr {
        &self.actual
    }
}

impl Display for HashMismatch {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(
            f,
            "{} has name {}",
            self.expected.to_short_string(),
            self.actual.to_string_lossy(),
        )
    }
}

/// Indicates that a file has an invalid file type.
#[derive(Debug)]
pub struct NotAFile {
    file_name: OsString,
    file_type: FileType,
}

impl NotAFile {
    /// The name of the file.
    #[must_use]
    pub fn file_name(&self) -> &OsStr {
        &self.file_name
    }

    /// The type of the file.
    #[must_use]
    pub fn file_type(&self) -> FileType {
        self.file_type
    }
}

impl Display for NotAFile {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let file_type = if self.file_type.is_dir() {
            "directory"
        } else if self.file_type.is_symlink() {
            "symlink"
        } else if self.file_type.is_file() {
            unreachable!("NotAFile should only be used for non-file types");
        } else {
            "unknown"
        };
        write!(
            f,
            "{} is not a regular file (type: {file_type})",
            self.file_name.to_string_lossy(),
        )
    }
}

/// Results from [`Library::remove_all()`].
///
/// See [`Library::remove_all()`] for details.
#[derive(Debug)]
pub struct RemovalResults<'a> {
    ambiguous: Vec<AmbiguousHashMatchOwned<'a>>,
    errors: Vec<RemovalError>,
    not_found: Vec<&'a str>,
    removed: Vec<IndexEntry>,
}

impl<'a> RemovalResults<'a> {
    /// Return true if all documents were successfully removed.
    #[must_use]
    pub fn success(&self) -> bool {
        self.ambiguous.is_empty() && self.errors.is_empty() && self.not_found.is_empty()
    }

    /// Entries that could not be removed because multiple documents matched the hash prefix.
    #[must_use]
    pub fn ambiguous(&self) -> &[AmbiguousHashMatchOwned<'a>] {
        &self.ambiguous
    }

    /// Entries that could not be removed due to an error.
    #[must_use]
    pub fn errors(&self) -> &[RemovalError] {
        &self.errors
    }

    /// Hashes that could not be found in the library.
    #[must_use]
    pub fn not_found(&self) -> &[&'a str] {
        &self.not_found
    }

    /// Entries that were successfully removed from the library.
    #[must_use]
    pub fn removed(&self) -> &[IndexEntry] {
        &self.removed
    }
}

/// Error that occurred when trying to remove a document from the library.
#[derive(Debug)]
pub struct RemovalError {
    entry: IndexEntry,
    error: anyhow::Error,
}

impl RemovalError {
    /// Get the entry that could not be removed.
    #[must_use]
    pub fn entry(&self) -> &IndexEntry {
        &self.entry
    }

    /// Get the error that occurred when trying to remove the entry.
    #[must_use]
    pub fn error(&self) -> &anyhow::Error {
        &self.error
    }
}

/// The index of the document library.
///
/// The index is a list of all documents in the library along with metadata about each document.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
struct LibraryIndex {
    documents: Vec<IndexEntry>,
}

/// An entry in the index of the document library.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    hash: sha256::Hash,
    #[serde(flatten)]
    metadata: DocMetadata,
}

impl IndexEntry {
    /// Return the default file name for the document.
    #[must_use]
    pub fn default_file_name(&self) -> String {
        let mut file_name = crate::format_as_file_name(self.title());
        file_name.push('.');
        file_name.push_str(self.file_format().extension());
        file_name
    }

    /// Return the hash of the document.
    #[must_use]
    pub fn hash(&self) -> &sha256::Hash {
        &self.hash
    }

    /// Return the title of the document.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.metadata.title
    }

    /// Return the authors of the document.
    pub fn authors(&self) -> impl Iterator<Item = &str> {
        self.metadata.authors.iter().map(String::as_str)
    }

    /// Return the ISBNs of the document.
    pub fn isbns(&self) -> impl Iterator<Item = &Isbn13> {
        self.metadata.isbns.iter()
    }

    /// Return the file format of the document.
    #[must_use]
    pub fn file_format(&self) -> FileFormat {
        self.metadata.file_format
    }
}

impl LibraryIndex {
    /// Create a new, empty index.
    ///
    /// This will not write the index to disk. Use [`LibraryIndex::save()`] to write the index to
    /// disk.
    fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    /// Find all documents in the index that match the specified hash prefixes.
    fn find_all_hashes<'hash, 'entry, H>(
        &'entry self,
        hash_prefixes: H,
    ) -> HashMatches<'hash, 'entry>
    where
        H: Iterator<Item = &'hash str>,
    {
        // Collect the hashes into a HashSet to remove duplicates.
        let hash_prefixes: Vec<_> = hash_prefixes.collect::<HashSet<_>>().into_iter().collect();
        let mut matches = vec![Vec::new(); hash_prefixes.len()];

        for entry in &self.documents {
            let entry_hash = entry.hash().to_string();
            for (i, hash_prefix) in hash_prefixes.iter().enumerate() {
                if entry_hash.starts_with(hash_prefix) {
                    matches[i].push(entry);
                }
            }
        }

        let mut ambiguous = Vec::new();
        let mut not_found = Vec::new();
        let mut found = Vec::new();

        for (hash_prefix, matches) in hash_prefixes.iter().zip(matches) {
            match matches.len() {
                0 => not_found.push(*hash_prefix),
                1 => found.push(matches[0]),
                _ => ambiguous.push(AmbiguousHashMatch {
                    hash_prefix,
                    matches,
                }),
            }
        }

        HashMatches {
            ambiguous,
            found,
            not_found,
        }
    }

    /// Read the index from disk.
    fn open(index_path: &Path) -> anyhow::Result<Self> {
        let file = File::open(index_path).with_context(|| {
            format!(
                "Failed to open library index file at {}",
                index_path.display()
            )
        })?;
        let documents: Vec<IndexEntry> = serde_json::from_reader(file).with_context(|| {
            format!("Failed to read library index from {}", index_path.display())
        })?;
        Ok(Self { documents })
    }

    /// Save the index to disk.
    fn save(&self, index_path: &Path) -> anyhow::Result<()> {
        let writer = File::create(index_path).with_context(|| {
            format!(
                "Failed to write library index file at {}",
                index_path.display()
            )
        })?;
        serde_json::to_writer_pretty(writer, &self).with_context(|| {
            format!(
                "Failed to serialize library index to {}",
                index_path.display()
            )
        })
    }
}

/// Results from [`LibraryIndex::find_all_hashes()`].
#[derive(Debug)]
struct HashMatches<'hash, 'entry> {
    ambiguous: Vec<AmbiguousHashMatch<'hash, 'entry>>,
    found: Vec<&'entry IndexEntry>,
    not_found: Vec<&'hash str>,
}

/// Information about a hash prefix that matched multiple documents in the index.
#[derive(Debug)]
struct AmbiguousHashMatch<'hash, 'entry> {
    hash_prefix: &'hash str,
    matches: Vec<&'entry IndexEntry>,
}

impl<'hash, 'entry> AmbiguousHashMatch<'hash, 'entry> {
    fn to_owned(&self) -> AmbiguousHashMatchOwned<'hash> {
        AmbiguousHashMatchOwned {
            hash_prefix: self.hash_prefix,
            matches: self.matches.iter().copied().cloned().collect(),
        }
    }
}

/// Information about a hash prefix that matched multiple documents in the index.
#[derive(Debug)]
pub struct AmbiguousHashMatchOwned<'hash> {
    hash_prefix: &'hash str,
    matches: Vec<IndexEntry>,
}

impl<'hash> AmbiguousHashMatchOwned<'hash> {
    /// The hash prefix that matched multiple documents.
    #[must_use]
    pub fn hash_prefix(&self) -> &'hash str {
        self.hash_prefix
    }

    /// The documents that matched the hash prefix.
    #[must_use]
    pub fn matches(&self) -> &[IndexEntry] {
        &self.matches
    }
}

/// Metadata for a document stored in the library.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct DocMetadata {
    /// Title of the document.
    pub title: String,
    /// List of authors of the document. May be empty.
    pub authors: Vec<String>,
    /// List of ISBNs for the document. May be empty.
    pub isbns: Vec<Isbn13>,
    /// File format of the document.
    pub file_format: FileFormat,
    /// DOI of the document.
    pub doi: Option<String>,
}
