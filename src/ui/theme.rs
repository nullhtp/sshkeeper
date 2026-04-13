use ratatui::style::{Color, Modifier, Style};

pub const TITLE_STYLE: Style = Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD);
pub const SELECTED_STYLE: Style = Style::new().fg(Color::Black).bg(Color::Cyan);
pub const HEADER_STYLE: Style = Style::new().fg(Color::Yellow).add_modifier(Modifier::BOLD);
pub const GROUP_STYLE: Style = Style::new().fg(Color::Magenta).add_modifier(Modifier::BOLD);
pub const DIM_STYLE: Style = Style::new().fg(Color::DarkGray);
pub const HINT_STYLE: Style = Style::new().fg(Color::DarkGray);
pub const ERROR_STYLE: Style = Style::new().fg(Color::Red);
pub const SUCCESS_STYLE: Style = Style::new().fg(Color::Green);
pub const TREE_DIR_STYLE: Style = Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD);
pub const TREE_FILE_STYLE: Style = Style::new().fg(Color::White);
pub const ACTIVE_TAB_STYLE: Style = Style::new()
    .fg(Color::Black)
    .bg(Color::Cyan)
    .add_modifier(Modifier::BOLD);
pub const FIELD_LABEL_STYLE: Style = Style::new().fg(Color::Yellow);
pub const FIELD_VALUE_STYLE: Style = Style::new().fg(Color::White);
pub const TOGGLE_ON_STYLE: Style = Style::new().fg(Color::Green).add_modifier(Modifier::BOLD);
