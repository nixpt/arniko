//! Individual component rendering examples
//!
//! Shows how each Arniko component renders standalone.
//! For the builder API (ArnikoApp::html()), see `builder.rs`.

use arniko::{Alert, Badge, Button, Card, Component};

fn main() {
    demo_individual_components();
}

fn demo_individual_components() {
    println!("=== Individual Components ===\n");

    // Button with custom variant and size
    let button = Button::new("Test Button")
        .variant(arniko::button::ButtonVariant::Outline)
        .size(arniko::button::ButtonSize::Lg);
    println!("Button:\n{}\n", button.render());

    // Card with title, body, and custom class
    let card = Card::new()
        .title("Sample Card")
        .body("This is the body content")
        .class("custom-card");
    println!("Card:\n{}\n", card.render());

    // Alert with warning variant
    let alert = Alert::new("Warning message").variant(arniko::alert::AlertVariant::Warning);
    println!("Alert:\n{}\n", alert.render());

    // Badge with error variant
    let badge = Badge::new("New").variant(arniko::badge::BadgeVariant::Error);
    println!("Badge:\n{}\n", badge.render());

    // Component metadata
    let metadata = button.metadata();
    println!("Button metadata: {:?}\n", metadata);
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
}
