#![allow(clippy::manual_is_multiple_of)]

use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::theme::RatatuiThemeColors;

/// Style for a table header row.
pub fn table_header_style(theme: &RatatuiThemeColors) -> Style {
    Style::default()
        .fg(theme.accent)
        .bg(theme.surface)
        .bold()
}

/// Style for a table row with zebra striping.
pub fn table_row_style(theme: &RatatuiThemeColors, index: usize) -> Style {
    if index % 2 == 0 {
        Style::default().fg(theme.fg).bg(theme.bg)
    } else {
        Style::default()
            .fg(theme.fg)
            .bg(theme.surface)
    }
}

/// Style for a highlighted / selected table row.
pub fn table_row_highlight_style(theme: &RatatuiThemeColors) -> Style {
    Style::default()
        .fg(theme.fg)
        .bg(theme.highlight)
}

/// Build a status bar paragraph spanning the full width.
///
/// Renders `left` text on a surface-coloured bar.
/// An optional `right` label is placed alongside it.
pub fn status_bar<'a>(
    left: impl Into<Line<'a>>,
    right: Option<impl Into<Line<'a>>>,
    theme: &RatatuiThemeColors,
) -> Paragraph<'a> {
    let bar_style = Style::default()
        .fg(theme.dim)
        .bg(theme.surface);

    let left_line: Line = left.into();
    let line = if let Some(r) = right {
        let mut spans = left_line.spans;
        spans.push("  ".into());
        spans.extend(r.into().spans);
        Line::from(spans)
    } else {
        left_line
    };

    Paragraph::new(line)
        .style(bar_style)
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(theme.border)))
}
