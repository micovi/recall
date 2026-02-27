# recall

Searchable TUI for shell aliases and functions. Distributed as `cargo install recall-cli`, binary name `recall`.

## Architecture

```
src/
  main.rs        Entry point — CLI parse, TUI loop, clipboard on exit
  cli.rs         Hand-parsed args (no clap) — --config, --shell, --shell-config
  app.rs         App state, key handling, shell config resolution
  config.rs      TOML config loading (~/.config/recall/recall.toml), merge with discovered commands
  discovery.rs   Parse aliases and functions from shell rc files (zsh + bash)
  search.rs      Fuzzy matching with scoring (prefix, boundary, consecutive bonuses)
  clipboard.rs   Platform-aware clipboard (pbcopy, wl-copy, xclip, xsel)
  ui.rs          Ratatui rendering — tabs, list, preview, status bar
```

## Commands

```sh
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Conventions

- Zero new dependencies unless absolutely necessary — clipboard, CLI, XDG paths all use stdlib
- No clap — CLI is hand-parsed in `cli.rs`
- Config struct is `RecallConfig` (not `CheatConfig`) — renamed from original `cheat` project
- Config path respects `XDG_CONFIG_HOME`, defaults to `~/.config/recall/recall.toml`
- Shell auto-detection from `$SHELL` env var, falls back to zsh
- Missing shell config files are silently skipped (not errors)
