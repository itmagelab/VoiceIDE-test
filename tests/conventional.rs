//! End-to-end tests for the public API of the `conventional` crate.

use conventional::{parse, CommitKind, Error, Section};

#[test]
fn header_bang_marker_marks_a_breaking_change() {
    let commit = parse("feat(api)!: send password reset emails").unwrap();

    assert_eq!(commit.kind, CommitKind::Feat);
    assert_eq!(commit.scope.as_deref(), Some("api"));
    assert!(commit.breaking);
    assert_eq!(commit.section(), Section::Features);
}

#[test]
fn every_recommended_type_lands_in_its_section() {
    let cases = [
        ("feat", Section::Features),
        ("fix", Section::BugFixes),
        ("docs", Section::Documentation),
        ("style", Section::Styles),
        ("refactor", Section::Refactoring),
        ("perf", Section::Performance),
        ("test", Section::Testing),
        ("build", Section::BuildSystem),
        ("ci", Section::ContinuousIntegration),
        ("chore", Section::MiscellaneousChores),
        ("revert", Section::Reverts),
    ];

    for (kind, section) in cases {
        let commit = parse(&format!("{kind}: do something")).unwrap();

        assert_eq!(commit.kind.as_str(), kind);
        assert_eq!(commit.section(), section, "wrong section for `{kind}`");
    }
}

#[test]
fn crlf_line_endings_are_normalised() {
    let message = concat!(
        "feat(config)!: switch to TOML\r\n",
        "\r\n",
        "The old INI loader is gone.\r\n",
        "\r\n",
        "BREAKING CHANGE: drop INI support",
    );

    let commit = parse(message).unwrap();

    assert!(commit.breaking);
    assert_eq!(commit.description, "switch to TOML");
    assert_eq!(commit.body.as_deref(), Some("The old INI loader is gone."));
    assert_eq!(commit.footers.len(), 1);
}

#[test]
fn footer_only_messages_have_no_body() {
    let commit = parse("chore: bump dependencies\n\nRefs #42").unwrap();

    assert_eq!(commit.body, None);
    assert_eq!(commit.footers.len(), 1);
    assert_eq!(commit.footers[0].token, "Refs");
    assert_eq!(commit.footers[0].value, "42");
}

#[test]
fn plain_subjects_are_rejected() {
    let error = parse("just a plain subject line").unwrap_err();

    assert!(matches!(error, Error::MissingSeparator { .. }));
}

#[test]
fn unbalanced_scopes_are_rejected() {
    let error = parse("feat(api: describe the change").unwrap_err();

    assert!(matches!(error, Error::InvalidScope { .. }));
}

#[test]
fn section_headings_are_human_readable() {
    let commit = parse("feat: dark mode").unwrap();

    assert_eq!(commit.section().heading(), "Features");
}
