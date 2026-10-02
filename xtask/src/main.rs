use std::{env, ffi::OsString, process::Command};

const HELP: &str = "\
Tiny Tune development tasks

Usage: cargo xtask <command>

Commands:
  check         Run the complete quality gate
  format        Format all Rust sources
  format-check  Verify Rust formatting
  lint          Run Clippy with warnings denied
  test          Run all workspace tests
  release       Build the optimized Tiny Tune binary
";

fn main() {
    let result = match env::args().nth(1).as_deref() {
        Some("check") => check(),
        Some("format") => run("format", &["fmt", "--all"]),
        Some("format-check") => format_check(),
        Some("lint") => lint(),
        Some("test") => test(),
        Some("release") => release(),
        Some(command) => Err(format!("unknown command: {command}\n\n{HELP}")),
        None => Err(HELP.to_owned()),
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn check() -> Result<(), String> {
    format_check()?;
    lint()?;
    test()?;
    release()
}

fn format_check() -> Result<(), String> {
    run("format check", &["fmt", "--all", "--", "--check"])
}

fn lint() -> Result<(), String> {
    run(
        "Clippy",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )
}

fn test() -> Result<(), String> {
    run("tests", &["test", "--workspace"])
}

fn release() -> Result<(), String> {
    run(
        "release build",
        &["build", "--release", "--package", "tiny-tune"],
    )
}

fn run(label: &str, arguments: &[&str]) -> Result<(), String> {
    println!("==> {label}");
    let status = Command::new(cargo())
        .args(arguments)
        .status()
        .map_err(|error| format!("failed to start cargo for {label}: {error}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("{label} failed with {status}"))
    }
}

fn cargo() -> OsString {
    env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"))
}
