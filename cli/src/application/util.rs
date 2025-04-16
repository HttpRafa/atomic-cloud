use ratatui::style::{
    palette::tailwind::SLATE,
    Color, Modifier, Style,
};

pub mod list;

pub const HEADER_STYLE: Style = Style::new().fg(SLATE.c100).bg(SLATE.c900);
pub const NORMAL_ROW_BG: Color = SLATE.c950;
pub const ALT_ROW_BG_COLOR: Color = SLATE.c900;
pub const SELECTED_STYLE: Style = Style::new().bg(SLATE.c800).add_modifier(Modifier::BOLD);
pub const TEXT_FG_COLOR: Color = SLATE.c200;
