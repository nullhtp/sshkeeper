use crate::model::ConnectionStore;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::app::DetailAction;
use super::quick_actions::{ActionListResult, ActionListState, ParamFormResult, ParamFormState};
use super::theme;

enum Overlay {
    None,
    ActionList(ActionListState),
    ParamForm(ParamFormState),
}

pub struct DetailState {
    pub connection_id: String,
    pub confirm_delete: bool,
    overlay: Overlay,
}

impl DetailState {
    pub fn new(id: String) -> Self {
        Self {
            connection_id: id,
            confirm_delete: false,
            overlay: Overlay::None,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, store: &ConnectionStore) {
        let conn = match store.find_by_id(&self.connection_id) {
            Some(c) => c,
            None => {
                frame.render_widget(
                    Paragraph::new("Connection not found.").style(theme::ERROR_STYLE),
                    frame.area(),
                );
                return;
            }
        };

        let chunks = Layout::vertical([
            Constraint::Length(1),  // title
            Constraint::Min(1),    // details
            Constraint::Length(3), // SSH command
            Constraint::Length(1), // help
        ])
        .split(frame.area());

        // Title
        frame.render_widget(
            Paragraph::new(format!(" Connection: {}", conn.name)).style(theme::TITLE_STYLE),
            chunks[0],
        );

        // Details
        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled("  Host: ", theme::HEADER_STYLE),
                Span::raw(&conn.host),
            ]),
            Line::from(vec![
                Span::styled("  Port: ", theme::HEADER_STYLE),
                Span::raw(conn.port.to_string()),
            ]),
        ];

        if let Some(ref user) = conn.user {
            lines.push(Line::from(vec![
                Span::styled("  User: ", theme::HEADER_STYLE),
                Span::raw(user),
            ]));
        }
        if let Some(ref key) = conn.identity_file {
            lines.push(Line::from(vec![
                Span::styled("  Key:  ", theme::HEADER_STYLE),
                Span::raw(key),
            ]));
        }
        if let Some(ref group) = conn.group {
            lines.push(Line::from(vec![
                Span::styled("  Group:", theme::HEADER_STYLE),
                Span::raw(format!(" {}", group)),
            ]));
        }
        if !conn.tags.is_empty() {
            lines.push(Line::from(vec![
                Span::styled("  Tags: ", theme::HEADER_STYLE),
                Span::raw(conn.tags.join(", ")),
            ]));
        }
        if let Some(ref jump) = conn.proxy_jump {
            lines.push(Line::from(vec![
                Span::styled("  Jump: ", theme::HEADER_STYLE),
                Span::raw(jump),
            ]));
        }
        for (k, v) in &conn.ssh_options {
            lines.push(Line::from(vec![
                Span::styled(format!("  -o {}=", k), theme::HEADER_STYLE),
                Span::raw(v),
            ]));
        }

        let detail_block = Block::default().borders(Borders::ALL);
        frame.render_widget(
            Paragraph::new(lines).block(detail_block),
            chunks[1],
        );

        // SSH command
        let cmd_block = Block::default()
            .borders(Borders::ALL)
            .title(" SSH Command ");
        frame.render_widget(
            Paragraph::new(format!("  {}", conn.ssh_command()))
                .block(cmd_block)
                .style(theme::DIM_STYLE),
            chunks[2],
        );

        // Help line
        let help = if self.confirm_delete {
            " Really delete? y: yes | n: cancel"
        } else {
            " ESC: back | Enter: connect | e: edit | d: delete | t: transfer | K: setup key auth | a: actions"
        };
        frame.render_widget(
            Paragraph::new(help).style(theme::HINT_STYLE),
            chunks[3],
        );

        // Render overlay on top
        match &mut self.overlay {
            Overlay::None => {}
            Overlay::ActionList(state) => state.render(frame),
            Overlay::ParamForm(state) => state.render(frame),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, store: &ConnectionStore) -> DetailAction {
        // Delegate to overlay if active
        match &mut self.overlay {
            Overlay::ActionList(state) => {
                return match state.handle_key(key) {
                    ActionListResult::None => DetailAction::None,
                    ActionListResult::Dismiss => {
                        self.overlay = Overlay::None;
                        DetailAction::None
                    }
                    ActionListResult::Selected(action) => {
                        if action.has_params() || action.confirm_message.is_some() {
                            // Open param form
                            if let Some(conn) = store.find_by_id(&self.connection_id) {
                                self.overlay =
                                    Overlay::ParamForm(ParamFormState::new(action, conn));
                            }
                            DetailAction::None
                        } else {
                            // Execute immediately
                            self.overlay = Overlay::None;
                            let cmd = action.build_command(&[]);
                            DetailAction::RunRemoteAction {
                                conn_id: self.connection_id.clone(),
                                command: cmd,
                            }
                        }
                    }
                };
            }
            Overlay::ParamForm(state) => {
                return match state.handle_key(key) {
                    ParamFormResult::None => DetailAction::None,
                    ParamFormResult::Cancel => {
                        self.overlay = Overlay::ActionList(ActionListState::new());
                        DetailAction::None
                    }
                    ParamFormResult::Execute(cmd) => {
                        self.overlay = Overlay::None;
                        DetailAction::RunRemoteAction {
                            conn_id: self.connection_id.clone(),
                            command: cmd,
                        }
                    }
                };
            }
            Overlay::None => {}
        }

        // Normal detail key handling
        if self.confirm_delete {
            return match key.code {
                KeyCode::Char('y') => DetailAction::Delete(self.connection_id.clone()),
                _ => {
                    self.confirm_delete = false;
                    DetailAction::None
                }
            };
        }

        match key.code {
            KeyCode::Esc => DetailAction::Back,
            KeyCode::Enter => DetailAction::Connect(self.connection_id.clone()),
            KeyCode::Char('e') => DetailAction::Edit(self.connection_id.clone()),
            KeyCode::Char('d') => {
                self.confirm_delete = true;
                DetailAction::None
            }
            KeyCode::Char('K') => DetailAction::SetupKeyAuth(self.connection_id.clone()),
            KeyCode::Char('t') => DetailAction::Transfer(self.connection_id.clone()),
            KeyCode::Char('a') => {
                self.overlay = Overlay::ActionList(ActionListState::new());
                DetailAction::None
            }
            _ => DetailAction::None,
        }
    }
}
