use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::discovery::Discovered;

#[derive(Debug, Clone)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub definition: String,
    pub example: Option<String>,
    pub category: String,
}

#[derive(Deserialize, Default)]
struct RecallConfig {
    #[serde(default)]
    category_order: Vec<String>,
    #[serde(default)]
    commands: Vec<CommandEntry>,
}

#[derive(Deserialize)]
struct CommandEntry {
    name: String,
    category: Option<String>,
    description: Option<String>,
    example: Option<String>,
}

pub fn load_and_merge(
    discovered: &[Discovered],
    config_path: Option<&Path>,
) -> (Vec<Command>, Vec<String>) {
    let config = load_config(config_path);
    let discovered_names: Vec<&str> = discovered.iter().map(|d| d.name.as_str()).collect();

    let mut overrides: HashMap<&str, &CommandEntry> = HashMap::new();
    let mut static_commands: Vec<Command> = Vec::new();

    for entry in &config.commands {
        if discovered_names.contains(&entry.name.as_str()) {
            overrides.insert(&entry.name, entry);
        } else {
            static_commands.push(Command {
                name: entry.name.clone(),
                description: entry.description.clone().unwrap_or_default(),
                definition: String::new(),
                example: entry.example.clone(),
                category: entry.category.clone().unwrap_or_else(|| "Other".into()),
            });
        }
    }

    let mut commands: Vec<Command> = discovered
        .iter()
        .map(|disc| {
            let over = overrides.get(disc.name.as_str());
            Command {
                name: disc.name.clone(),
                description: over
                    .and_then(|o| o.description.clone())
                    .unwrap_or_else(|| disc.definition.clone()),
                definition: disc.definition.clone(),
                example: over.and_then(|o| o.example.clone()),
                category: over
                    .and_then(|o| o.category.clone())
                    .unwrap_or_else(|| "Other".into()),
            }
        })
        .collect();

    commands.extend(static_commands);

    let category_order = build_category_order(&config.category_order, &commands);

    commands.sort_by(|a, b| {
        let ai = category_pos(&category_order, &a.category);
        let bi = category_pos(&category_order, &b.category);
        ai.cmp(&bi).then(a.name.cmp(&b.name))
    });

    (commands, category_order)
}

fn default_config_path() -> PathBuf {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("recall/recall.toml");
    }
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".config/recall/recall.toml")
}

fn load_config(override_path: Option<&Path>) -> RecallConfig {
    let path = override_path.map_or_else(default_config_path, PathBuf::from);
    fs::read_to_string(path)
        .ok()
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_default()
}

fn build_category_order(configured: &[String], commands: &[Command]) -> Vec<String> {
    let mut order = configured.to_vec();
    for cmd in commands {
        if !order.contains(&cmd.category) {
            order.push(cmd.category.clone());
        }
    }
    order
}

fn category_pos(order: &[String], cat: &str) -> usize {
    order.iter().position(|c| c == cat).unwrap_or(usize::MAX)
}
