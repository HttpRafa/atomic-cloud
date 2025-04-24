use ratatui::style::{
    palette::tailwind::{CYAN, GREEN, RED, SLATE, YELLOW},
    Color, Modifier, Style,
};

pub mod area;
pub mod button;
pub mod center;
pub mod list;
pub mod status;

pub const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(SLATE.c900);
pub const NORMAL_ROW_BG: Color = SLATE.c950;
pub const ALT_ROW_BG_COLOR: Color = SLATE.c900;
pub const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
pub const TEXT_FG_COLOR: Color = SLATE.c200;

pub const OK_COLOR: Color = GREEN.c700;
pub const OK_SELECTED_COLOR: Color = GREEN.c500;
pub const WARN_COLOR: Color = YELLOW.c700;
pub const WARN_SELECTED_COLOR: Color = YELLOW.c500;
pub const INFO_COLOR: Color = CYAN.c700;
pub const INFO_SELECTED_COLOR: Color = CYAN.c500;
pub const ERROR_COLOR: Color = RED.c700;
pub const ERROR_SELECTED_COLOR: Color = RED.c500;
