//! Data table component for Arniko dashboard kit.
//!
//! Provides a styled HTML table with header, striped rows, and compact mode.
//! Follows the collection-container constructor convention: `Table::new()` + `.add(row)`.

use crate::components::escape_html;
use crate::{Component, ComponentMetadata};

/// A row in a `Table`.
///
/// # Examples
///
/// ```rust,no_run
/// use arniko::TableRow;
///
/// let row = TableRow::new(vec!["prometheus", "Running", "42 MB"]);
/// ```
#[derive(Clone)]
pub struct TableRow {
    cells: Vec<String>,
    class: String,
}

impl TableRow {
    pub fn new(cells: Vec<&str>) -> Self {
        Self {
            cells: cells.into_iter().map(str::to_owned).collect(),
            class: String::new(),
        }
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }
}

/// A data table with optional headers, striped rows, and compact density.
///
/// # Examples
///
/// ```rust,no_run
/// use arniko::{Table, TableRow};
///
/// let table = Table::new()
///     .headers(vec!["Service", "Status", "Memory"])
///     .add(TableRow::new(vec!["prometheus", "Running", "42 MB"]))
///     .add(TableRow::new(vec!["grafana", "Running", "128 MB"]))
///     .striped(true);
///
/// let html = table.render();
/// ```
///
/// With the `reactive` feature:
///
/// ```rust,ignore
/// # #[cfg(feature = "reactive")] {
/// use arniko::{table_reactive, TableRow};
/// use arniko::reactive::Signal;
///
/// let rows = Signal::new(vec![
///     TableRow::new(vec!["prometheus", "Running"]),
/// ]);
/// let view = table_reactive(vec!["Service".into(), "Status".into()], rows);
/// # }
/// ```
pub struct Table {
    headers: Vec<String>,
    rows: Vec<TableRow>,
    class: String,
    striped: bool,
    compact: bool,
}

impl Table {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            class: String::new(),
            striped: false,
            compact: false,
        }
    }

    pub fn headers(mut self, hdrs: Vec<impl Into<String>>) -> Self {
        self.headers = hdrs.into_iter().map(Into::into).collect();
        self
    }

    pub fn add(mut self, row: TableRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn add_rows(mut self, rows: Vec<TableRow>) -> Self {
        self.rows.extend(rows);
        self
    }

    pub fn striped(mut self, s: bool) -> Self {
        self.striped = s;
        self
    }

    pub fn compact(mut self, c: bool) -> Self {
        self.compact = c;
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    fn table_classes(&self) -> String {
        let mut classes = vec!["arniko-table"];
        if self.striped {
            classes.push("arniko-table-striped");
        }
        if self.compact {
            classes.push("arniko-table-compact");
        }
        classes.join(" ")
    }

    fn render_header(&self) -> String {
        if self.headers.is_empty() {
            return String::new();
        }
        let ths: String = self
            .headers
            .iter()
            .map(|h| format!("<th>{}</th>", escape_html(h)))
            .collect();
        format!("<thead><tr>{}</tr></thead>", ths)
    }

    fn render_body(&self) -> String {
        if self.rows.is_empty() {
            let colspan = self.headers.len().max(1);
            return format!(
                r#"<tbody><tr><td colspan="{}" class="arniko-table-empty">No data</td></tr></tbody>"#,
                colspan
            );
        }
        let rows: String = self
            .rows
            .iter()
            .map(|row| {
                let tds: String = row
                    .cells
                    .iter()
                    .map(|c| format!("<td>{}</td>", escape_html(c)))
                    .collect();
                format!(
                    r#"<tr class="{}">{}</tr>"#,
                    escape_html(&row.class),
                    tds
                )
            })
            .collect();
        format!("<tbody>{}</tbody>", rows)
    }

    pub fn render(&self) -> String {
        format!(
            r#"<div class="arniko-table-wrap {}"><table class="{}" role="table">{}{}</table></div>"#,
            escape_html(&self.class),
            self.table_classes(),
            self.render_header(),
            self.render_body(),
        )
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for Table {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata {
            css_classes: vec!["arniko-table".to_string()],
            requires_gpu: false,
            capabilities: vec![],
        }
    }
}

// ── Reactive View ────────────────────────────────────────────────────────────

#[cfg(feature = "reactive")]
use crate::reactive::{ReactiveHtml, Signal, View};

/// Create a reactive table whose rows update when `rows_signal` changes.
/// Headers are static; only the body re-renders on signal change.
#[cfg(feature = "reactive")]
pub fn table_reactive(headers: Vec<String>, rows_signal: Signal<Vec<TableRow>>) -> Box<dyn View> {
    table_reactive_with(headers, rows_signal, |t| t)
}

/// Reactive table with a builder closure for full static configuration.
#[cfg(feature = "reactive")]
pub fn table_reactive_with<F>(
    headers: Vec<String>,
    rows_signal: Signal<Vec<TableRow>>,
    build: F,
) -> Box<dyn View>
where
    F: Fn(Table) -> Table + Send + Sync + 'static,
{
    let html = rows_signal.derive(move |rows| {
        let table = build(Table::new().headers(headers.clone()).add_rows(rows));
        table.render()
    });
    Box::new(ReactiveHtml::new(html))
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_empty() {
        let table = Table::new().headers(vec!["A", "B"]);
        let html = table.render();
        assert!(html.contains("arniko-table"));
        assert!(html.contains("No data"));
    }

    #[test]
    fn test_table_rows() {
        let table = Table::new()
            .headers(vec!["Name", "Status"])
            .add(TableRow::new(vec!["prometheus", "Running"]))
            .add(TableRow::new(vec!["grafana", "Stopped"]));
        let html = table.render();
        assert!(html.contains("prometheus"));
        assert!(html.contains("grafana"));
        assert!(html.contains("<th>Name</th>"));
    }

    #[test]
    fn test_table_striped() {
        let table = Table::new().striped(true);
        assert!(table.render().contains("arniko-table-striped"));
    }

    #[test]
    fn test_table_xss() {
        let table = Table::new()
            .headers(vec!["<script>alert(1)</script>"])
            .add(TableRow::new(vec!["<b>bold</b>"]));
        let html = table.render();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
