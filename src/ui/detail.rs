use crate::model::ConnectionStore;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::app::DetailAction;
use super::browse::render_help_popup;
use super::quick_actions::{ActionListResult, ActionListState, ParamFormResult, ParamFormState};
use super::theme;

enum Overlay {
    None,
    Help,
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

    #[allow(clippy::too_many_lines)]
    pub fn render(&mut self, frame: &mut Frame, store: &ConnectionStore) {
        let Some(conn) = store.find_by_id(&self.connection_id) else {
            frame.render_widget(
                Paragraph::new("Connection not found.").style(theme::ERROR_STYLE),
                frame.area(),
            );
            return;
        };

        let chunks = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Min(1),    // details
            Constraint::Length(3), // SSH command
            Constraint::Length(1), // help line 1
            Constraint::Length(1), // help line 2
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
                Span::raw(format!(" {group}")),
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
                Span::styled(format!("  -o {k}="), theme::HEADER_STYLE),
                Span::raw(v),
            ]));
        }

        let detail_block = Block::default().borders(Borders::ALL);
        frame.render_widget(Paragraph::new(lines).block(detail_block), chunks[1]);

        // SSH command
        let cmd_block = Block::default()
            .borders(Borders::ALL)
            .title(" SSH Command ");
        frame.render_widget(
            Paragraph::new(format!("  {}", conn.ssh_command())).block(cmd_block),
            chunks[2],
        );

        // Help lines
        if self.confirm_delete {
            frame.render_widget(
                Paragraph::new(" Really delete? y: yes | n: cancel").style(theme::ERROR_STYLE),
                chunks[3],
            );
        } else {
            frame.render_widget(
                Paragraph::new(" ESC: back | Enter: connect | e: edit | d: delete")
                    .style(theme::HINT_STYLE),
                chunks[3],
            );
            frame.render_widget(
                Paragraph::new(" a: actions | t: transfer | u: tunnels | K: key auth | ?: help")
                    .style(theme::HINT_STYLE),
                chunks[4],
            );
        }

        // Render overlay on top
        match &mut self.overlay {
            Overlay::None => {}
            Overlay::Help => {
                render_help_popup(
                    frame,
                    "Detail Keys",
                    &[
                        ("Enter", "Connect via SSH"),
                        ("e", "Edit connection"),
                        ("d", "Delete connection"),
                        ("a", "Quick actions"),
                        ("t", "File transfer"),
                        ("u", "Manage tunnels"),
                        ("K", "Setup key auth"),
                        ("ESC", "Back to list"),
                        ("?", "Toggle this help"),
                    ],
                );
            }
            Overlay::ActionList(state) => state.render(frame),
            Overlay::ParamForm(state) => state.render(frame),
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent, store: &ConnectionStore) -> DetailAction {
        // Delegate to overlay if active
        match &mut self.overlay {
            Overlay::Help => {
                self.overlay = Overlay::None;
                return DetailAction::None;
            }
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
            return if let KeyCode::Char('y') = key.code {
                DetailAction::Delete(self.connection_id.clone())
            } else {
                self.confirm_delete = false;
                DetailAction::None
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
            KeyCode::Char('u') => DetailAction::ManageTunnels(self.connection_id.clone()),
            KeyCode::Char('a') => {
                self.overlay = Overlay::ActionList(ActionListState::new());
                DetailAction::None
            }
            KeyCode::Char('?') => {
                self.overlay = Overlay::Help;
                DetailAction::None
            }
            _ => DetailAction::None,
        }
    }
}
