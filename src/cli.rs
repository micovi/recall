use std::path::PathBuf;

use anyhow::{bail, Result};

#[derive(Debug, Default)]
pub struct Args {
    pub config: Option<PathBuf>,
    pub shell: ShellMode,
    pub shell_configs: Vec<PathBuf>,
}

#[derive(Debug, Default, Clone, Copy)]
pub enum ShellMode {
    #[default]
    Auto,
    Zsh,
    Bash,
    Both,
}

pub fn parse() -> Result<Args> {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut args = Args::default();
    let mut i = 0;

    while i < raw.len() {
        match raw[i].as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--version" | "-V" => {
                println!("recall {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--config" => {
                i += 1;
                let path = require_value(&raw, i, "--config")?;
                args.config = Some(PathBuf::from(path));
            }
            "--shell" => {
                i += 1;
                let val = require_value(&raw, i, "--shell")?;
                args.shell = match val.as_str() {
                    "zsh" => ShellMode::Zsh,
                    "bash" => ShellMode::Bash,
                    "both" => ShellMode::Both,
                    other => bail!(
                        "Unknown shell '{other}'. \
                         Expected: zsh, bash, both"
                    ),
                };
            }
            "--shell-config" => {
                i += 1;
                let path = require_value(&raw, i, "--shell-config")?;
                args.shell_configs.push(PathBuf::from(path));
            }
            other => bail!("Unknown argument: {other}"),
        }
        i += 1;
    }

    Ok(args)
}

fn require_value(raw: &[String], i: usize, flag: &str) -> Result<String> {
    raw.get(i)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("{flag} requires a value"))
}

fn print_help() {
    println!(
        "\
recall - searchable TUI for shell aliases and functions

USAGE:
    recall [OPTIONS]

OPTIONS:
    --config PATH         Config file path \
(default: ~/.config/recall/recall.toml)
    --shell zsh|bash|both Shell to scan \
(default: auto-detect from $SHELL)
    --shell-config PATH   Explicit shell config path (repeatable)
    -h, --help            Show this help
    -V, --version         Show version

KEYBINDINGS:
    /          Search
    j/k        Navigate up/down
    Tab        Next category
    Shift+Tab  Previous category
    Enter      Copy selected command
    q, Esc     Quit"
    );
}
