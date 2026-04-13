use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::process::Command;

use crate::model::Connection;
use super::theme;

const MAX_ENTRIES_PER_DIR: usize = 200;

#[derive(Debug, Clone)]
pub struct RemoteNode {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

pub struct RemoteFileTree {
    pub nodes: Vec<RemoteNode>,
    pub cursor: usize,
    pub selected_path: Option<String>,
    pub show_hidden: bool,
    pub scroll_offset: usize,
    pub current_root: String,
    pub loading_error: Option<String>,
    conn: Connection,
    jump_mode: bool,
    jump_input: String,
}

impl RemoteFileTree {
    pub fn new(conn: Connection) -> Self {
        let mut tree = Self {
            nodes: Vec::new(),
            cursor: 0,
            selected_path: None,
            show_hidden: false,
            scroll_offset: 0,
            current_root: "~".to_string(),
            loading_error: None,
            conn,
            jump_mode: false,
            jump_input: String::new(),
        };
        // Resolve ~ to actual home path on the remote
        let home = tree.resolve_home().unwrap_or_else(|| "/".to_string());
        tree.load_root(&home);
        tree
    }

    fn resolve_home(&self) -> Option<String> {
        let mut cmd = self.build_ssh_command("echo $HOME");
        let output = cmd.output().ok()?;
        if output.status.success() {
            let home = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !home.is_empty() {
                return Some(home);
            }
        }
        None
    }

    fn load_root(&mut self, root: &str) {
        self.nodes.clear();
        self.cursor = 0;
        self.scroll_offset = 0;
        self.loading_error = None;
        self.current_root = root.to_string();

        // Resolve ~ to absolute path if user typed it in jump mode
        let root = if root == "~" || root.starts_with("~/") {
            match self.resolve_home() {
                Some(home) if root == "~" => {
                    self.current_root = home.clone();
                    home
                }
                Some(home) => {
                    let resolved = format!("{}/{}", home, &root[2..]);
                    self.current_root = resolved.clone();
                    resolved
                }
                None => root.to_string(),
            }
        } else {
            root.to_string()
        };
        let root = root.as_str();

        // Add parent entry
        let parent = if root == "/" {
            None
        } else {
            // Get parent path
            let trimmed = root.trim_end_matches('/');
            let parent_path = match trimmed.rfind('/') {
                Some(pos) if pos == 0 => Some("/".to_string()),
                Some(pos) => Some(trimmed[..pos].to_string()),
                None => Some("~".to_string()),
            };
            parent_path
        };

        if let Some(parent_path) = parent {
            self.nodes.push(RemoteNode {
                path: parent_path,
                name: "..".into(),
                is_dir: true,
                depth: 0,
                expanded: false,
            });
        }

        match self.list_remote_dir(root) {
            Ok(entries) => {
                for (name, is_dir) in entries.into_iter().take(MAX_ENTRIES_PER_DIR) {
                    let path = if root == "/" {
                        format!("/{}", name)
                    } else {
                        format!("{}/{}", root.trim_end_matches('/'), name)
                    };
                    self.nodes.push(RemoteNode {
                        path,
                        name,
                        is_dir,
                        depth: 0,
                        expanded: false,
                    });
                }
            }
            Err(e) => {
                self.loading_error = Some(e);
            }
        }
    }

