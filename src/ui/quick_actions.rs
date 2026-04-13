use crate::model::Connection;
use crate::ssh::actions::{
    ActionCategory, ActionParam, ParamType, QuickAction, build_actions, build_ssh_command,
};
use crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use std::sync::mpsc;
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use super::theme;

// ── Return types ──

pub enum ActionListResult {
    None,
    Dismiss,
    Selected(QuickAction),
}

pub enum ParamFormResult {
    None,
    Cancel,
    Execute(String), // final remote command
}

// ── Action List (the popup menu) ──

enum ListEntry {
    CategoryHeader(ActionCategory),
    Action(usize), // index into actions vec
}

pub struct ActionListState {
    actions: Vec<QuickAction>,
    search: Input,
    // Filtered view
    filtered_entries: Vec<ListEntry>,
    filtered_selectable: Vec<usize>, // indices into filtered_entries that are actions
    selected: usize,                 // index into filtered_selectable
    list_state: ListState,
}

impl ActionListState {
    pub fn new() -> Self {
        let actions = build_actions();
        let mut state = Self {
            actions,
            search: Input::default(),
            filtered_entries: Vec::new(),
            filtered_selectable: Vec::new(),
            selected: 0,
            list_state: ListState::default(),
        };
        state.rebuild_filtered();
        state
    }

    fn rebuild_filtered(&mut self) {
        let query = self.search.value().to_lowercase();
        self.filtered_entries.clear();
        self.filtered_selectable.clear();

        let mut last_cat: Option<ActionCategory> = None;
        // Collect matching action indices first, grouped by category
        let mut cat_actions: Vec<(ActionCategory, usize)> = Vec::new();
        for (i, action) in self.actions.iter().enumerate() {
            if query.is_empty()
                || action.name.to_lowercase().contains(&query)
                || action.description.to_lowercase().contains(&query)
                || action.category.label().to_lowercase().contains(&query)
            {
                cat_actions.push((action.category, i));
            }
        }

        for (cat, action_idx) in &cat_actions {
            if last_cat != Some(*cat) {
                self.filtered_entries.push(ListEntry::CategoryHeader(*cat));
                last_cat = Some(*cat);
            }
            self.filtered_selectable.push(self.filtered_entries.len());
            self.filtered_entries.push(ListEntry::Action(*action_idx));
        }

        // Reset selection
        self.selected = 0;
        if self.filtered_selectable.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(self.filtered_selectable[0]));
        }
    }

    pub fn render(&mut self, frame: &mut Frame) {
        let area = centered_rect(60, 80, frame.area());
        frame.render_widget(Clear, area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Quick Actions ");
        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Split inner into: search bar (2 lines) + list
        let chunks = Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(inner);

        // Search bar
        let search_line = Line::from(vec![
            Span::styled("  / ", theme::HEADER_STYLE),
            Span::styled(self.search.value(), theme::FIELD_VALUE_STYLE),
            Span::styled("▏", theme::FIELD_VALUE_STYLE),
        ]);
        let search_hint = Line::from(Span::styled(
            if self.search.value().is_empty() {
                "  Type to filter..."
            } else {
                ""
            },
            theme::DIM_STYLE,
        ));
        frame.render_widget(Paragraph::new(vec![search_line, search_hint]), chunks[0]);

        // Action list
        let items: Vec<ListItem> = self
            .filtered_entries
            .iter()
            .map(|entry| match entry {
                ListEntry::CategoryHeader(cat) => ListItem::new(Line::from(Span::styled(
                    format!("  {} ", cat.label()),
                    theme::GROUP_STYLE,
                ))),
                ListEntry::Action(idx) => {
                    let a = &self.actions[*idx];
                    ListItem::new(Line::from(vec![
                        Span::raw("    "),
                        Span::styled(a.name, theme::FIELD_VALUE_STYLE),
                        Span::styled(format!("  {}", a.description), theme::DIM_STYLE),
                    ]))
                }
            })
            .collect();

        let list = List::new(items).highlight_style(theme::SELECTED_STYLE);
        frame.render_stateful_widget(list, chunks[1], &mut self.list_state);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> ActionListResult {
        match key.code {
            KeyCode::Esc => {
                if self.search.value().is_empty() {
                    ActionListResult::Dismiss
                } else {
                    self.search = Input::default();
                    self.rebuild_filtered();
                    ActionListResult::None
                }
            }
            KeyCode::Down => {
                if self.selected + 1 < self.filtered_selectable.len() {
                    self.selected += 1;
                    self.list_state
                        .select(Some(self.filtered_selectable[self.selected]));
                }
                ActionListResult::None
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    self.list_state
                        .select(Some(self.filtered_selectable[self.selected]));
                }
                ActionListResult::None
            }
            KeyCode::Enter => {
                if let Some(&entry_idx) = self.filtered_selectable.get(self.selected) {
                    if let ListEntry::Action(action_idx) = &self.filtered_entries[entry_idx] {
                        return ActionListResult::Selected(self.actions[*action_idx].clone());
                    }
                }
                ActionListResult::None
            }
            _ => {
                // All other keys go to search input
                self.search.handle_event(&Event::Key(key));
                self.rebuild_filtered();
                ActionListResult::None
            }
        }
    }
}

