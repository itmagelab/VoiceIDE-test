//! Parsing of [Conventional Commits](https://www.conventionalcommits.org)
//! messages.
//!
//! The parser covers the practical subset of the specification: a
//! `<type>[scope][!]: <description>` header, an optional body and an optional
//! footer block, including the `BREAKING CHANGE:` footer. Anything it cannot
//! make sense of becomes an [`Error`] instead of a guessed commit.

use crate::error::{Error, Result};

/// A single parsed commit message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    /// The type of change, e.g. `feat` or `fix`.
    pub kind: CommitKind,
    /// The optional scope, e.g. `api` in `feat(api): ...`.
    pub scope: Option<String>,
    /// Whether the commit is marked as a breaking change, either through a
    /// trailing `!` in the header or a `BREAKING CHANGE:` footer.
    pub breaking: bool,
    /// The subject following the `:` in the header.
    pub description: String,
    /// The free-form body between the header and the footer block.
    pub body: Option<String>,
    /// Structured footers, e.g. `Reviewed-by: Z` or `Closes #12`.
    pub footers: Vec<Footer>,
}

impl Commit {
    /// The changelog section this commit belongs to.
    pub fn section(&self) -> Section {
        self.kind.section()
    }
}

/// A single footer line, e.g. `Reviewed-by: Z` or `Closes #12`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Footer {
    /// The footer token, e.g. `Reviewed-by`, `Closes` or `BREAKING CHANGE`.
    pub token: String,
    /// Everything after the separator, trimmed.
    pub value: String,
}

impl Footer {
    /// Whether this footer marks a breaking change.
    pub fn is_breaking(&self) -> bool {
        self.token == "BREAKING CHANGE"
    }
}

/// The type of a conventional commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommitKind {
    /// New feature (`feat`).
    Feat,
    /// Bug fix (`fix`).
    Fix,
    /// Documentation only (`docs`).
    Docs,
    /// Code style: formatting, whitespace, typos (`style`).
    Style,
    /// Refactor that neither fixes a bug nor adds a feature (`refactor`).
    Refactor,
    /// Performance improvement (`perf`).
    Perf,
    /// Adding or correcting tests (`test`).
    Test,
    /// Build system or dependency changes (`build`).
    Build,
    /// CI configuration and pipelines (`ci`).
    Ci,
    /// Maintenance that touches neither `src` nor tests (`chore`).
    Chore,
    /// Reverts a previous commit (`revert`).
    Revert,
    /// Any type outside the recommended set.
    Other(String),
}

impl CommitKind {
    /// Turn the raw type from the header into a kind, falling back to
    /// [`CommitKind::Other`] for anything outside the recommended set.
    pub fn from_raw(raw: &str) -> Self {
        match raw {
            "feat" => Self::Feat,
            "fix" => Self::Fix,
            "docs" => Self::Docs,
            "style" => Self::Style,
            "refactor" => Self::Refactor,
            "perf" => Self::Perf,
            "test" => Self::Test,
            "build" => Self::Build,
            "ci" => Self::Ci,
            "chore" => Self::Chore,
            "revert" => Self::Revert,
            other => Self::Other(other.to_owned()),
        }
    }

    /// The type keyword as it appears in the header.
    pub fn as_str(&self) -> &str {
        match self {
            Self::Feat => "feat",
            Self::Fix => "fix",
            Self::Docs => "docs",
            Self::Style => "style",
            Self::Refactor => "refactor",
            Self::Perf => "perf",
            Self::Test => "test",
            Self::Build => "build",
            Self::Ci => "ci",
            Self::Chore => "chore",
            Self::Revert => "revert",
            Self::Other(raw) => raw,
        }
    }

    /// The changelog section a commit of this type lands in.
    pub fn section(&self) -> Section {
        match self {
            Self::Feat => Section::Features,
            Self::Fix => Section::BugFixes,
            Self::Docs => Section::Documentation,
            Self::Style => Section::Styles,
            Self::Refactor => Section::Refactoring,
            Self::Perf => Section::Performance,
            Self::Test => Section::Testing,
            Self::Build => Section::BuildSystem,
            Self::Ci => Section::ContinuousIntegration,
            Self::Chore => Section::MiscellaneousChores,
            Self::Revert => Section::Reverts,
            Self::Other(_) => Section::Other,
        }
    }
}

