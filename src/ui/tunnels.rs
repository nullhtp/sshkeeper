use crate::model::tunnel::{Tunnel, TunnelType};
use crate::ssh::tunnel::TunnelManager;
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use super::theme;

// Compact form: only the essentials. Defaults handle the rest.
const FORM_FIELDS: &[FormField] = &[
    FormField {
        label: "Local Port",
        hint: "e.g. 5432",
    },
    FormField {
        label: "Remote Host",
        hint: "default: localhost",
    },
    FormField {
        label: "Remote Port",
        hint: "leave empty = same as local",
    },
    FormField {
        label: "Name",
        hint: "leave empty = auto",
    },
    FormField {
        label: "Type",
        hint: "l=local r=remote d=dynamic",
    },
    FormField {
        label: "Bind Address",
        hint: "default: 127.0.0.1",
    },
];

struct FormField {
    label: &'static str,
    hint: &'static str,
}

const FIELD_COUNT: usize = 6;
// Field indices
const F_LOCAL_PORT: usize = 0;
const F_REMOTE_HOST: usize = 1;
const F_REMOTE_PORT: usize = 2;
const F_NAME: usize = 3;
const F_TYPE: usize = 4;
const F_BIND_ADDR: usize = 5;

enum Mode {
    List,
    Add(FormState),
    Edit(FormState),
}

struct FormState {
    fields: [Input; FIELD_COUNT],
    focused: usize,
    editing_id: Option<String>,
}

impl FormState {
    fn new_empty() -> Self {
        let fields = std::array::from_fn(|_| Input::default());
        Self {
            fields,
            focused: 0,
            editing_id: None,
        }
    }

    fn from_tunnel(tunnel: &Tunnel) -> Self {
        let type_str = match tunnel.tunnel_type {
            TunnelType::Local => "",
            TunnelType::Remote => "r",
            TunnelType::Dynamic => "d",
        };
        let mut fields = std::array::from_fn(|_| Input::default());
        fields[F_LOCAL_PORT] = Input::default().with_value(tunnel.bind_port.to_string());
        fields[F_REMOTE_HOST] =
            Input::default().with_value(tunnel.remote_host.clone().unwrap_or_default());
        fields[F_REMOTE_PORT] = Input::default()
            .with_value(tunnel.remote_port.map_or(String::new(), |p| p.to_string()));
        fields[F_NAME] = Input::default().with_value(tunnel.name.clone());
        fields[F_TYPE] = Input::default().with_value(type_str.into());
        fields[F_BIND_ADDR] = Input::default().with_value(if tunnel.bind_address == "127.0.0.1" {
            String::new()
        } else {
            tunnel.bind_address.clone()
        });

        Self {
            fields,
            focused: 0,
            editing_id: Some(tunnel.id.clone()),
        }
    }

    fn parse_tunnel(&self) -> Option<Tunnel> {
        let bind_port: u16 = self.fields[F_LOCAL_PORT].value().trim().parse().ok()?;

        let tunnel_type = match self.fields[F_TYPE].value().trim().to_lowercase().as_str() {
            "r" | "remote" => TunnelType::Remote,
            "d" | "dynamic" => TunnelType::Dynamic,
            // Default: local (empty or "l" or "local")
            _ => TunnelType::Local,
        };

        let bind_address = {
            let v = self.fields[F_BIND_ADDR].value().trim().to_string();
            if v.is_empty() {
                "127.0.0.1".to_string()
            } else {
                v
            }
        };

        let remote_host = if tunnel_type == TunnelType::Dynamic {
            None
        } else {
            let v = self.fields[F_REMOTE_HOST].value().trim().to_string();
            Some(if v.is_empty() {
                "localhost".to_string()
            } else {
                v
            })
        };

        let remote_port: Option<u16> = if tunnel_type == TunnelType::Dynamic {
            None
        } else {
            let v = self.fields[F_REMOTE_PORT].value().trim();
            if v.is_empty() {
                Some(bind_port)
            } else {
                Some(v.parse().ok()?)
            }
        };

        // Auto-generate name if empty
        let name = {
            let v = self.fields[F_NAME].value().trim().to_string();
            if v.is_empty() {
                if tunnel_type == TunnelType::Dynamic {
                    format!("SOCKS :{bind_port}")
                } else {
                    let rh = remote_host.as_deref().unwrap_or("?");
                    let rp = remote_port.unwrap_or(bind_port);
                    format!("{rh}:{rp}")
                }
            } else {
                v
            }
        };

        let mut tunnel = Tunnel::new(name, tunnel_type, bind_port);
        tunnel.bind_address = bind_address;
        tunnel.remote_host = remote_host;
        tunnel.remote_port = remote_port;

        if let Some(ref id) = self.editing_id {
            tunnel.id.clone_from(id);
        }

        if tunnel.validate().is_err() {
            return None;
        }

        Some(tunnel)
    }
}