// ── Parameter Form ──

enum SelectState {
    Loading(mpsc::Receiver<Vec<String>>),
    Loaded(Vec<String>),
    Error(String),
}

struct FieldState {
    param: ActionParam,
    kind: FieldKind,
}

enum FieldKind {
    Text(Input),
    Select {
        state: SelectState,
        selected: usize,
        expanded: bool,
        search: Input,
        filtered: Vec<usize>, // indices into the options vec
    },
    Confirm(bool),
}

/// focused index: `0..fields.len()` = fields, `fields.len()` = submit button
pub struct ParamFormState {
    action: QuickAction,
    fields: Vec<FieldState>,
    focused: usize,
    confirm_message: Option<&'static str>,
    confirm_shown: bool,
    confirmed: bool,
}

impl ParamFormState {
    fn focus_count(&self) -> usize {
        self.fields.len() + 1 // fields + submit button
    }

    fn on_submit_button(&self) -> bool {
        self.focused == self.fields.len()
    }
}

impl ParamFormState {
    pub fn new(action: QuickAction, conn: &Connection) -> Self {
        let fields: Vec<FieldState> = action
            .params
            .iter()
            .map(|p| {
                let kind = match &p.param_type {
                    ParamType::Text { default } => {
                        FieldKind::Text(Input::default().with_value((*default).into()))
                    }
                    ParamType::Select { fetch_command } => {
                        let (tx, rx) = mpsc::channel();
                        let mut cmd = build_ssh_command(conn, fetch_command);
                        std::thread::spawn(move || match cmd.output() {
                            Ok(output) if output.status.success() => {
                                let lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
                                    .lines()
                                    .filter(|l| !l.trim().is_empty())
                                    .map(|l| l.trim().to_string())
                                    .collect();
                                let _ = tx.send(lines);
                            }
                            Ok(output) => {
                                let err = String::from_utf8_lossy(&output.stderr).to_string();
                                let _ = tx.send(vec![format!("ERROR: {}", err)]);
                            }
                            Err(e) => {
                                let _ = tx.send(vec![format!("ERROR: {}", e)]);
                            }
                        });
                        FieldKind::Select {
                            state: SelectState::Loading(rx),
                            selected: 0,
                            expanded: false,
                            search: Input::default(),
                            filtered: Vec::new(),
                        }
                    }
                    ParamType::Confirm => FieldKind::Confirm(false),
                };
                FieldState {
                    param: p.clone(),
                    kind,
                }
            })
            .collect();

        let confirm_message = action.confirm_message;
        Self {
            action,
            fields,
            focused: 0,
            confirm_message,
            confirm_shown: false,
            confirmed: false,
        }
    }

