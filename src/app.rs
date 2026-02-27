use std::path::PathBuf;

use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;

use crate::cli::{Args, ShellMode};
use crate::config::{self, Command};
use crate::discovery;
use crate::search;

pub enum Mode {
    Normal,
    Search,
}

pub enum Action {
    Quit,
    Copy(String),
    Continue,
}

pub struct App {
    pub commands: Vec<Command>,
    pub categories: Vec<String>,
    pub selected_tab: usize,
    pub search_query: String,
    pub mode: Mode,
    pub visible: Vec<usize>,
    pub list_state: ListState,
}

impl App {
    pub fn new(args: &Args) -> Result<Self> {
        let shell_configs = resolve_shell_configs(args);

        let mut discovered = Vec::new();
        for path in &shell_configs {
            if let Some(path_str) = path.to_str() {
                if path.exists() {
                    discovered.extend(discovery::parse_shell_config(path_str)?);
                }
            }
        }

        let config_path = args.config.as_deref();
        let (commands, categories) = config::load_and_merge(&discovered, config_path);

        let mut app = Self {
            visible: (0..commands.len()).collect(),
            commands,
            categories,
            selected_tab: 0,
            search_query: String::new(),
            mode: Mode::Normal,
            list_state: ListState::default().with_selected(Some(0)),
        };
        app.refresh_visible();
        Ok(app)
    }

    pub fn refresh_visible(&mut self) {
        if !self.search_query.is_empty() {
            let mut scored: Vec<(usize, i64)> = self
                .commands
                .iter()
                .enumerate()
                .filter_map(|(i, cmd)| {
                    let name_score = search::fuzzy_match(&self.search_query, &cmd.name);
                    let desc_score = search::fuzzy_match(&self.search_query, &cmd.description);
                    let score = match (name_score, desc_score) {
                        (Some(n), Some(d)) => Some(n * 2 + d),
                        (Some(n), None) => Some(n * 2),
                        (None, Some(d)) => Some(d),
                        (None, None) => None,
                    };
                    score.map(|s| (i, s))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            self.visible = scored.into_iter().map(|(i, _)| i).collect();
        } else if self.selected_tab < self.categories.len() {
            let cat = &self.categories[self.selected_tab];
            self.visible = self
                .commands
                .iter()
                .enumerate()
                .filter(|(_, cmd)| &cmd.category == cat)
                .map(|(i, _)| i)
                .collect();
        } else {
            self.visible = (0..self.commands.len()).collect();
        }

        let sel = if self.visible.is_empty() {
            None
        } else {
            let current = self
                .list_state
                .selected()
                .unwrap_or(0)
                .min(self.visible.len() - 1);
            Some(current)
        };
        self.list_state.select(sel);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            return Action::Quit;
        }

        match self.mode {
            Mode::Normal => self.handle_normal(key),
            Mode::Search => self.handle_search(key),
        }
    }

    fn selected_command_name(&self) -> Option<String> {
        let sel = self.list_state.selected()?;
        self.visible
            .get(sel)
            .map(|&i| self.commands[i].name.clone())
    }

    fn select_next(&mut self) {
        if self.visible.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let max = self.visible.len() - 1;
        self.list_state.select(Some((current + 1).min(max)));
    }

    fn select_prev(&mut self) {
        let current = self.list_state.selected().unwrap_or(0);
        self.list_state.select(Some(current.saturating_sub(1)));
    }

    fn handle_normal(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => Action::Quit,
            KeyCode::Char('j') | KeyCode::Down => {
                self.select_next();
                Action::Continue
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.select_prev();
                Action::Continue
            }
            KeyCode::Tab => {
                self.selected_tab = (self.selected_tab + 1) % (self.categories.len() + 1);
                self.list_state.select(Some(0));
                self.refresh_visible();
                Action::Continue
            }
            KeyCode::BackTab => {
                if self.selected_tab == 0 {
                    self.selected_tab = self.categories.len();
                } else {
                    self.selected_tab -= 1;
                }
                self.list_state.select(Some(0));
                self.refresh_visible();
                Action::Continue
            }
            KeyCode::Char('/') => {
                self.mode = Mode::Search;
                self.search_query.clear();
                Action::Continue
            }
            KeyCode::Enter => self
                .selected_command_name()
                .map_or(Action::Continue, Action::Copy),
            _ => Action::Continue,
        }
    }

    fn handle_search(&mut self, key: KeyEvent) -> Action {
        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::Normal;
                self.search_query.clear();
                self.refresh_visible();
                Action::Continue
            }
            KeyCode::Enter => self
                .selected_command_name()
                .map_or(Action::Continue, Action::Copy),
            KeyCode::Backspace => {
                self.search_query.pop();
                self.refresh_visible();
                Action::Continue
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.refresh_visible();
                Action::Continue
            }
            KeyCode::Down => {
                self.select_next();
                Action::Continue
            }
            KeyCode::Up => {
                self.select_prev();
                Action::Continue
            }
            _ => Action::Continue,
        }
    }
}

fn resolve_shell_configs(args: &Args) -> Vec<PathBuf> {
    if !args.shell_configs.is_empty() {
        return args.shell_configs.clone();
    }

    let home = std::env::var("HOME").unwrap_or_default();

    match args.shell {
        ShellMode::Zsh => vec![PathBuf::from(format!("{home}/.zshrc"))],
        ShellMode::Bash => {
            vec![PathBuf::from(format!("{home}/.bashrc"))]
        }
        ShellMode::Both => vec![
            PathBuf::from(format!("{home}/.zshrc")),
            PathBuf::from(format!("{home}/.bashrc")),
        ],
        ShellMode::Auto => {
            let shell = std::env::var("SHELL").unwrap_or_default();
            if shell.ends_with("/bash") {
                vec![PathBuf::from(format!("{home}/.bashrc"))]
            } else {
                vec![PathBuf::from(format!("{home}/.zshrc"))]
            }
        }
    }
}
