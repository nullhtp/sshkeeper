use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;
use std::fs;
use std::path::{Path, PathBuf};

use super::theme;

const MAX_ENTRIES_PER_DIR: usize = 200;

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
    pub children_loaded: bool,
}

pub struct FileTree {
    pub nodes: Vec<TreeNode>,
    pub cursor: usize,
    pub selected_path: Option<PathBuf>,
    pub show_hidden: bool,
    pub scroll_offset: usize,
    jump_mode: bool,
    jump_input: String,
}

impl FileTree {
    pub fn new(root: &Path) -> Self {
        let mut tree = Self {
            nodes: Vec::new(),
            cursor: 0,
            selected_path: None,
            show_hidden: false,
            scroll_offset: 0,
            jump_mode: false,
            jump_input: String::new(),
        };
        tree.load_root(root);
        tree
    }

    fn load_root(&mut self, root: &Path) {
        self.nodes.clear();
        self.cursor = 0;
        self.scroll_offset = 0;

        // Add parent entry if not at filesystem root
        if let Some(parent) = root.parent() {
            self.nodes.push(TreeNode {
                path: parent.to_path_buf(),
                name: "..".into(),
                is_dir: true,
                depth: 0,
                expanded: false,
                children_loaded: false,
            });
        }

        if let Ok(entries) = Self::read_dir_sorted(root, self.show_hidden) {
            for (path, name, is_dir) in entries.into_iter().take(MAX_ENTRIES_PER_DIR) {
                self.nodes.push(TreeNode {
                    path,
                    name,
                    is_dir,
                    depth: 0,
                    expanded: false,
                    children_loaded: false,
                });
            }
        }
    }