    /// Poll select fields for loaded data. Call this each render cycle.
    pub fn poll_selects(&mut self) {
        for field in &mut self.fields {
            if let FieldKind::Select {
                state, filtered, ..
            } = &mut field.kind
            {
                if let SelectState::Loading(rx) = state {
                    if let Ok(options) = rx.try_recv() {
                        if options.len() == 1 && options[0].starts_with("ERROR:") {
                            *state = SelectState::Error(options[0].clone());
                        } else {
                            *filtered = (0..options.len()).collect();
                            *state = SelectState::Loaded(options);
                        }
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn render(&mut self, frame: &mut Frame) {
        self.poll_selects();

        let full_area = frame.area();

        // If we're showing confirm dialog
        if self.confirm_shown {
            let popup = centered_rect(50, 20, full_area);
            frame.render_widget(Clear, popup);
            let msg = self.confirm_message.unwrap_or("Are you sure?");
            let block = Block::default().borders(Borders::ALL).title(" Confirm ");
            let text = Paragraph::new(vec![
                Line::from(""),
                Line::from(Span::styled(msg, theme::ERROR_STYLE)),
                Line::from(""),
                Line::from(Span::styled("  y: yes  |  n: cancel", theme::HINT_STYLE)),
            ])
            .block(block);
            frame.render_widget(text, popup);
            return;
        }

        // Calculate popup height based on fields
        let mut content_height: u16 = 2; // title + blank line
        for field in &self.fields {
            match &field.kind {
                FieldKind::Select {
                    expanded: true,
                    filtered,
                    ..
                } => {
                    content_height += (filtered.len() as u16).min(10) + 3;
                }
                _ => content_height += 2,
            }
        }
        content_height += 2; // submit button row
        content_height += 1; // help line
        content_height += 2; // border top + bottom

        // Clamp popup size
        let popup_height = content_height
            .min(full_area.height.saturating_sub(4))
            .max(8);
        let popup_width = (full_area.width * 70 / 100)
            .max(40)
            .min(full_area.width.saturating_sub(4));

        let popup = Rect {
            x: (full_area.width.saturating_sub(popup_width)) / 2,
            y: (full_area.height.saturating_sub(popup_height)) / 2,
            width: popup_width,
            height: popup_height,
        };

        frame.render_widget(Clear, popup);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!(" {} ", self.action.name));

        let inner = block.inner(popup);
        frame.render_widget(block, popup);

        // Layout inside the popup
        let mut constraints = vec![Constraint::Length(1)]; // title description
        for field in &self.fields {
            match &field.kind {
                FieldKind::Select {
                    expanded: true,
                    filtered,
                    ..
                } => {
                    constraints.push(Constraint::Length((filtered.len() as u16).min(10) + 3));
                }
                _ => constraints.push(Constraint::Length(2)),
            }
        }
        constraints.push(Constraint::Length(2)); // submit button
        constraints.push(Constraint::Min(0)); // spacer
        constraints.push(Constraint::Length(1)); // help

        let chunks = Layout::vertical(constraints).split(inner);

        // Description line
        frame.render_widget(
            Paragraph::new(format!(" {}", self.action.description)).style(theme::DIM_STYLE),
            chunks[0],
        );

        // Fields
        for (i, field) in self.fields.iter().enumerate() {
            let is_focused = i == self.focused;
            let chunk = chunks[i + 1];

            match &field.kind {
                FieldKind::Text(input) => {
                    let style = if is_focused {
                        theme::SELECTED_STYLE
                    } else {
                        ratatui::style::Style::default()
                    };
                    let content = Line::from(vec![
                        Span::styled(format!("  {}: ", field.param.label), theme::HEADER_STYLE),
                        Span::styled(input.value(), style),
                        if is_focused {
                            Span::styled("▏", style)
                        } else {
                            Span::raw("")
                        },
                    ]);
                    frame.render_widget(Paragraph::new(content), chunk);
                }
                FieldKind::Select {
                    state,
                    selected,
                    expanded,
                    search,
                    filtered,
                } => {
                    let label =
                        Span::styled(format!("  {}: ", field.param.label), theme::HEADER_STYLE);
                    match state {
                        SelectState::Loading(_) => {
                            let content = Line::from(vec![
                                label,
                                Span::styled("Loading...", theme::DIM_STYLE),
                            ]);
                            frame.render_widget(Paragraph::new(content), chunk);
                        }
                        SelectState::Error(err) => {
                            let content = Line::from(vec![
                                label,
                                Span::styled(err.as_str(), theme::ERROR_STYLE),
                            ]);
                            frame.render_widget(Paragraph::new(content), chunk);
                        }
                        SelectState::Loaded(options) => {
                            if *expanded {
                                // Search bar + filtered list
                                let search_line = Line::from(vec![
                                    Span::styled("  / ", theme::HEADER_STYLE),
                                    Span::styled(search.value(), theme::FIELD_VALUE_STYLE),
                                    Span::styled("▏ ", theme::FIELD_VALUE_STYLE),
                                    Span::styled(
                                        format!("{}/{}", filtered.len(), options.len()),
                                        theme::DIM_STYLE,
                                    ),
                                ]);
                                let mut lines = vec![search_line];
                                let visible_count = filtered.len().min(10);
                                let start = if *selected >= visible_count {
                                    selected - visible_count + 1
                                } else {
                                    0
                                };
                                for (j, &opt_idx) in
                                    filtered.iter().enumerate().skip(start).take(visible_count)
                                {
                                    let style = if j == *selected {
                                        theme::SELECTED_STYLE
                                    } else {
                                        ratatui::style::Style::default()
                                    };
                                    lines.push(Line::from(Span::styled(
                                        format!("    {}", &options[opt_idx]),
                                        style,
                                    )));
                                }
                                if filtered.len() > visible_count {
                                    lines.push(Line::from(Span::styled(
                                        format!("    ... {} more", filtered.len() - visible_count),
                                        theme::DIM_STYLE,
                                    )));
                                }
                                frame.render_widget(Paragraph::new(lines), chunk);
                            } else {
                                // Collapsed: show selected value from filtered list
                                let display_val = filtered
                                    .get(*selected)
                                    .and_then(|&idx| options.get(idx))
                                    .map_or("(none)", std::string::String::as_str);
                                let style = if is_focused {
                                    theme::SELECTED_STYLE
                                } else {
                                    ratatui::style::Style::default()
                                };
                                let content = Line::from(vec![
                                    label,
                                    Span::styled(display_val, style),
                                    Span::styled(
                                        if is_focused {
                                            " ▼ (Enter to expand)"
                                        } else {
                                            ""
                                        },
                                        theme::DIM_STYLE,
                                    ),
                                ]);
                                frame.render_widget(Paragraph::new(content), chunk);
                            }
                        }
                    }
                }
                FieldKind::Confirm(val) => {
                    let style = if is_focused {
                        theme::SELECTED_STYLE
                    } else {
                        ratatui::style::Style::default()
                    };
                    let val_str = if *val { "Yes" } else { "No" };
                    let content = Line::from(vec![
                        Span::styled(format!("  {}: ", field.param.label), theme::HEADER_STYLE),
                        Span::styled(val_str, style),
                        if is_focused {
                            Span::styled(" (y/n to toggle)", theme::DIM_STYLE)
                        } else {
                            Span::raw("")
                        },
                    ]);
                    frame.render_widget(Paragraph::new(content), chunk);
                }
            }
        }

        // Submit button
        let button_chunk = chunks[self.fields.len() + 1];
        let button_style = if self.on_submit_button() {
            theme::SELECTED_STYLE
        } else {
            theme::DIM_STYLE
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::raw("  "),
                Span::styled("  Execute  ", button_style),
            ])),
            button_chunk,
        );

        // Help line
        let help = " Tab/↓: next | ↑: prev | Enter: interact/execute | Esc: back";
        frame.render_widget(
            Paragraph::new(help).style(theme::HINT_STYLE),
            chunks[chunks.len() - 1],
        );
    }

    #[allow(clippy::too_many_lines)]
    pub fn handle_key(&mut self, key: KeyEvent) -> ParamFormResult {
        // Confirm dialog handling
        if self.confirm_shown {
            return if let KeyCode::Char('y') = key.code {
                self.confirmed = true;
                self.confirm_shown = false;
                self.try_submit()
            } else {
                self.confirm_shown = false;
                ParamFormResult::None
            };
        }

        // If a select is expanded, delegate to it
        if let Some(field) = self.fields.get_mut(self.focused) {
            if let FieldKind::Select {
                state: SelectState::Loaded(options),
                selected,
                expanded,
                search,
                filtered,
            } = &mut field.kind
            {
                if *expanded {
                    match key.code {
                        KeyCode::Down => {
                            if *selected + 1 < filtered.len() {
                                *selected += 1;
                            }
                            return ParamFormResult::None;
                        }
                        KeyCode::Up => {
                            if *selected > 0 {
                                *selected -= 1;
                            }
                            return ParamFormResult::None;
                        }
                        KeyCode::Enter => {
                            *expanded = false;
                            return ParamFormResult::None;
                        }
                        KeyCode::Esc => {
                            if search.value().is_empty() {
                                *expanded = false;
                            } else {
                                *search = Input::default();
                                *filtered = (0..options.len()).collect();
                                *selected = 0;
                            }
                            return ParamFormResult::None;
                        }
                        _ => {
                            // Type to search
                            search.handle_event(&Event::Key(key));
                            let query = search.value().to_lowercase();
                            *filtered = options
                                .iter()
                                .enumerate()
                                .filter(|(_, opt)| {
                                    query.is_empty() || opt.to_lowercase().contains(&query)
                                })
                                .map(|(i, _)| i)
                                .collect();
                            *selected = 0;
                            return ParamFormResult::None;
                        }
                    }
                }
            }
        }

        match key.code {
            KeyCode::Esc => ParamFormResult::Cancel,
            KeyCode::Tab | KeyCode::Down => {
                self.focused = (self.focused + 1) % self.focus_count();
                ParamFormResult::None
            }
            KeyCode::BackTab | KeyCode::Up => {
                self.focused = if self.focused == 0 {
                    self.focus_count() - 1
                } else {
                    self.focused - 1
                };
                ParamFormResult::None
            }
            KeyCode::Enter => {
                // On submit button -> submit
                if self.on_submit_button() {
                    return self.try_submit();
                }
                // On a select field -> expand it
                if let Some(field) = self.fields.get_mut(self.focused) {
                    if let FieldKind::Select {
                        state: SelectState::Loaded(_),
                        expanded,
                        ..
                    } = &mut field.kind
                    {
                        *expanded = true;
                        return ParamFormResult::None;
                    }
                }
                // On other fields -> move to next
                self.focused = (self.focused + 1) % self.focus_count();
                ParamFormResult::None
            }
            _ => {
                // Delegate to focused field
                if let Some(field) = self.fields.get_mut(self.focused) {
                    match &mut field.kind {
                        FieldKind::Text(input) => {
                            input.handle_event(&Event::Key(key));
                        }
                        FieldKind::Confirm(val) => match key.code {
                            KeyCode::Char('y') => *val = true,
                            KeyCode::Char('n') => *val = false,
                            _ => {}
                        },
                        FieldKind::Select { .. } => {}
                    }
                }
                ParamFormResult::None
            }
        }
    }

    fn try_submit(&mut self) -> ParamFormResult {
        // Check confirm fields are confirmed
        for field in &self.fields {
            if let FieldKind::Confirm(val) = &field.kind {
                if !val {
                    return ParamFormResult::None;
                }
            }
        }

        // Check if action has a confirm_message and we haven't confirmed yet
        if self.confirm_message.is_some() && !self.confirmed {
            self.confirm_shown = true;
            return ParamFormResult::None;
        }

        // Build values
        let mut values: Vec<(String, String)> = Vec::new();
        for field in &self.fields {
            let val = match &field.kind {
                FieldKind::Text(input) => input.value().to_string(),
                FieldKind::Select {
                    state: SelectState::Loaded(options),
                    selected,
                    filtered,
                    ..
                } => filtered
                    .get(*selected)
                    .and_then(|&idx| options.get(idx))
                    .cloned()
                    .unwrap_or_default(),
                FieldKind::Confirm(v) => if *v { "yes" } else { "no" }.to_string(),
                FieldKind::Select { .. } => String::new(),
            };
            values.push((field.param.key.to_string(), val));
        }

        let cmd = self.action.build_command(&values);
        ParamFormResult::Execute(cmd)
    }
}

// ── Helpers ──

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
