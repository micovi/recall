use anyhow::Result;
use std::fs;

#[derive(Debug)]
pub struct Discovered {
    pub name: String,
    pub definition: String,
}

pub fn parse_shell_config(path: &str) -> Result<Vec<Discovered>> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let mut commands = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty() || line.starts_with('#') {
            i += 1;
            continue;
        }

        if let Some(cmd) = parse_alias(line) {
            commands.push(cmd);
            i += 1;
            continue;
        }

        if let Some(name) = parse_function_start(line) {
            let definition = extract_function_body(line, &lines, &mut i);
            commands.push(Discovered { name, definition });
            continue;
        }

        i += 1;
    }

    Ok(commands)
}

fn parse_alias(line: &str) -> Option<Discovered> {
    let rest = line.strip_prefix("alias ")?;
    let eq_pos = rest.find('=')?;
    let name = rest[..eq_pos].to_string();
    let value = &rest[eq_pos + 1..];

    let definition = if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value[1..value.len() - 1].to_string()
    } else {
        value.to_string()
    };

    Some(Discovered { name, definition })
}

fn parse_function_start(line: &str) -> Option<String> {
    let paren_pos = line.find("()")?;
    let before = line[..paren_pos].trim();

    if before.is_empty() {
        return None;
    }

    let first = before.chars().next()?;
    if !first.is_ascii_alphabetic() && first != '_' {
        return None;
    }
    if !before
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return None;
    }

    let after = line[paren_pos + 2..].trim();
    if !after.starts_with('{') {
        return None;
    }

    let keywords = [
        "if", "then", "else", "elif", "fi", "for", "while", "do", "done", "case", "esac",
    ];
    if keywords.contains(&before) {
        return None;
    }

    Some(before.to_string())
}

fn extract_function_body(first_line: &str, lines: &[&str], i: &mut usize) -> String {
    // One-liner: name() { body; }
    if first_line.contains('}') {
        *i += 1;
        return first_line
            .find('{')
            .and_then(|start| {
                first_line
                    .rfind('}')
                    .map(|end| first_line[start + 1..end].trim().to_string())
            })
            .unwrap_or_default();
    }

    // Multi-line function
    let mut body_lines = Vec::new();
    let mut depth: usize = 1;
    *i += 1;

    while *i < lines.len() && depth > 0 {
        let fline = lines[*i].trim();
        depth += fline.matches('{').count();
        depth = depth.saturating_sub(fline.matches('}').count());
        if depth > 0 && !fline.is_empty() && !fline.starts_with('#') {
            body_lines.push(fline);
        }
        *i += 1;
    }

    body_lines
        .into_iter()
        .take(3)
        .collect::<Vec<_>>()
        .join("; ")
}
