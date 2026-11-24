//! Errors for operations that can fail outright (I/O, unsafe archives), as opposed to theme
//! problems, which are reported as [`crate::Issue`]s.

/// A failure that stops an operation.
#[derive(Debug, thiserror::Error)]
pub enum ThemeError {
    /// A file or folder could not be read or written.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// The ZIP file is corrupt or unreadable.
    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// The folder has no `theme.json`.
    #[error("no theme.json in the folder")]
    MissingManifest,
    /// The ZIP contains no theme at all.
    #[error("the archive contains no theme")]
    NoThemeInArchive,
    /// An archive entry path is absolute, contains `..` or is otherwise unsafe.
    #[error("unsafe path in archive: {0}")]
    UnsafePath(String),
    /// An archive entry is a symbolic link.
    #[error("symbolic link in archive: {0}")]
    SymlinkEntry(String),
    /// The archive has more entries than allowed.
    #[error("too many entries in archive")]
    TooManyEntries,
    /// The archive expands to more bytes than allowed (in total or for one entry).
    #[error("archive expands to too many bytes")]
    TooLarge,
    /// An entry compresses suspiciously well (zip bomb).
    #[error("compression ratio too high for: {0}")]
    CompressionRatio(String),
}