    fn list_remote_dir(&self, dir: &str) -> Result<Vec<(String, bool)>, String> {
        // Use ls -1pa: one entry per line, append / to dirs, include hidden
        // For ~, use $HOME so the remote shell expands it (~ inside quotes is literal)
        let dir_expr = if dir == "~" {
            "$HOME".to_string()
        } else if dir.starts_with("~/") {
            format!("$HOME/{}", &dir[2..])
        } else {
            shell_escape(dir)
        };
        let ls_cmd = format!("ls -1pa {}", dir_expr);
        let mut cmd = self.build_ssh_command(&ls_cmd);

        let output = cmd
            .output()
            .map_err(|e| format!("SSH failed: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("ls failed: {}", stderr.trim()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut entries: Vec<(String, bool)> = Vec::new();

        for line in stdout.lines() {
            let line = line.trim();
            if line.is_empty() || line == "./" || line == "../" {
                continue;
            }
            let is_dir = line.ends_with('/');
            let name = if is_dir {
                line.trim_end_matches('/').to_string()
            } else {
                line.to_string()
            };
            if name.is_empty() {
                continue;
            }
            if !self.show_hidden && name.starts_with('.') {
                continue;
            }
            entries.push((name, is_dir));
        }

        // Sort: dirs first, then alphabetical
        entries.sort_by(|a, b| {
            b.1.cmp(&a.1)
                .then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
        });

        Ok(entries)
    }

    fn build_ssh_command(&self, remote_cmd: &str) -> Command {
        let mut cmd = Command::new("ssh");
        // Batch mode to avoid interactive prompts that would hang
        cmd.arg("-o").arg("BatchMode=yes");
        // Suppress banners
        cmd.arg("-o").arg("LogLevel=ERROR");
        if self.conn.port != 22 {
            cmd.arg("-p").arg(self.conn.port.to_string());
        }
        if let Some(ref user) = self.conn.user {
            cmd.arg("-l").arg(user);
        }
        if let Some(ref key) = self.conn.identity_file {
            cmd.arg("-i").arg(key);
        }
        if let Some(ref jump) = self.conn.proxy_jump {
            cmd.arg("-J").arg(jump);
        }
        for (key, val) in &self.conn.ssh_options {
            cmd.arg("-o").arg(format!("{}={}", key, val));
        }
        cmd.arg(&self.conn.host);
        cmd.arg(remote_cmd);
        cmd
    }

    pub fn expand(&mut self, index: usize) {
        if index >= self.nodes.len() || !self.nodes[index].is_dir {
            return;
        }
        if self.nodes[index].name == ".." {
            let parent = self.nodes[index].path.clone();
            self.load_root(&parent);
            return;
        }
        if self.nodes[index].expanded {
            return;
        }

        let dir_path = self.nodes[index].path.clone();
        let depth = self.nodes[index].depth + 1;

        match self.list_remote_dir(&dir_path) {
            Ok(entries) => {
                self.nodes[index].expanded = true;
                let children: Vec<RemoteNode> = entries
                    .into_iter()
                    .take(MAX_ENTRIES_PER_DIR)
                    .map(|(name, is_dir)| {
                        let path = format!("{}/{}", dir_path.trim_end_matches('/'), name);
                        RemoteNode {
                            path,
                            name,
                            is_dir,
                            depth,
                            expanded: false,
                        }
                    })
                    .collect();
                let insert_pos = index + 1;
                self.nodes.splice(insert_pos..insert_pos, children);
            }
            Err(e) => {
                self.loading_error = Some(e);
            }
        }
    }

    pub fn collapse(&mut self, index: usize) {
        if index >= self.nodes.len() || !self.nodes[index].is_dir || !self.nodes[index].expanded {
            return;
        }
        self.nodes[index].expanded = false;
        let depth = self.nodes[index].depth;
        let mut remove_end = index + 1;
        while remove_end < self.nodes.len() && self.nodes[remove_end].depth > depth {
            remove_end += 1;
        }
        self.nodes.drain((index + 1)..remove_end);
    }

    fn go_to_parent(&mut self, index: usize) {
        if index >= self.nodes.len() {
            return;
        }
        let depth = self.nodes[index].depth;
        if depth == 0 {
            if !self.nodes.is_empty() && self.nodes[0].name == ".." {
                let parent = self.nodes[0].path.clone();
                self.load_root(&parent);
            }
            return;
        }
        for i in (0..index).rev() {
            if self.nodes[i].depth == depth - 1 && self.nodes[i].is_dir {
                self.cursor = i;
                break;
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> RemoteTreeAction {
        if self.jump_mode {
            return self.handle_jump_key(key);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                RemoteTreeAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < self.nodes.len() {
                    self.cursor += 1;
                }
                RemoteTreeAction::None
            }
            KeyCode::Right | KeyCode::Enter => {
                if self.cursor < self.nodes.len() && self.nodes[self.cursor].is_dir {
                    if self.nodes[self.cursor].expanded {
                        if self.cursor + 1 < self.nodes.len()
                            && self.nodes[self.cursor + 1].depth > self.nodes[self.cursor].depth
                        {
                            self.cursor += 1;
                        }
                    } else {
                        self.expand(self.cursor);
                    }
                }
                RemoteTreeAction::None
            }
            KeyCode::Left => {
                if self.cursor < self.nodes.len() {
                    if self.nodes[self.cursor].is_dir && self.nodes[self.cursor].expanded {
                        self.collapse(self.cursor);
                    } else {
                        self.go_to_parent(self.cursor);
                    }
                }
                RemoteTreeAction::None
            }
            KeyCode::Backspace => {
                self.go_to_parent(self.cursor);
                RemoteTreeAction::None
            }
            KeyCode::Char(' ') => {
                if self.cursor < self.nodes.len() {
                    let node = &self.nodes[self.cursor];
                    if node.name == ".." {
                        return RemoteTreeAction::None;
                    }
                    let path = node.path.clone();
                    let is_dir = node.is_dir;
                    self.selected_path = Some(path.clone());
                    RemoteTreeAction::Selected { path, is_dir }
                } else {
                    RemoteTreeAction::None
                }
            }
            KeyCode::Char('h') => {
                self.show_hidden = !self.show_hidden;
                let root = self.current_root.clone();
                self.load_root(&root);
                RemoteTreeAction::None
            }
            KeyCode::Char('/') => {
                self.jump_mode = true;
                self.jump_input.clear();
                RemoteTreeAction::None
            }
            KeyCode::Char('g') => {
                self.cursor = 0;
                RemoteTreeAction::None
            }
            KeyCode::Char('G') => {
                if !self.nodes.is_empty() {
                    self.cursor = self.nodes.len() - 1;
                }
                RemoteTreeAction::None
            }
            _ => RemoteTreeAction::None,
        }
    }

    fn handle_jump_key(&mut self, key: KeyEvent) -> RemoteTreeAction {
        match key.code {
            KeyCode::Enter => {
                let path = self.jump_input.clone();
                self.jump_mode = false;
                self.jump_input.clear();
                self.load_root(&path);
                RemoteTreeAction::None
            }
            KeyCode::Esc => {
                self.jump_mode = false;
                self.jump_input.clear();
                RemoteTreeAction::None
            }
            KeyCode::Backspace => {
                self.jump_input.pop();
                RemoteTreeAction::None
            }
            KeyCode::Char(c) => {
                self.jump_input.push(c);
                RemoteTreeAction::None
            }
            _ => RemoteTreeAction::None,
        }
    }

    pub fn is_jump_mode(&self) -> bool {
        self.jump_mode
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            theme::TITLE_STYLE
        } else {
            theme::DIM_STYLE
        };

        let title = format!(" Remote: {} ", self.current_root);
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(title);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        if let Some(ref err) = self.loading_error {
            frame.render_widget(
                Paragraph::new(format!("  Error: {}", err)).style(theme::ERROR_STYLE),
                inner,
            );
            return;
        }

        let visible_height = inner.height as usize;
        if self.nodes.is_empty() {
            frame.render_widget(
                Paragraph::new("  (empty directory)").style(theme::DIM_STYLE),
                inner,
            );
            return;
        }

        let scroll = if self.cursor < self.scroll_offset {
            self.cursor
        } else if self.cursor >= self.scroll_offset + visible_height {
            self.cursor - visible_height + 1
        } else {
            self.scroll_offset
        };

        if self.jump_mode {
            let lines = vec![Line::from(vec![
                Span::styled("Go to: ", theme::HEADER_STYLE),
                Span::raw(&self.jump_input),
                Span::styled("_", theme::TITLE_STYLE),
            ])];
            frame.render_widget(Paragraph::new(lines), inner);
            return;
        }

        let lines: Vec<Line> = self
            .nodes
            .iter()
            .enumerate()
            .skip(scroll)
            .take(visible_height)
            .map(|(i, node)| {
                let indent = "  ".repeat(node.depth);
                let icon = if node.name == ".." {
                    "↑ "
                } else if node.is_dir {
                    if node.expanded { "▾ " } else { "▸ " }
                } else {
                    "  "
                };

                let is_selected = self.selected_path.as_deref() == Some(node.path.as_str());
                let is_cursor = i == self.cursor;

                let name_style = if is_cursor && focused {
                    theme::SELECTED_STYLE
                } else if is_selected {
                    theme::SUCCESS_STYLE
                } else if node.is_dir {
                    theme::TREE_DIR_STYLE
                } else {
                    theme::TREE_FILE_STYLE
                };

                Line::from(vec![
                    Span::styled(indent, theme::DIM_STYLE),
                    Span::styled(icon, if node.is_dir { theme::TREE_DIR_STYLE } else { theme::DIM_STYLE }),
                    Span::styled(&node.name, name_style),
                ])
            })
            .collect();

        frame.render_widget(Paragraph::new(lines), inner);
    }
}

pub enum RemoteTreeAction {
    None,
    Selected { path: String, is_dir: bool },
}

fn shell_escape(s: &str) -> String {
    // Simple quoting for remote shell
    format!("'{}'", s.replace('\'', "'\\''"))
}