/// The changelog section a commit is grouped under.
///
/// The headings mirror the defaults used by
/// [git-cliff](https://git-cliff.org) so both tools agree on a changelog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Changes landed by `feat` commits.
    Features,
    /// Changes landed by `fix` commits.
    BugFixes,
    /// Changes landed by `docs` commits.
    Documentation,
    /// Changes landed by `style` commits.
    Styles,
    /// Changes landed by `refactor` commits.
    Refactoring,
    /// Changes landed by `perf` commits.
    Performance,
    /// Changes landed by `test` commits.
    Testing,
    /// Changes landed by `build` commits.
    BuildSystem,
    /// Changes landed by `ci` commits.
    ContinuousIntegration,
    /// Changes landed by `chore` commits.
    MiscellaneousChores,
    /// Changes landed by `revert` commits.
    Reverts,
    /// Anything typed outside the recommended set.
    Other,
}

impl Section {
    /// The human-readable heading for the section.
    pub fn heading(self) -> &'static str {
        match self {
            Section::Features => "Features",
            Section::BugFixes => "Bug Fixes",
            Section::Documentation => "Documentation",
            Section::Styles => "Styles",
            Section::Refactoring => "Refactoring",
            Section::Performance => "Performance",
            Section::Testing => "Testing",
            Section::BuildSystem => "Build System",
            Section::ContinuousIntegration => "Continuous Integration",
            Section::MiscellaneousChores => "Miscellaneous Chores",
            Section::Reverts => "Reverts",
            Section::Other => "Other",
        }
    }
}

/// Parse a full commit message into its parts.
///
/// The message is trimmed and CRLF line endings are normalised before
/// parsing, so messages coming from `git log` or a text field behave the
/// same way.
pub fn parse(message: &str) -> Result<Commit> {
    let message = message.trim().replace("\r\n", "\n");
    if message.is_empty() {
        return Err(Error::EmptyMessage);
    }

    let (header, rest) = match message.split_once('\n') {
        Some((header, rest)) => (header.trim_end(), rest),
        None => (message.as_str(), ""),
    };

    let (breaking, kind, scope, description) = parse_header(header)?;
    let (body, footers) = split_body_and_footers(rest);

    Ok(Commit {
        breaking: breaking || footers.iter().any(Footer::is_breaking),
        kind,
        scope,
        description,
        body,
        footers,
    })
}

/// Parse the `<type>[scope][!]: <description>` header line.
fn parse_header(header: &str) -> Result<(bool, CommitKind, Option<String>, String)> {
    let (raw_type, raw_description) = header
        .split_once(':')
        .ok_or_else(|| Error::MissingSeparator {
            header: header.to_owned(),
        })?;

    let description = raw_description.trim();
    if description.is_empty() {
        return Err(Error::EmptyDescription {
            header: header.to_owned(),
        });
    }

    // A trailing `!` before the separator marks a breaking change.
    let (breaking, raw_type) = match raw_type.strip_suffix('!') {
        Some(stripped) => (true, stripped),
        None => (false, raw_type),
    };

    // Split `type(scope)` into its two halves, rejecting malformed scopes.
    let (type_name, scope) = if let Some(outer) = raw_type.strip_suffix(')') {
        let open = outer.find('(').ok_or_else(|| Error::InvalidScope {
            header: header.to_owned(),
        })?;
        let inner = &outer[open + 1..];
        if inner.is_empty() {
            return Err(Error::EmptyScope {
                header: header.to_owned(),
            });
        }
        if inner.contains('(') || inner.contains(')') {
            return Err(Error::InvalidScope {
                header: header.to_owned(),
            });
        }
        (&outer[..open], Some(inner))
    } else if raw_type.contains('(') || raw_type.contains(')') {
        return Err(Error::InvalidScope {
            header: header.to_owned(),
        });
    } else {
        (raw_type, None)
    };

    if type_name.is_empty() {
        return Err(Error::EmptyType {
            header: header.to_owned(),
        });
    }
    if type_name.contains(char::is_whitespace) {
        return Err(Error::InvalidScope {
            header: header.to_owned(),
        });
    }

    Ok((
        breaking,
        CommitKind::from_raw(type_name),
        scope.map(str::to_owned),
        description.to_owned(),
    ))
}

