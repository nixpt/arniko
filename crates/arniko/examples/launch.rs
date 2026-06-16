//! End-to-end launch example: arniko components → bliss native window

use arniko::alert::AlertVariant;
use arniko::badge::BadgeVariant;
use arniko::button::ButtonVariant;
use arniko::{Alert, ArnikoApp, Badge, Button, Card};

fn main() {
    ArnikoApp::html()
        .style(
            r#"
            body {
                background: #050508;
                color: #f4f4f5;
                font-family: ui-sans-serif, system-ui, -apple-system, sans-serif;
                padding: 40px;
                display: flex;
                flex-direction: column;
                gap: 16px;
                max-width: 600px;
                margin: 0 auto;
            }
        "#,
        )
        .component(Alert::new("Arniko is running on Bliss.").variant(AlertVariant::Success))
        .component(
            Card::new()
                .title("Agent Dashboard")
                .body("Exosphere UI — native window via bliss."),
        )
        .component(Button::new("Launch Capsule").variant(ButtonVariant::Accent))
        .component(Badge::new("Online").variant(BadgeVariant::Success))
        .launch();
}
