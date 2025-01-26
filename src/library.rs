use {
    crate::{sha256, FileFormat, Isbn13},
    anyhow::{anyhow, bail, Context},
    serde::{Deserialize, Serialize},
    std::{
        env,
        fs::{self, File},
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
    // For now, we don't use the version field. However, it may be useful in the future if we want
    // to implement backwards compatibility or other features that depend on the version of the
    // library.
    #[allow(dead_code)]
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
                    &doc.hash.to_string()[..8]
                );
            }
            if doc.hash == hash {
                bail!(
                    "Document is already in the library ({})",
                    &hash.to_string()[..8]
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

    pub fn validate(&self) -> anyhow::Result<()> {
        unimplemented!()
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
struct IndexEntry {
    hash: sha256::Hash,
    #[serde(flatten)]
    metadata: DocMetadata,
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
}