/// Split everything after the header line into an optional body and footers.
///
/// Footers are the last blank-line-separated paragraph, but only when every
/// line in that paragraph looks like a footer or a continuation of one.
fn split_body_and_footers(rest: &str) -> (Option<String>, Vec<Footer>) {
    if rest.trim().is_empty() {
        return (None, Vec::new());
    }

    let paragraphs: Vec<&str> = rest
        .split("\n\n")
        .map(str::trim)
        .filter(|paragraph| !paragraph.is_empty())
        .collect();

    // `rest` is non-empty here, so there is always at least one paragraph.
    let (body_paragraphs, footer_paragraph) = match paragraphs.split_last() {
        Some((last, body)) => (body, Some(*last)),
        None => (&paragraphs[..], None),
    };

    let footers = match footer_paragraph.filter(|p| is_footer_block(p)) {
        Some(paragraph) => footer_block(paragraph),
        None => Vec::new(),
    };

    let body = if footers.is_empty() {
        paragraphs_body(&paragraphs)
    } else {
        paragraphs_body(body_paragraphs)
    };

    (body, footers)
}

/// Whether a paragraph consists solely of footers and their continuations.
fn is_footer_block(paragraph: &str) -> bool {
    let mut saw_footer = false;
    for line in paragraph.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            // Continuation of the previous footer.
            if !saw_footer {
                return false;
            }
        } else if parse_footer(line).is_none() {
            return false;
        } else {
            saw_footer = true;
        }
    }
    saw_footer
}

/// Turn a validated footer paragraph into `Footer` values.
fn footer_block(paragraph: &str) -> Vec<Footer> {
    let mut footers = Vec::new();
    for line in paragraph.lines() {
        if line.starts_with(' ') || line.starts_with('\t') {
            if let Some(last) = footers.last_mut() {
                last.value.push(' ');
                last.value.push_str(line.trim());
            }
        } else if let Some(footer) = parse_footer(line) {
            footers.push(footer);
        }
    }
    footers
}