pub struct TunnelScreenState {
    pub connection_name: String,
    pub tunnels: Vec<Tunnel>,
    table_state: TableState,
    mode: Mode,
}

pub enum TunnelAction {
    None,
    Back,
    Start { tunnel_id: String },
    Stop { tunnel_id: String },
    Save { tunnels: Vec<Tunnel> },
}

impl TunnelScreenState {
    pub fn new(connection_name: String, tunnels: Vec<Tunnel>) -> Self {
        Self {
            connection_name,
            tunnels,
            table_state: TableState::default().with_selected(Some(0)),
            mode: Mode::List,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, tunnel_manager: &mut TunnelManager) {
        match &self.mode {
            Mode::List => self.render_list(frame, tunnel_manager),
            Mode::Add(form) | Mode::Edit(form) => self.render_form(frame, form),
        }
    }

    fn render_list(&mut self, frame: &mut Frame, tunnel_manager: &mut TunnelManager) {
        let chunks = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Min(1),    // table
            Constraint::Length(1), // help
        ])
        .split(frame.area());

        frame.render_widget(
            Paragraph::new(format!(" Tunnels: {}", self.connection_name)).style(theme::TITLE_STYLE),
            chunks[0],
        );

        if self.tunnels.is_empty() {
            let block = Block::default().borders(Borders::ALL).title(" Tunnels ");
            frame.render_widget(
                Paragraph::new("No tunnels configured.\n\nPress 'a' to add a tunnel.")
                    .block(block)
                    .style(theme::DIM_STYLE),
                chunks[1],
            );
        } else {
            let header = Row::new(vec![
                Cell::from("Status").style(theme::HEADER_STYLE),
                Cell::from("Name").style(theme::HEADER_STYLE),
                Cell::from("Type").style(theme::HEADER_STYLE),
                Cell::from("Bind").style(theme::HEADER_STYLE),
                Cell::from("Remote").style(theme::HEADER_STYLE),
            ]);

            let rows: Vec<Row> = self
                .tunnels
                .iter()
                .map(|t| {
                    let is_running = tunnel_manager.is_running(&t.id);
                    let status_cell = if is_running {
                        Cell::from(" ON").style(theme::TOGGLE_ON_STYLE)
                    } else {
                        Cell::from("OFF").style(theme::ERROR_STYLE)
                    };

                    Row::new(vec![
                        status_cell,
                        Cell::from(t.name.as_str()),
                        Cell::from(t.tunnel_type.label()),
                        Cell::from(format!("{}:{}", t.bind_address, t.bind_port)),
                        Cell::from(t.remote_target()),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [
                    Constraint::Length(4),
                    Constraint::Percentage(25),
                    Constraint::Length(5),
                    Constraint::Percentage(30),
                    Constraint::Percentage(30),
                ],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Tunnels ({}) ", self.tunnels.len())),
            )
            .row_highlight_style(theme::SELECTED_STYLE);

            frame.render_stateful_widget(table, chunks[1], &mut self.table_state);
        }

        let help = " ESC: back | Enter: start/stop | a: add | e: edit | d: delete";
        frame.render_widget(Paragraph::new(help).style(theme::HINT_STYLE), chunks[2]);
    }

    fn render_form(&self, frame: &mut Frame, form: &FormState) {
        let title = match &self.mode {
            Mode::Add(_) => " Add Tunnel ",
            Mode::Edit(_) => " Edit Tunnel ",
            Mode::List => unreachable!(),
        };

        let mut constraints: Vec<Constraint> = vec![Constraint::Length(1)]; // title
        for _ in 0..FIELD_COUNT {
            constraints.push(Constraint::Length(2));
        }
        constraints.push(Constraint::Min(0)); // spacer
        constraints.push(Constraint::Length(1)); // help

        let chunks = Layout::vertical(constraints).split(frame.area());

        frame.render_widget(Paragraph::new(title).style(theme::TITLE_STYLE), chunks[0]);

        for (i, field_def) in FORM_FIELDS.iter().enumerate() {
            let is_focused = i == form.focused;
            let style = if is_focused {
                theme::SELECTED_STYLE
            } else {
                ratatui::style::Style::default()
            };

            let value = form.fields[i].value();
            let show_hint = value.is_empty() && !is_focused;

            let block = Block::default().borders(Borders::NONE);
            let content = if show_hint {
                Line::from(vec![
                    Span::styled(format!("  {}: ", field_def.label), theme::HEADER_STYLE),
                    Span::styled(field_def.hint, theme::DIM_STYLE),
                ])
            } else {
                Line::from(vec![
                    Span::styled(format!("  {}: ", field_def.label), theme::HEADER_STYLE),
                    Span::styled(value, style),
                    if is_focused {
                        Span::styled("▏", style)
                    } else {
                        Span::raw("")
                    },
                ])
            };
            frame.render_widget(Paragraph::new(content).block(block), chunks[i + 1]);
        }

        let help = " Tab/↓: next | Shift+Tab/↑: prev | Enter: save | ESC: cancel — only port + host required";
        frame.render_widget(
            Paragraph::new(help).style(theme::HINT_STYLE),
            chunks[FIELD_COUNT + 2],
        );
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> TunnelAction {
        let in_form = !matches!(self.mode, Mode::List);
        if in_form {
            self.handle_form_key(key)
        } else {
            self.handle_list_key(key)
        }
    }

    fn handle_list_key(&mut self, key: KeyEvent) -> TunnelAction {
        match key.code {
            KeyCode::Esc => TunnelAction::Back,
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1);
                TunnelAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1);
                TunnelAction::None
            }
            KeyCode::Char('a') => {
                self.mode = Mode::Add(FormState::new_empty());
                TunnelAction::None
            }
            KeyCode::Char('e') => {
                if let Some(tunnel) = self.selected_tunnel() {
                    let tunnel = tunnel.clone();
                    self.mode = Mode::Edit(FormState::from_tunnel(&tunnel));
                }
                TunnelAction::None
            }
            KeyCode::Char('d') => {
                if let Some(idx) = self.table_state.selected() {
                    if idx < self.tunnels.len() {
                        let tunnel_id = self.tunnels[idx].id.clone();
                        self.tunnels.remove(idx);
                        if !self.tunnels.is_empty() && idx >= self.tunnels.len() {
                            self.table_state.select(Some(self.tunnels.len() - 1));
                        }
                        return TunnelAction::Stop { tunnel_id };
                    }
                }
                TunnelAction::None
            }
            KeyCode::Enter => {
                if let Some(tunnel) = self.selected_tunnel() {
                    let tunnel_id = tunnel.id.clone();
                    return TunnelAction::Start { tunnel_id };
                }
                TunnelAction::None
            }
            _ => TunnelAction::None,
        }
    }

