use crate::model::Connection;
use chrono::Utc;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use super::app::EditorAction;
use super::theme;

const FIELD_LABELS: &[&str] = &[
    "Name",
    "Host",
    "Port",
    "User",
    "Identity File",
    "Group",
    "Tags (comma-sep)",
    "Proxy Jump",
];
const FIELD_COUNT: usize = 8;

#[derive(Clone)]
pub enum EditorMode {
    Add,
    Edit,
}

pub struct EditorState {
    pub mode: EditorMode,
    fields: [Input; FIELD_COUNT],
    focused: usize,
    original_id: Option<String>,
    original_created: Option<chrono::DateTime<Utc>>,
}

impl EditorState {
    pub fn new_add() -> Self {
        let mut fields = std::array::from_fn(|_| Input::default());
        fields[2] = Input::default().with_value("22".into());
        Self {
            mode: EditorMode::Add,
            fields,
            focused: 0,
            original_id: None,
            original_created: None,
        }
    }

    pub fn new_edit(conn: Connection) -> Self {
        let fields = [
            Input::default().with_value(conn.name.clone()),
            Input::default().with_value(conn.host.clone()),
            Input::default().with_value(conn.port.to_string()),
            Input::default().with_value(conn.user.clone().unwrap_or_default()),
            Input::default().with_value(conn.identity_file.clone().unwrap_or_default()),
            Input::default().with_value(conn.group.clone().unwrap_or_default()),
            Input::default().with_value(conn.tags.join(", ")),
            Input::default().with_value(conn.proxy_jump.clone().unwrap_or_default()),
        ];
        Self {
            mode: EditorMode::Edit,
            fields,
            focused: 0,
            original_id: Some(conn.id),
            original_created: Some(conn.created_at),
        }
    }

    pub fn render(&self, frame: &mut Frame) {
        let title = match &self.mode {
            EditorMode::Add => " Add Connection ",
            EditorMode::Edit => " Edit Connection ",
        };

        let mut constraints: Vec<Constraint> = vec![Constraint::Length(1)]; // title
        for _ in 0..FIELD_COUNT {
            constraints.push(Constraint::Length(2)); // label + input
        }
        constraints.push(Constraint::Min(0)); // spacer
        constraints.push(Constraint::Length(1)); // help

        let chunks = Layout::vertical(constraints).split(frame.area());

        frame.render_widget(Paragraph::new(title).style(theme::TITLE_STYLE), chunks[0]);

        for (i, label) in FIELD_LABELS.iter().enumerate() {
            let style = if i == self.focused {
                theme::SELECTED_STYLE
            } else {
                ratatui::style::Style::default()
            };
            let block = Block::default().borders(Borders::NONE);
            let content = Line::from(vec![
                Span::styled(format!("  {label}: "), theme::HEADER_STYLE),
                Span::styled(self.fields[i].value(), style),
                if i == self.focused {
                    Span::styled("▏", style)
                } else {
                    Span::raw("")
                },
            ]);
            frame.render_widget(Paragraph::new(content).block(block), chunks[i + 1]);
        }

        let help = " Tab/↓: next | Shift+Tab/↑: prev | Enter: save | ESC: cancel";
        frame.render_widget(
            Paragraph::new(help).style(theme::HINT_STYLE),
            chunks[FIELD_COUNT + 2],
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> EditorAction {
        match key.code {
            KeyCode::Esc => EditorAction::Cancel,
            KeyCode::Enter => self.try_save(),
            KeyCode::Tab | KeyCode::Down => {
                self.focused = (self.focused + 1) % FIELD_COUNT;
                EditorAction::None
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focused = if self.focused == 0 {
                    FIELD_COUNT - 1
                } else {
                    self.focused - 1
                };
                EditorAction::None
            }
            _ => {
                self.fields[self.focused].handle_event(&Event::Key(key));
                EditorAction::None
            }
        }
    }

    fn try_save(&self) -> EditorAction {
        let name = self.fields[0].value().trim().to_string();
        let host = self.fields[1].value().trim().to_string();
        if name.is_empty() || host.is_empty() {
            return EditorAction::None; // require at minimum name + host
        }

        let port: u16 = self.fields[2].value().trim().parse().unwrap_or(22);
        let user = non_empty(self.fields[3].value());
        let identity_file = non_empty(self.fields[4].value());
        let group = non_empty(self.fields[5].value());
        let tags: Vec<String> = self.fields[6]
            .value()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let proxy_jump = non_empty(self.fields[7].value());

        let mut conn = Connection::new(name, host);
        conn.port = port;
        conn.user = user;
        conn.identity_file = identity_file;
        conn.group = group;
        conn.tags = tags;
        conn.proxy_jump = proxy_jump;

        // Preserve id and created_at for edits
        if let Some(ref id) = self.original_id {
            conn.id.clone_from(id);
        }
        if let Some(created) = self.original_created {
            conn.created_at = created;
        }

        EditorAction::Save(Box::new(conn))
    }
}

fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
