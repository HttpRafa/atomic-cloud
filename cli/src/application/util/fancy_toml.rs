use ratatui::{
    style::{palette::tailwind::AMBER, Color, Style, Stylize},
    text::{Line, Span},
};

use super::{
    ERROR_SELECTED_COLOR, INFO_SELECTED_COLOR, OK_SELECTED_COLOR, TEXT_FG_COLOR,
    WARN_SELECTED_COLOR,
};

pub struct FancyToml;

impl FancyToml {
    pub fn to_lines(toml: &str) -> Vec<Line<'static>> {
        let mut lines = Vec::new();

        for line in toml.lines() {
            if line.is_empty() {
                lines.push(Line::default());
                continue;
            }

            if line.starts_with('[') {
                lines.push(Line::styled(
                    format!("● {}", line.replace(['[', ']'], "").to_uppercase()),
                    Style::default().bold().fg(OK_SELECTED_COLOR),
                ));
                continue;
            }

            if line.contains(" = ") {
                if let Some((key, value)) = line.split_once(" = ") {
                    let key = key.trim();
                    let value = value.trim();

                    lines.push(Line::from(vec![
                        Span::styled(
                            key.to_owned(),
                            Style::default().fg(Self::word_to_color(key)),
                        ),
                        Span::styled(" = ", Style::default()),
                        Span::styled(value.to_owned(), Style::default().fg(TEXT_FG_COLOR)),
                    ]));
                } else {
                    lines.push(Line::styled(
                        format!("{line} // Failed to format this line"),
                        Style::default().fg(ERROR_SELECTED_COLOR),
                    ));
                }
                continue;
            }

            lines.push(Line::styled(line.to_owned(), Style::default()));
        }

        lines
    }

    fn word_to_color(key: &str) -> Color {
        match key {
            "name" | "id" => WARN_SELECTED_COLOR,
            "group" | "node" | "plugin" | "ctrl_addr" | "nodes" => AMBER.c600,
            "state" | "ready" | "users" | "token" | "enabled" | "start_threshold"
            | "stop_empty" => OK_SELECTED_COLOR,
            "host" | "port" => INFO_SELECTED_COLOR,
            "memory" | "swap" | "cpu" | "io" | "disk" | "ports" => Color::Magenta,
            "img" | "max_players" | "settings" | "env" | "retention" => Color::Blue,
            "key" | "value" => Color::LightYellow,
            "max" | "min" | "prio" => Color::LightCyan,
            _ => TEXT_FG_COLOR,
        }
    }
}
