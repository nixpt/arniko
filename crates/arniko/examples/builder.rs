//! ArnikoHtmlBuilder — full API demonstration
//!
//! This example shows every builder method and output format.
//! Run with: `cargo run --example builder`

use arniko::{
    ArnikoApp, Badge, BadgeVariant, Button, ButtonSize, ButtonVariant, Card,
};

fn main() {
    demo_minimal();
    demo_full_page();
    demo_link_buttons();
    demo_mixed_content();
    demo_multiple_styles();
    demo_opt_out_arniko();
}

/// Bare-minimum builder — just the skeleton.
fn demo_minimal() {
    let html = ArnikoApp::html()
        .include_arniko_styles(false)
        .render();

    println!("=== Minimal ===\n{}\n", html);
    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("<meta charset=\"utf-8\">"));
    assert!(html.contains("</html>"));
}

/// Full builder with every option.
fn demo_full_page() {
    let html = ArnikoApp::html()
        .title("Dashboard")
        .base_styles(true)
        .include_arniko_styles(true)
        .component(
            Card::new()
                .title("Status")
                .body("All systems operational"),
        )
        .component(
            Button::new("Refresh")
                .variant(ButtonVariant::Accent),
        )
        .component(
            Badge::new("Online")
                .variant(BadgeVariant::Success),
        )
        .style("body { padding: 32px; font-family: system-ui, sans-serif; }")
        .style("h1 { color: #a78bfa; }")
        .render();

    println!("=== Full Page ===\n{}\n", html);

    // Structure assertions
    assert!(html.contains("<title>Dashboard</title>"));
    assert!(html.contains("arniko-btn"));
    assert!(html.contains("arniko-card"));
    assert!(html.contains("arniko-badge"));
    assert!(html.contains("box-sizing: border-box"));
    assert!(html.contains("padding: 32px"));
    assert!(html.contains("color: #a78bfa"));
}

/// Link buttons via Button::link()
fn demo_link_buttons() {
    let html = ArnikoApp::html()
        .title("Links")
        .include_arniko_styles(true)
        .html_content("<h1>Navigation</h1>")
        .html_content(
            Button::link("Visit Docs", "/docs")
                .variant(ButtonVariant::Outline)
                .render(),
        )
        .html_content(
            Button::link("Cancel", "/cancel")
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .render(),
        )
        .html_content(
            Button::link("Delete", "/delete")
                .variant(ButtonVariant::Destructive)
                .class("danger-link")
                .render(),
        )
        .render();

    println!("=== Link Buttons ===\n{}\n", html);

    assert!(html.contains("<a href=\"/docs\""));
    assert!(html.contains("<a href=\"/cancel\""));
    assert!(html.contains("<a href=\"/delete\""));
    assert!(html.contains("arniko-btn-outline"));
    assert!(html.contains("arniko-btn-sm"));
    assert!(html.contains("arniko-btn-destructive"));
    assert!(html.contains("danger-link"));
}

/// Mix of .component() and .html_content()
fn demo_mixed_content() {
    let html = ArnikoApp::html()
        .title("Hybrid")
        .include_arniko_styles(false)
        .html_content("<header><h1>App Header</h1></header>")
        .component(
            Card::new()
                .title("Component Card")
                .body("This came from .component()"),
        )
        .html_content("<footer>Page Footer</footer>")
        .render();

    println!("=== Mixed Content ===\n{}\n", html);

    assert!(html.contains("<header>"));
    assert!(html.contains("Component Card"));
    assert!(html.contains("<footer>"));
}

/// Multiple .style() calls accumulate
fn demo_multiple_styles() {
    let html = ArnikoApp::html()
        .title("Styles")
        .include_arniko_styles(false)
        .base_styles(true)
        .style("body { background: #050508; }")
        .style("h1 { color: #00f2ff; }")
        .style("p { color: #a1a1aa; }")
        .html_content("<h1>Styled</h1><p>Multi-style page</p>")
        .render();

    println!("=== Multiple Styles ===\n{}\n", html);

    assert!(html.contains("background: #050508"));
    assert!(html.contains("color: #00f2ff"));
    assert!(html.contains("color: #a1a1aa"));
}

/// Opt out of arniko base styles
fn demo_opt_out_arniko() {
    let html = ArnikoApp::html()
        .title("No Arniko")
        .include_arniko_styles(false)
        .base_styles(true)
        .html_content("<p>This page has no arniko component styles.</p>")
        .render();

    println!("=== Opt Out Arniko ===\n{}\n", html);

    assert!(!html.contains("arniko-btn"));
    assert!(!html.contains("arniko-card"));
    assert!(html.contains("box-sizing: border-box"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_minimal() {
        let html = ArnikoApp::html()
            .include_arniko_styles(false)
            .render();
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<meta charset=\"utf-8\">"));
    }

    #[test]
    fn test_builder_with_title() {
        let html = ArnikoApp::html()
            .include_arniko_styles(false)
            .title("Test")
            .render();
        assert!(html.contains("<title>Test</title>"));
    }

    #[test]
    fn test_builder_with_arniko_styles() {
        let html = ArnikoApp::html()
            .title("With Arniko")
            .render(); // include_arniko_styles defaults to true
        assert!(html.contains("arniko-btn"));
    }

    #[test]
    fn test_builder_content_order() {
        let html = ArnikoApp::html()
            .include_arniko_styles(false)
            .html_content("<p>First</p>")
            .html_content("<p>Second</p>")
            .render();
        let body_start = html.find("<body>").unwrap();
        let body_end = html.find("</body>").unwrap();
        let body = &html[body_start..body_end];
        assert!(body.contains("<p>First</p>"));
        assert!(body.contains("<p>Second</p>"));
        assert!(body.find("<p>First</p>").unwrap() < body.find("<p>Second</p>").unwrap());
    }

    #[test]
    fn test_builder_escaping() {
        let html = ArnikoApp::html()
            .include_arniko_styles(false)
            .title("Foo & Bar <3")
            .render();
        assert!(html.contains("Foo &amp; Bar &lt;3"));
        assert!(!html.contains("<title>Foo & Bar <3"));
    }
}
