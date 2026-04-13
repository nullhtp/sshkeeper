use crate::model::ConnectionStore;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState};
use ratatui::Frame;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use super::app::BrowseAction;
use super::theme;

pub struct BrowseState {
    pub table_state: TableState,
    pub search_input: Input,
    pub search_active: bool,
    pub grouped: bool,
    filtered_ids: Vec<String>,
}

impl BrowseState {
    pub fn new() -> Self {
        Self {
            table_state: TableState::default().with_selected(Some(0)),
            search_input: Input::default(),
            search_active: false,
            grouped: false,
            filtered_ids: Vec::new(),
        }
    }

    pub fn render(&mut self, frame: &mut Frame, store: &ConnectionStore, status: Option<&str>) {
        let chunks = Layout::vertical([
            Constraint::Length(1), // title
            Constraint::Min(1),   // list
            Constraint::Length(1), // status/search
            Constraint::Length(1), // help
        ])
        .split(frame.area());

        // Title
        frame.render_widget(
            Paragraph::new(" SSHKeeper").style(theme::TITLE_STYLE),
            chunks[0],
        );

        // Build filtered list
        let query = if self.search_active {
            self.search_input.value()
        } else {
            ""
        };
        let connections = store.search(query);
        self.filtered_ids = connections.iter().map(|c| c.id.clone()).collect();

        if connections.is_empty() {
            self.render_empty(frame, chunks[1], store.all().is_empty());
        } else if self.grouped {
            self.render_grouped(frame, chunks[1], store, &connections);
        } else {
            self.render_flat(frame, chunks[1], &connections);
        }

        // Status / search bar
        if self.search_active {
            let search_line = Line::from(vec![
                Span::styled("/", theme::HEADER_STYLE),
                Span::raw(self.search_input.value()),
            ]);
            frame.render_widget(Paragraph::new(search_line), chunks[2]);
        } else if let Some(msg) = status {
            frame.render_widget(
                Paragraph::new(format!(" {}", msg)).style(theme::SUCCESS_STYLE),
                chunks[2],
            );
        }

        // Help line
        let help = if self.search_active {
            " ESC: cancel search"
        } else {
            " q: quit | a: add | i: import | /: search | Tab: group | Enter: view"
        };
        frame.render_widget(
            Paragraph::new(help).style(theme::HINT_STYLE),
            chunks[3],
        );
    }

    fn render_empty(&self, frame: &mut Frame, area: Rect, truly_empty: bool) {
        let msg = if truly_empty {
            "No connections yet.\n\nPress 'a' to add a connection or 'i' to import from ~/.ssh/config"
        } else {
            "No matching connections."
        };
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Connections ");
        frame.render_widget(
            Paragraph::new(msg).block(block).style(theme::DIM_STYLE),
            area,
        );
    }

