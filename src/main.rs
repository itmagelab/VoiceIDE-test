//! CLI front-end for the `conventional` library.
//!
//! Each command-line argument is treated as one commit message. With no
//! arguments, messages are read from stdin and separated by blank lines.
//! Each parsed commit is printed with its changelog section; the exit status
//! is non-zero if any message failed to parse.

#![forbid(unsafe_code)]

use std::io::{self, Read};
use std::process::ExitCode;

use conventional::{parse, Commit};

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();

    let messages = if arguments.is_empty() {
        match read_stdin() {
            Ok(messages) => messages,
            Err(error) => {
                eprintln!("error: could not read stdin: {error}");
                return ExitCode::from(2);
            }
        }
    } else {
        arguments
    };

    if messages.is_empty() {
        eprintln!("usage: conventional <message>...");
        eprintln!("       or commit messages on stdin, separated by blank lines");
        return ExitCode::from(2);
    }

    let mut failures = 0;
    for message in &messages {
        match parse(message) {
            Ok(commit) => print_commit(message, &commit),
            Err(error) => {
                failures += 1;
                eprintln!("error: {error}");
            }
        }
    }

    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Read stdin and split it into messages on blank lines.
fn read_stdin() -> io::Result<Vec<String>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;

    Ok(input
        .split("\n\n")
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(str::to_owned)
        .collect())
}

/// Print one parsed commit together with its classification.
fn print_commit(source: &str, commit: &Commit) {
    println!("{}", source.trim_end());
    println!("  section:  {}", commit.section().heading());
    println!("  type:     {}", commit.kind.as_str());
    println!("  scope:    {}", commit.scope.as_deref().unwrap_or("-"));
    println!("  breaking: {}", if commit.breaking { "yes" } else { "no" });
    println!("  body:     {}", commit.body.as_deref().unwrap_or("-"));
    for footer in &commit.footers {
        println!("  footer:   {}: {}", footer.token, footer.value);
    }
}
