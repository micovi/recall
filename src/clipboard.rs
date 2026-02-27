use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{bail, Result};

pub fn copy(text: &str) -> Result<()> {
    let (cmd, args) = clipboard_command()?;
    let mut child = Command::new(cmd).args(args).stdin(Stdio::piped()).spawn()?;
    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

fn clipboard_command() -> Result<(&'static str, Vec<&'static str>)> {
    if cfg!(target_os = "macos") {
        return Ok(("pbcopy", vec![]));
    }

    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return Ok(("wl-copy", vec![]));
    }

    if which_exists("xclip") {
        return Ok(("xclip", vec!["-selection", "clipboard"]));
    }

    if which_exists("xsel") {
        return Ok(("xsel", vec!["--clipboard", "--input"]));
    }

    bail!(
        "No clipboard tool found. Install one of: \
         xclip, xsel (X11) or wl-copy (Wayland)"
    )
}

fn which_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}