    fn render_flat(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        connections: &[&crate::model::Connection],
    ) {
        let header = Row::new(vec![
            Cell::from("Name").style(theme::HEADER_STYLE),
            Cell::from("Host").style(theme::HEADER_STYLE),
            Cell::from("Group").style(theme::HEADER_STYLE),
        ]);

        let rows: Vec<Row> = connections
            .iter()
            .map(|c| {
                Row::new(vec![
                    Cell::from(c.name.as_str()),
                    Cell::from(format!(
                        "{}{}",
                        c.host,
                        if c.port != 22 {
                            format!(":{}", c.port)
                        } else {
                            String::new()
                        }
                    )),
                    Cell::from(c.group.as_deref().unwrap_or("-")),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(35),
                Constraint::Percentage(40),
                Constraint::Percentage(25),
            ],
        )
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Connections ({}) ", connections.len())),
        )
        .row_highlight_style(theme::SELECTED_STYLE);

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn render_grouped(
        &mut self,
        frame: &mut Frame,
        area: Rect,
        store: &ConnectionStore,
        connections: &[&crate::model::Connection],
    ) {
        let groups = store.groups();
        let mut rows: Vec<Row> = Vec::new();
        let mut flat_ids: Vec<String> = Vec::new();

        for group in &groups {
            let group_name = group.as_deref().unwrap_or("(ungrouped)");
            rows.push(
                Row::new(vec![
                    Cell::from(format!("▸ {}", group_name)).style(theme::GROUP_STYLE),
                    Cell::from(""),
                    Cell::from(""),
                ])
            );
            flat_ids.push(String::new()); // group header placeholder

            let group_conns: Vec<&&crate::model::Connection> = connections
                .iter()
                .filter(|c| c.group == *group)
                .collect();

            for c in group_conns {
                rows.push(Row::new(vec![
                    Cell::from(format!("  {}", c.name)),
                    Cell::from(format!(
                        "{}{}",
                        c.host,
                        if c.port != 22 { format!(":{}", c.port) } else { String::new() }
                    )),
                    Cell::from(""),
                ]));
                flat_ids.push(c.id.clone());
            }
        }

        self.filtered_ids = flat_ids;

        let table = Table::new(
            rows,
            [
                Constraint::Percentage(35),
                Constraint::Percentage(40),
                Constraint::Percentage(25),
            ],
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" Connections — Grouped ({}) ", connections.len())),
        )
        .row_highlight_style(theme::SELECTED_STYLE);

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    pub fn handle_key(&mut self, key: KeyEvent, store: &ConnectionStore) -> BrowseAction {
        if self.search_active {
            return self.handle_search_key(key);
        }

        match key.code {
            KeyCode::Char('q') => BrowseAction::Quit,
            KeyCode::Char('j') | KeyCode::Down => {
                self.move_selection(1, store);
                BrowseAction::None
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.move_selection(-1, store);
                BrowseAction::None
            }
            KeyCode::Char('g') => {
                self.table_state.select(Some(0));
                BrowseAction::None
            }
            KeyCode::Char('G') => {
                if !self.filtered_ids.is_empty() {
                    self.table_state.select(Some(self.filtered_ids.len() - 1));
                }
                BrowseAction::None
            }
            KeyCode::Char('/') => {
                self.search_active = true;
                self.search_input.reset();
                BrowseAction::None
            }
            KeyCode::Tab => {
                self.grouped = !self.grouped;
                self.table_state.select(Some(0));
                BrowseAction::None
            }
            KeyCode::Char('a') => BrowseAction::AddNew,
            KeyCode::Char('i') => BrowseAction::Import,
            KeyCode::Enter => {
                if let Some(id) = self.selected_id() {
                    if !id.is_empty() {
                        return BrowseAction::ViewDetail(id);
                    }
                }
                BrowseAction::None
            }
            _ => BrowseAction::None,
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> BrowseAction {
        match key.code {
            KeyCode::Esc => {
                self.search_active = false;
                self.search_input.reset();
                self.table_state.select(Some(0));
                BrowseAction::None
            }
            KeyCode::Enter => {
                self.search_active = false;
                BrowseAction::None
            }
            _ => {
                self.search_input.handle_event(&Event::Key(key));
                self.table_state.select(Some(0));
                BrowseAction::None
            }
        }
    }

    fn move_selection(&mut self, delta: i32, _store: &ConnectionStore) {
        let max = if self.filtered_ids.is_empty() {
            0
        } else {
            self.filtered_ids.len() - 1
        };
        let current = self.table_state.selected().unwrap_or(0) as i32;
        let mut next = (current + delta).clamp(0, max as i32) as usize;

        // In grouped view, skip group headers (empty ids)
        if self.grouped && next < self.filtered_ids.len() && self.filtered_ids[next].is_empty() {
            let try_next = (next as i32 + delta).clamp(0, max as i32) as usize;
            if try_next < self.filtered_ids.len() && !self.filtered_ids[try_next].is_empty() {
                next = try_next;
            }
        }

        self.table_state.select(Some(next));
    }

    fn selected_id(&self) -> Option<String> {
        self.table_state
            .selected()
            .and_then(|i| self.filtered_ids.get(i).cloned())
    }
}

use crossterm::event::Event;
