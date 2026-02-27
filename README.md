# recall

A searchable TUI for your shell aliases and functions.

recall auto-discovers aliases and functions from your shell config and presents
them in a fuzzy-searchable, categorized terminal interface. Select a command and
it's copied to your clipboard.

## Install

```sh
cargo install recall-cli
```

Or build from source:

```sh
git clone https://github.com/micovi/recall
cd recall
cargo build --release
```

## Usage

```sh
recall                          # auto-detect shell from $SHELL
recall --shell bash             # scan ~/.bashrc
recall --shell both             # scan both ~/.zshrc and ~/.bashrc
recall --shell-config ~/my.sh   # scan a specific file
recall --config ~/recall.toml   # custom config path
```

## Keybindings

| Key         | Action            |
|-------------|-------------------|
| `/`         | Search            |
| `j` / `k`   | Navigate down/up  |
| `Tab`       | Next category     |
| `Shift+Tab` | Previous category |
| `Enter`     | Copy to clipboard |
| `q` / `Esc` | Quit              |

## Config

Default location: `~/.config/recall/recall.toml`
(respects `$XDG_CONFIG_HOME`)

```toml
category_order = ["Git", "Docker", "Navigation"]

[[commands]]
name = "myalias"
category = "Custom"
description = "Does something useful"
example = "myalias --flag value"
```

Use `category` and `description` to override auto-discovered commands, or define
static commands that aren't in your shell config.

## Clipboard support

| Platform | Tool       |
|----------|------------|
| macOS    | `pbcopy`   |
| Wayland  | `wl-copy`  |
| X11      | `xclip` or `xsel` |

## License

MIT