/// Reassemble body paragraphs; `None` when there is no body at all.
fn paragraphs_body(paragraphs: &[&str]) -> Option<String> {
    if paragraphs.is_empty() {
        return None;
    }
    let body = paragraphs.join("\n\n");
    let trimmed = body.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

/// Parse a single footer line such as `Reviewed-by: Z` or `Closes #12`.
fn parse_footer(line: &str) -> Option<Footer> {
    // `BREAKING CHANGE` is the only token allowed to contain a space, and
    // may also be spelled `BREAKING-CHANGE`.
    for marker in ["BREAKING CHANGE: ", "BREAKING-CHANGE: "] {
        if let Some(raw) = line.strip_prefix(marker) {
            let value = raw.trim();
            if value.is_empty() {
                return None;
            }
            return Some(Footer {
                token: "BREAKING CHANGE".to_owned(),
                value: value.to_owned(),
            });
        }
    }

    let (token, rest) = split_token(line)?;

    // Either `Token: value` or `Token #123`, where the reference form must
    // be a single word.
    let value = if let Some(raw) = rest.strip_prefix(": ") {
        raw.trim()
    } else {
        let reference = rest.strip_prefix(" #")?.trim();
        if reference.contains(char::is_whitespace) {
            return None;
        }
        reference
    };

    if value.is_empty() {
        return None;
    }

    Some(Footer {
        token: token.to_owned(),
        value: value.to_owned(),
    })
}

/// Split a leading token off `line`; tokens are words of `[A-Za-z0-9-]`.
fn split_token(line: &str) -> Option<(&str, &str)> {
    let mut end = 0;
    for (index, character) in line.char_indices() {
        if character.is_ascii_alphanumeric() || character == '-' {
            end = index + character.len_utf8();
        } else {
            break;
        }
    }
    if end == 0 {
        return None;
    }
    Some((&line[..end], &line[end..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_minimal_header() {
        let commit = parse("feat: send password reset emails").unwrap();

        assert_eq!(commit.kind, CommitKind::Feat);
        assert_eq!(commit.scope, None);
        assert!(!commit.breaking);
        assert_eq!(commit.description, "send password reset emails");
        assert_eq!(commit.body, None);
        assert!(commit.footers.is_empty());
    }

    #[test]
    fn parses_scope_and_bang_marker() {
        let commit = parse("fix(api)!: stop crashing on empty input").unwrap();

        assert_eq!(commit.kind, CommitKind::Fix);
        assert_eq!(commit.scope.as_deref(), Some("api"));
        assert!(commit.breaking);
    }

    #[test]
    fn breaking_change_via_footer() {
        let message = "\
feat: switch the config format

The config file is now TOML.

BREAKING CHANGE: drop support for INI files";

        let commit = parse(message).unwrap();

        assert!(commit.breaking);
        assert_eq!(commit.body.as_deref(), Some("The config file is now TOML."));
        assert_eq!(commit.footers.len(), 1);
        assert_eq!(commit.footers[0].token, "BREAKING CHANGE");
        assert_eq!(commit.footers[0].value, "drop support for INI files");
    }

    #[test]
    fn keeps_the_body_when_there_is_no_footer() {
        let message = "\
refactor: tidy the parser

The parser now normalises CRLF line endings.

It also trims trailing whitespace.";

        let commit = parse(message).unwrap();

        assert!(commit.footers.is_empty());
        assert_eq!(
            commit.body.as_deref(),
            Some(
                "The parser now normalises CRLF line endings.\n\n\
                 It also trims trailing whitespace."
            )
        );
    }

    #[test]
    fn joins_footer_continuation_lines() {
        let message = "\
feat: multi-line footer

BREAKING CHANGE: the CLI now
  requires a subcommand";

        let commit = parse(message).unwrap();

        assert_eq!(
            commit.footers[0].value,
            "the CLI now requires a subcommand"
        );
    }

    #[test]
    fn reads_multiple_footers() {
        let message = "\
docs: expand the README

More words.

Reviewed-by: Z
Refs #1337";

        let commit = parse(message).unwrap();

        assert_eq!(commit.body.as_deref(), Some("More words."));
        assert_eq!(commit.footers.len(), 2);
        assert_eq!(commit.footers[0].token, "Reviewed-by");
        assert_eq!(commit.footers[0].value, "Z");
        assert_eq!(commit.footers[1].token, "Refs");
        assert_eq!(commit.footers[1].value, "1337");
    }

    #[test]
    fn keeps_a_body_paragraph_that_only_looks_like_a_footer() {
        let message = "chore: cleanup\n\nSee the migration guide for details.";
        let commit = parse(message).unwrap();

        assert!(commit.footers.is_empty());
        assert_eq!(
            commit.body.as_deref(),
            Some("See the migration guide for details.")
        );
    }

    #[test]
    fn unknown_types_fall_back_to_other() {
        let commit = parse("wibble: recalibrate the frobnicator").unwrap();

        assert_eq!(commit.kind, CommitKind::Other("wibble".to_owned()));
        assert_eq!(commit.kind.as_str(), "wibble");
        assert_eq!(commit.section(), Section::Other);
    }

    #[test]
    fn rejects_a_header_without_a_separator() {
        let error = parse("feat send emails").unwrap_err();

        assert_eq!(
            error,
            Error::MissingSeparator {
                header: "feat send emails".to_owned()
            }
        );
    }

    #[test]
    fn rejects_an_empty_description() {
        let error = parse("feat:   ").unwrap_err();

        assert!(matches!(error, Error::EmptyDescription { .. }));
    }

    #[test]
    fn rejects_an_empty_scope() {
        let error = parse("feat(): add a thing").unwrap_err();

        assert!(matches!(error, Error::EmptyScope { .. }));
    }

    #[test]
    fn rejects_a_malformed_scope() {
        let error = parse("fix)parser: handle stray parens").unwrap_err();

        assert!(matches!(error, Error::InvalidScope { .. }));
    }

    #[test]
    fn rejects_a_message_that_is_only_whitespace() {
        assert_eq!(parse("   \n \n"), Err(Error::EmptyMessage));
    }
}