    fn read_dir_sorted(dir: &Path, show_hidden: bool) -> std::io::Result<Vec<(PathBuf, String, bool)>> {
        let mut entries: Vec<(PathBuf, String, bool)> = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && name.starts_with('.') {
                continue;
            }
            let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
            entries.push((entry.path(), name, is_dir));
        }
        // Dirs first, then alphabetical
        entries.sort_by(|a, b| {
            b.2.cmp(&a.2).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase()))
        });
        Ok(entries)
    }

    pub fn expand(&mut self, index: usize) {
        if index >= self.nodes.len() || !self.nodes[index].is_dir {
            return;
        }
        if self.nodes[index].name == ".." {
            // Navigate to parent
            let parent = self.nodes[index].path.clone();
            self.load_root(&parent);
            return;
        }
        if self.nodes[index].expanded {
            return;
        }

        self.nodes[index].expanded = true;
        self.nodes[index].children_loaded = true;

        let dir_path = self.nodes[index].path.clone();
        let depth = self.nodes[index].depth + 1;

        if let Ok(entries) = Self::read_dir_sorted(&dir_path, self.show_hidden) {
            let children: Vec<TreeNode> = entries
                .into_iter()
                .take(MAX_ENTRIES_PER_DIR)
                .map(|(path, name, is_dir)| TreeNode {
                    path,
                    name,
                    is_dir,
                    depth,
                    expanded: false,
                    children_loaded: false,
                })
                .collect();

            let insert_pos = index + 1;
            self.nodes.splice(insert_pos..insert_pos, children);
        }
    }

    pub fn collapse(&mut self, index: usize) {
        if index >= self.nodes.len() || !self.nodes[index].is_dir || !self.nodes[index].expanded {
            return;
        }
        self.nodes[index].expanded = false;
        let depth = self.nodes[index].depth;

        // Remove all children (nodes with depth > this node's depth, until we hit same or lower)
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
            // At root level — navigate to parent directory via ".." if present
            if !self.nodes.is_empty() && self.nodes[0].name == ".." {
                let parent = self.nodes[0].path.clone();
                self.load_root(&parent);
            }
            return;
        }
        // Find the parent node (closest node above with depth - 1)
        for i in (0..index).rev() {
            if self.nodes[i].depth == depth - 1 && self.nodes[i].is_dir {
                self.cursor = i;
                break;
            }
        }
    }

    fn jump_to_path(&mut self, path_str: &str) {
        let path = PathBuf::from(path_str);
        if path.is_dir() {
            self.load_root(&path);
        } else if let Some(parent) = path.parent() {
            if parent.is_dir() {
                self.load_root(parent);
                // Try to find and select the file
                let name = path.file_name().map(|n| n.to_string_lossy().to_string());
                if let Some(name) = name {
                    for (i, node) in self.nodes.iter().enumerate() {
                        if node.name == name {
                            self.cursor = i;
                            break;
                        }
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> FileTreeAction {
        if self.jump_mode {
            return self.handle_jump_key(key);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                FileTreeAction::None
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < self.nodes.len() {
                    self.cursor += 1;
                }
                FileTreeAction::None
            }
            KeyCode::Right | KeyCode::Enter => {
                if self.cursor < self.nodes.len() && self.nodes[self.cursor].is_dir {
                    if self.nodes[self.cursor].expanded {
                        // Enter on expanded dir: move to first child
                        if self.cursor + 1 < self.nodes.len()
                            && self.nodes[self.cursor + 1].depth > self.nodes[self.cursor].depth
                        {
                            self.cursor += 1;
                        }
                    } else {
                        self.expand(self.cursor);
                    }
                }
                FileTreeAction::None
            }
            KeyCode::Left => {
                if self.cursor < self.nodes.len() {
                    if self.nodes[self.cursor].is_dir && self.nodes[self.cursor].expanded {
                        self.collapse(self.cursor);
                    } else {
                        self.go_to_parent(self.cursor);
                    }
                }
                FileTreeAction::None
            }
            KeyCode::Backspace => {
                self.go_to_parent(self.cursor);
                FileTreeAction::None
            }
            KeyCode::Char(' ') => {
                if self.cursor < self.nodes.len() {
                    let node = &self.nodes[self.cursor];
                    if node.name == ".." {
                        return FileTreeAction::None;
                    }
                    let path = node.path.clone();
                    let is_dir = node.is_dir;
                    self.selected_path = Some(path.clone());
                    FileTreeAction::Selected { path, is_dir }
                } else {
                    FileTreeAction::None
                }
            }
            KeyCode::Char('h') => {
                self.show_hidden = !self.show_hidden;
                // Reload current root
                if let Some(root) = self.current_root() {
                    self.load_root(&root);
                }
                FileTreeAction::None
            }
            KeyCode::Char('/') => {
                self.jump_mode = true;
                self.jump_input.clear();
                FileTreeAction::None
            }
            KeyCode::Char('g') => {
                self.cursor = 0;
                FileTreeAction::None
            }
            KeyCode::Char('G') => {
                if !self.nodes.is_empty() {
                    self.cursor = self.nodes.len() - 1;
                }
                FileTreeAction::None
            }
            _ => FileTreeAction::None,
        }
    }

    fn handle_jump_key(&mut self, key: KeyEvent) -> FileTreeAction {
        match key.code {
            KeyCode::Enter => {
                let path = self.jump_input.clone();
                self.jump_mode = false;
                self.jump_input.clear();
                self.jump_to_path(&path);
                FileTreeAction::None
            }
            KeyCode::Esc => {
                self.jump_mode = false;
                self.jump_input.clear();
                FileTreeAction::None
            }
            KeyCode::Backspace => {
                self.jump_input.pop();
                FileTreeAction::None
            }
            KeyCode::Char(c) => {
                self.jump_input.push(c);
                FileTreeAction::None
            }
            _ => FileTreeAction::None,
        }
    }

    fn current_root(&self) -> Option<PathBuf> {
        // Find root by looking at the ".." entry or first entry's parent
        if let Some(first) = self.nodes.first() {
            if first.name == ".." {
                // The ".." points to parent, so the root is one level down
                return Some(first.path.clone());
            }
            return first.path.parent().map(|p| p.to_path_buf());
        }
        None
    }

    pub fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_style = if focused {
            theme::TITLE_STYLE
        } else {
            theme::DIM_STYLE
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(border_style)
            .title(" Files ");

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let visible_height = inner.height as usize;
        if self.nodes.is_empty() {
            frame.render_widget(
                Paragraph::new("  (empty directory)").style(theme::DIM_STYLE),
                inner,
            );
            return;
        }

        // Compute scroll offset to keep cursor visible
        let scroll = if self.cursor < self.scroll_offset {
            self.cursor
        } else if self.cursor >= self.scroll_offset + visible_height {
            self.cursor - visible_height + 1
        } else {
            self.scroll_offset
        };

        // Render jump input if active
        if self.jump_mode {
            let mut lines: Vec<Line> = Vec::new();
            lines.push(Line::from(vec![
                Span::styled("Go to: ", theme::HEADER_STYLE),
                Span::raw(&self.jump_input),
                Span::styled("_", theme::TITLE_STYLE),
            ]));
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
                    if node.expanded {
                        "▾ "
                    } else {
                        "▸ "
                    }
                } else {
                    "  "
                };

                let is_selected = self.selected_path.as_ref() == Some(&node.path);
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

    pub fn is_jump_mode(&self) -> bool {
        self.jump_mode
    }

    pub fn ensure_cursor_visible(&mut self, height: usize) {
        if height == 0 {
            return;
        }
        if self.cursor < self.scroll_offset {
            self.scroll_offset = self.cursor;
        } else if self.cursor >= self.scroll_offset + height {
            self.scroll_offset = self.cursor - height + 1;
        }
    }
}

pub enum FileTreeAction {
    None,
    Selected { path: PathBuf, is_dir: bool },
}
