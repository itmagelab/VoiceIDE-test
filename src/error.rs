//! Errors produced while parsing commit messages.

use std::fmt;

/// Result alias used throughout the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Everything that can go wrong while parsing a commit message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The message consists solely of whitespace.
    EmptyMessage,
    /// The header has no `<type>[scope][!]:` separator.
    MissingSeparator {
        /// The offending header line.
        header: String,
    },
    /// Nothing precedes the `:` in the header.
    EmptyType {
        /// The offending header line.
        header: String,
    },
    /// The scope is present but empty, as in `feat(): ...`.
    EmptyScope {
        /// The offending header line.
        header: String,
    },
    /// The type/scope part contains stray parentheses or spaces.
    InvalidScope {
        /// The offending header line.
        header: String,
    },
    /// Nothing follows the `:` in the header.
    EmptyDescription {
        /// The offending header line.
        header: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptyMessage => f.write_str("commit message is empty"),
            Error::MissingSeparator { header } => write!(
                f,
                "header `{header}` has no `<type>[scope][!]:` separator"
            ),
            Error::EmptyType { header } => {
                write!(f, "commit type in `{header}` is empty")
            }
            Error::EmptyScope { header } => {
                write!(f, "commit scope in `{header}` is empty")
            }
            Error::InvalidScope { header } => {
                write!(f, "type or scope in `{header}` is malformed")
            }
            Error::EmptyDescription { header } => {
                write!(f, "commit description in `{header}` is empty")
            }
        }
    }
}

impl std::error::Error for Error {}
