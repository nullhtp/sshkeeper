use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use std::path::PathBuf;

use crate::model::Connection;
use crate::ssh::transfer::TransferDirection;
use super::file_tree::{FileTree, FileTreeAction};
use super::remote_file_tree::{RemoteFileTree, RemoteTreeAction};
use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Local,
    Remote,
}

pub struct TransferScreen {
    pub local_tree: FileTree,
    pub remote_tree: RemoteFileTree,
    pub direction: TransferDirection,
    pub local_path: Option<PathBuf>,
    pub remote_path: Option<String>,
    pub recursive: bool,
    pub focused_pane: Pane,
    pub connection_name: String,
}

impl TransferScreen {
    pub fn new(conn: Connection) -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let connection_name = conn.name.clone();
        Self {
            local_tree: FileTree::new(&home),
            remote_tree: RemoteFileTree::new(conn),
            direction: TransferDirection::Upload,
            local_path: None,
            remote_path: None,
            recursive: false,
            focused_pane: Pane::Local,
            connection_name,
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let chunks = Layout::vertical([
            Constraint::Length(1),  // title
            Constraint::Min(1),    // trees
            Constraint::Length(3), // status bar
            Constraint::Length(1), // help
        ])
        .split(frame.area());

        // Title
        let dir_indicator = match self.direction {
            TransferDirection::Upload => "Upload →",
            TransferDirection::Download => "← Download",
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!(" Transfer: {} ", self.connection_name),
                    theme::TITLE_STYLE,
                ),
                Span::styled(format!("  [{}]", dir_indicator), theme::ACTIVE_TAB_STYLE),
                if self.recursive {
                    Span::styled("  [recursive]", theme::TOGGLE_ON_STYLE)
                } else {
                    Span::raw("")
                },
            ])),
            chunks[0],
        );

        // Two tree panes side by side
        let trees = Layout::horizontal([
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ])
        .split(chunks[1]);

        self.local_tree
            .render(frame, trees[0], self.focused_pane == Pane::Local);
        self.remote_tree
            .render(frame, trees[1], self.focused_pane == Pane::Remote);

        // Status bar: selected paths + history hint
        let local_display = self
            .local_path
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| "(none)".into());
        let remote_display = self
            .remote_path
            .as_deref()
            .unwrap_or("(none)");

        let (src_label, dst_label, src_path, dst_path) = match self.direction {
            TransferDirection::Upload => ("Local", "Remote", local_display.as_str(), remote_display),
            TransferDirection::Download => ("Remote", "Local", remote_display, local_display.as_str()),
        };

        let status_lines = vec![
            Line::from(vec![
                Span::styled("  Source: ", theme::FIELD_LABEL_STYLE),
                Span::styled(format!("[{}] ", src_label), theme::DIM_STYLE),
                Span::styled(src_path, theme::FIELD_VALUE_STYLE),
            ]),
            Line::from(vec![
                Span::styled("  Dest:   ", theme::FIELD_LABEL_STYLE),
                Span::styled(format!("[{}] ", dst_label), theme::DIM_STYLE),
                Span::styled(dst_path, theme::FIELD_VALUE_STYLE),
            ]),
        ];
        frame.render_widget(Paragraph::new(status_lines), chunks[2]);

        // Help line
        let any_jump = self.local_tree.is_jump_mode() || self.remote_tree.is_jump_mode();
        let help = if any_jump {
            " Type path, Enter: go | Esc: cancel"
        } else {
            " Tab: switch pane | Space: select | d: direction | r: recursive | Enter: transfer | Esc: back"
        };
        frame.render_widget(
            Paragraph::new(help).style(theme::HINT_STYLE),
            chunks[3],
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> TransferAction {
        // Route to jump mode if active
        if self.local_tree.is_jump_mode() {
            self.local_tree.handle_key(key);
            return TransferAction::None;
        }
        if self.remote_tree.is_jump_mode() {
            self.remote_tree.handle_key(key);
            return TransferAction::None;
        }

        // Global keys
        match key.code {
            KeyCode::Esc => return TransferAction::Cancel,
            KeyCode::Tab => {
                self.focused_pane = match self.focused_pane {
                    Pane::Local => Pane::Remote,
                    Pane::Remote => Pane::Local,
                };
                return TransferAction::None;
            }
            KeyCode::Char('d') => {
                self.direction = match self.direction {
                    TransferDirection::Upload => TransferDirection::Download,
                    TransferDirection::Download => TransferDirection::Upload,
                };
                return TransferAction::None;
            }
            KeyCode::Char('r') => {
                self.recursive = !self.recursive;
                return TransferAction::None;
            }
            _ => {}
        }

        match self.focused_pane {
            Pane::Local => {
                // Enter on local tree when both paths selected => execute
                if key.code == KeyCode::Enter
                    && self.local_path.is_some()
                    && self.remote_path.is_some()
                {
                    // But only if cursor is not on a directory (Enter expands dirs)
                    let on_dir = self.local_tree.cursor < self.local_tree.nodes.len()
                        && self.local_tree.nodes[self.local_tree.cursor].is_dir;
                    if !on_dir {
                        return self.try_execute();
                    }
                }

                match self.local_tree.handle_key(key) {
                    FileTreeAction::None => {}
                    FileTreeAction::Selected { path, is_dir } => {
                        self.local_path = Some(path);
                        self.recursive = is_dir;
                    }
                }
                TransferAction::None
            }
            Pane::Remote => {
                // Enter on remote tree when both paths selected => execute
                if key.code == KeyCode::Enter
                    && self.local_path.is_some()
                    && self.remote_path.is_some()
                {
                    let on_dir = self.remote_tree.cursor < self.remote_tree.nodes.len()
                        && self.remote_tree.nodes[self.remote_tree.cursor].is_dir;
                    if !on_dir {
                        return self.try_execute();
                    }
                }

                match self.remote_tree.handle_key(key) {
                    RemoteTreeAction::None => {}
                    RemoteTreeAction::Selected { path, is_dir } => {
                        self.remote_path = Some(path);
                        if is_dir {
                            self.recursive = true;
                        }
                    }
                }
                TransferAction::None
            }
        }
    }

    fn try_execute(&self) -> TransferAction {
        let local = match &self.local_path {
            Some(p) => p.to_string_lossy().to_string(),
            None => return TransferAction::None,
        };
        let remote = match &self.remote_path {
            Some(p) => p.clone(),
            None => return TransferAction::None,
        };

        TransferAction::Execute {
            local_path: local,
            remote_path: remote,
            direction: self.direction,
            recursive: self.recursive,
        }
    }
}

pub enum TransferAction {
    None,
    Cancel,
    Execute {
        local_path: String,
        remote_path: String,
        direction: TransferDirection,
        recursive: bool,
    },
}
