//! Classify a few sample commit messages and print their changelog sections.

use conventional::parse;

const SAMPLES: &[&str] = &[
    "feat(parser)!: accept scopes containing dashes",
    "fix: stop panicking on empty commit messages",
    "docs: expand the README with usage examples",
    "refactor(core): flatten the footer scanner",
    "chore(deps): bump the Rust toolchain",
];

fn main() {
    for message in SAMPLES {
        let commit = parse(message).expect("the samples are valid commits");

        let scope = commit.scope.as_deref().unwrap_or("-");
        let breaking = if commit.breaking { "  (breaking)" } else { "" };

        println!(
            "{:>26} <- {}({}){}",
            commit.section().heading(),
            commit.kind.as_str(),
            scope,
            breaking,
        );
    }
}