    fn handle_form_key(&mut self, key: KeyEvent) -> TunnelAction {
        let form = match &mut self.mode {
            Mode::Add(f) | Mode::Edit(f) => f,
            Mode::List => unreachable!(),
        };

        match key.code {
            KeyCode::Esc => {
                self.mode = Mode::List;
                TunnelAction::None
            }
            KeyCode::Enter => {
                if let Some(tunnel) = form.parse_tunnel() {
                    let is_edit = matches!(self.mode, Mode::Edit(_));
                    if is_edit {
                        if let Some(pos) = self.tunnels.iter().position(|t| t.id == tunnel.id) {
                            self.tunnels[pos] = tunnel;
                        }
                    } else {
                        self.tunnels.push(tunnel);
                    }
                    self.mode = Mode::List;
                    return TunnelAction::Save {
                        tunnels: self.tunnels.clone(),
                    };
                }
                TunnelAction::None
            }
            KeyCode::Tab | KeyCode::Down => {
                form.focused = (form.focused + 1) % FIELD_COUNT;
                TunnelAction::None
            }
            KeyCode::BackTab | KeyCode::Up => {
                form.focused = if form.focused == 0 {
                    FIELD_COUNT - 1
                } else {
                    form.focused - 1
                };
                TunnelAction::None
            }
            _ => {
                form.fields[form.focused].handle_event(&Event::Key(key));
                TunnelAction::None
            }
        }
    }

    fn move_selection(&mut self, delta: i32) {
        if self.tunnels.is_empty() {
            return;
        }
        let max = self.tunnels.len() - 1;
        let current = self.table_state.selected().unwrap_or(0) as i32;
        let next = (current + delta).clamp(0, max as i32) as usize;
        self.table_state.select(Some(next));
    }

    fn selected_tunnel(&self) -> Option<&Tunnel> {
        self.table_state
            .selected()
            .and_then(|i| self.tunnels.get(i))
    }
}
