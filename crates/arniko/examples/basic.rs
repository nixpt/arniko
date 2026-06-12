//! Test example for Arniko UI framework
//!
//! This example demonstrates basic usage of Arniko components.

use arniko::{Alert, ArnikoApp, Badge, Button, Card, Component};

fn main() {
    // Test HTML generation
    let html = ArnikoApp::html()
        .component(Button::new("Click me").variant(arniko::button::ButtonVariant::Accent))
        .component(Card::new().title("Test Card").body("This is a test card"))
        .component(Alert::new("Success!").variant(arniko::alert::AlertVariant::Success))
        .component(Badge::new("Active").variant(arniko::badge::BadgeVariant::Success))
        .style("body { font-family: Arial, sans-serif; padding: 20px; }")
        .render();

    println!("Generated HTML:");
    println!("{}", html);

    // Test individual components
    test_individual_components();
}

fn test_individual_components() {
    println!("\n=== Testing Individual Components ===");

    // Test Button
    let button = Button::new("Test Button")
        .variant(arniko::button::ButtonVariant::Outline)
        .size(arniko::button::ButtonSize::Large);
    println!("Button HTML: {}", button.render());

    // Test Card
    let card = Card::new()
        .title("Sample Card")
        .body("This is the body content")
        .class("custom-card");
    println!("Card HTML: {}", card.render());

    // Test Alert
    let alert = Alert::new("Warning message").variant(arniko::alert::AlertVariant::Warning);
    println!("Alert HTML: {}", alert.render());

    // Test Badge
    let badge = Badge::new("New").variant(arniko::badge::BadgeVariant::Error);
    println!("Badge HTML: {}", badge.render());

    // Test component metadata
    let metadata = button.metadata();
    println!("Button metadata: {:?}", metadata);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_component() {
        let button = Button::new("Test").variant(arniko::button::ButtonVariant::Accent);
        let html = button.render();

        assert!(html.contains("arniko-btn"));
        assert!(html.contains("arniko-btn-accent"));
        assert!(html.contains("Test"));
    }

    #[test]
    fn test_card_component() {
        let card = Card::new().title("Test").body("Body");
        let html = card.render();

        assert!(html.contains("arniko-card"));
        assert!(html.contains("Test"));
        assert!(html.contains("Body"));
    }

    #[test]
    fn test_alert_component() {
        let alert = Alert::new("Test").variant(arniko::alert::AlertVariant::Info);
        let html = alert.render();

        assert!(html.contains("arniko-alert"));
        assert!(html.contains("arniko-alert-info"));
        assert!(html.contains("Test"));
    }

    #[test]
    fn test_badge_component() {
        let badge = Badge::new("Test").variant(arniko::badge::BadgeVariant::Success);
        let html = badge.render();

        assert!(html.contains("arniko-badge"));
        assert!(html.contains("arniko-badge-success"));
        assert!(html.contains("Test"));
    }

    #[test]
    fn test_arniko_app_html_builder() {
        let html = ArnikoApp::html()
            .component(Button::new("Click"))
            .component(Card::new().title("Card"))
            .render();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("arniko-btn"));
        assert!(html.contains("arniko-card"));
        assert!(html.contains("Click"));
        assert!(html.contains("Card"));
    }
}
