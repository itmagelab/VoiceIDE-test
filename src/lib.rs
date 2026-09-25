//! `conventional` — a small parser for
//! [Conventional Commits](https://www.conventionalcommits.org) messages.
//!
//! The entry point is [`parse`], which turns a raw commit message into a
//! [`Commit`] with the type, scope, breaking-change flag, description, body
//! and footers already split apart:
//!
//! ```
//! use conventional::{parse, CommitKind};
//!
//! let commit = parse("feat(api)!: send password reset emails")?;
//! assert_eq!(commit.kind, CommitKind::Feat);
//! assert_eq!(commit.scope.as_deref(), Some("api"));
//! assert!(commit.breaking);
//! # Ok::<(), conventional::Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod commit;
pub mod error;

pub use commit::{parse, Commit, CommitKind, Footer, Section};
pub use error::{Error, Result};
