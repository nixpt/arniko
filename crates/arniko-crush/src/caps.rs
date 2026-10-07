//! Host capabilities that render Arniko UI components to HTML.
//!
//! Each capability returns an HTML string that the capsule can print, store,
//! or compose into a larger document. The full Arniko stylesheet is available
//! via `arniko.styles`.

use crush_lang_sdk::Value;
use crush_lang_sdk::{HostCap, HostCapSpec, HostCaps};

/// Register all Arniko UI capabilities on the given [`HostCaps`] registry.
pub fn register(caps: &mut HostCaps) {
    caps.register(Box::new(ArnikoStylesCap));
    caps.register(Box::new(ArnikoButtonCap));
    caps.register(Box::new(ArnikoCardCap));
    caps.register(Box::new(ArnikoAlertCap));
    caps.register(Box::new(ArnikoBadgeCap));
    caps.register(Box::new(ArnikoInputCap));
    caps.register(Box::new(ArnikoSeparatorCap));
    caps.register(Box::new(ArnikoSpinnerCap));
    caps.register(Box::new(ArnikoKbdCap));
    caps.register(Box::new(ArnikoStatusBadgeCap));
    caps.register(Box::new(ArnikoEmptyStateCap));
    caps.register(Box::new(ArnikoPanelCap));
    caps.register(Box::new(ArnikoToastCap));
    caps.register(Box::new(ArnikoMetricCardCap));
    caps.register(Box::new(ArnikoProgressBarCap));
    caps.register(Box::new(ArnikoTooltipCap));
    caps.register(Box::new(ArnikoGlassCardCap));
}

fn text_arg(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::Int(i) => i.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Str(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => a.borrow().iter().map(text_arg).collect::<Vec<_>>().join(", "),
        // Maps, errors, bytes and the collection variants added in crush-lang-sdk 0.3
        // (tuples, lists, vectors, …) have no text form for a UI argument.
        _ => String::new(),
    }
}

fn parse_variant<T: Default + Clone>(s: &str, variants: &[(&str, T)]) -> T {
    let lower = s.to_lowercase();
    for (name, v) in variants {
        if *name == lower {
            return v.clone();
        }
    }
    T::default()
}

pub struct ArnikoStylesCap;

impl HostCap for ArnikoStylesCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.styles".to_string(),
            argc: Some(0),
            returns: true,
        }
    }

    fn call(&self, _args: Vec<Value>) -> Result<Option<Value>, String> {
        Ok(Some(Value::Str(arniko::ARNIKO_STYLES.to_string())))
    }
}

pub struct ArnikoButtonCap;

impl HostCap for ArnikoButtonCap {
    fn spec(&self) -> HostCapSpec {
        // label, [variant], [size]
        HostCapSpec {
            name: "arniko.button".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.is_empty() {
            return Err("arniko.button requires at least a label".to_string());
        }
        let label = text_arg(&args[0]);
        let variant = args
            .get(1)
            .map(text_arg)
            .as_deref()
            .map(parse_button_variant)
            .unwrap_or_default();
        let size = args
            .get(2)
            .map(text_arg)
            .as_deref()
            .map(parse_button_size)
            .unwrap_or_default();

        use arniko::{Button, ButtonSize};
        let mut button = Button::new(&label).variant(variant);
        button = match size {
            ButtonSize::Default => button,
            _ => button.size(size),
        };
        Ok(Some(Value::Str(button.render())))
    }
}

fn parse_button_variant(s: &str) -> arniko::ButtonVariant {
    parse_variant(
        s,
        &[
            ("default", arniko::ButtonVariant::Default),
            ("destructive", arniko::ButtonVariant::Destructive),
            ("outline", arniko::ButtonVariant::Outline),
            ("secondary", arniko::ButtonVariant::Secondary),
            ("ghost", arniko::ButtonVariant::Ghost),
            ("accent", arniko::ButtonVariant::Accent),
        ],
    )
}

fn parse_button_size(s: &str) -> arniko::ButtonSize {
    parse_variant(
        s,
        &[
            ("default", arniko::ButtonSize::Default),
            ("sm", arniko::ButtonSize::Sm),
            ("lg", arniko::ButtonSize::Lg),
            ("icon", arniko::ButtonSize::Icon),
        ],
    )
}

pub struct ArnikoCardCap;

impl HostCap for ArnikoCardCap {
    fn spec(&self) -> HostCapSpec {
        // [title], [body]
        HostCapSpec {
            name: "arniko.card".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        let title = args.get(0).map(text_arg).filter(|s| !s.is_empty());
        let body = args.get(1).map(text_arg).filter(|s| !s.is_empty());

        let mut card = arniko::Card::new();
        if let Some(t) = title {
            card = card.title(&t);
        }
        if let Some(b) = body {
            card = card.body(&b);
        }
        Ok(Some(Value::Str(card.render())))
    }
}

pub struct ArnikoAlertCap;

impl HostCap for ArnikoAlertCap {
    fn spec(&self) -> HostCapSpec {
        // message, [variant]
        HostCapSpec {
            name: "arniko.alert".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.is_empty() {
            return Err("arniko.alert requires at least a message".to_string());
        }
        let message = text_arg(&args[0]);
        let variant = args
            .get(1)
            .map(text_arg)
            .as_deref()
            .map(parse_alert_variant)
            .unwrap_or_default();

        let alert = arniko::Alert::new(&message).variant(variant);
        Ok(Some(Value::Str(alert.render())))
    }
}

fn parse_alert_variant(s: &str) -> arniko::AlertVariant {
    parse_variant(
        s,
        &[
            ("info", arniko::AlertVariant::Info),
            ("success", arniko::AlertVariant::Success),
            ("warning", arniko::AlertVariant::Warning),
            ("error", arniko::AlertVariant::Error),
        ],
    )
}

pub struct ArnikoBadgeCap;

impl HostCap for ArnikoBadgeCap {
    fn spec(&self) -> HostCapSpec {
        // text, [variant]
        HostCapSpec {
            name: "arniko.badge".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.is_empty() {
            return Err("arniko.badge requires at least text".to_string());
        }
        let text = text_arg(&args[0]);
        let variant = args
            .get(1)
            .map(text_arg)
            .as_deref()
            .map(parse_badge_variant)
            .unwrap_or_default();

        let badge = arniko::Badge::new(&text).variant(variant);
        Ok(Some(Value::Str(badge.render())))
    }
}

fn parse_badge_variant(s: &str) -> arniko::BadgeVariant {
    parse_variant(
        s,
        &[
            ("default", arniko::BadgeVariant::Default),
            ("success", arniko::BadgeVariant::Success),
            ("warning", arniko::BadgeVariant::Warning),
            ("error", arniko::BadgeVariant::Error),
            ("info", arniko::BadgeVariant::Info),
            ("purple", arniko::BadgeVariant::Purple),
        ],
    )
}

pub struct ArnikoInputCap;

impl HostCap for ArnikoInputCap {
    fn spec(&self) -> HostCapSpec {
        // [placeholder], [value], [type]
        HostCapSpec {
            name: "arniko.input".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        let placeholder = args.get(0).map(text_arg).unwrap_or_default();
        let value = args.get(1).map(text_arg).unwrap_or_default();
        let input_type = args
            .get(2)
            .map(text_arg)
            .unwrap_or_else(|| "text".to_string());

        let mut input = arniko::Input::new().placeholder(&placeholder).value(&value);
        if input_type != "text" {
            input = input.input_type(&input_type);
        }
        Ok(Some(Value::Str(input.render())))
    }
}

pub struct ArnikoSeparatorCap;

impl HostCap for ArnikoSeparatorCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.separator".to_string(),
            argc: Some(0),
            returns: true,
        }
    }

    fn call(&self, _args: Vec<Value>) -> Result<Option<Value>, String> {
        Ok(Some(Value::Str(arniko::Separator::new().render())))
    }
}

pub struct ArnikoSpinnerCap;

impl HostCap for ArnikoSpinnerCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.spinner".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        let size = args
            .get(0)
            .map(text_arg)
            .as_deref()
            .map(parse_spinner_size)
            .unwrap_or_default();
        let spinner = arniko::Spinner::new().size(size);
        Ok(Some(Value::Str(spinner.render())))
    }
}

fn parse_spinner_size(s: &str) -> arniko::SpinnerSize {
    parse_variant(
        s,
        &[
            ("default", arniko::SpinnerSize::Default),
            ("sm", arniko::SpinnerSize::Sm),
            ("lg", arniko::SpinnerSize::Lg),
        ],
    )
}

pub struct ArnikoKbdCap;

impl HostCap for ArnikoKbdCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.kbd".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        let key = args
            .get(0)
            .map(text_arg)
            .unwrap_or_else(|| "key".to_string());
        Ok(Some(Value::Str(arniko::Kbd::new(&key).render())))
    }
}

pub struct ArnikoStatusBadgeCap;

impl HostCap for ArnikoStatusBadgeCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.status_badge".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.is_empty() {
            return Err("arniko.status_badge requires a label".to_string());
        }
        let label = text_arg(&args[0]);
        let variant = args
            .get(1)
            .map(text_arg)
            .as_deref()
            .map(parse_status_variant)
            .unwrap_or_default();
        let pulse = args.get(2).map(text_arg).as_deref() == Some("true");

        let badge = arniko::StatusBadge::new(&label)
            .variant(variant)
            .pulse(pulse);
        Ok(Some(Value::Str(badge.render())))
    }
}

fn parse_status_variant(s: &str) -> arniko::StatusVariant {
    parse_variant(
        s,
        &[
            ("active", arniko::StatusVariant::Active),
            ("success", arniko::StatusVariant::Success),
            ("warning", arniko::StatusVariant::Warning),
            ("error", arniko::StatusVariant::Error),
            ("info", arniko::StatusVariant::Info),
            ("offline", arniko::StatusVariant::Offline),
        ],
    )
}

pub struct ArnikoEmptyStateCap;

impl HostCap for ArnikoEmptyStateCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.empty_state".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.is_empty() {
            return Err("arniko.empty_state requires a title".to_string());
        }
        let title = text_arg(&args[0]);
        let icon = args.get(1).map(text_arg).filter(|s| !s.is_empty());
        let description = args.get(2).map(text_arg).filter(|s| !s.is_empty());
        let action = args.get(3).map(text_arg).filter(|s| !s.is_empty());

        let mut es = arniko::EmptyState::new(&title);
        if let Some(i) = icon {
            es = es.icon(&i);
        }
        if let Some(d) = description {
            es = es.description(&d);
        }
        if let Some(a) = action {
            es = es.action(&a);
        }
        Ok(Some(Value::Str(es.render())))
    }
}

pub struct ArnikoPanelCap;

impl HostCap for ArnikoPanelCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.panel".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        let title = args.get(0).map(text_arg).filter(|s| !s.is_empty());
        let body = args.get(1).map(text_arg).filter(|s| !s.is_empty());
        let closable = args.get(2).map(text_arg).as_deref() == Some("true");

        let mut panel = arniko::Panel::new();
        if let Some(t) = title {
            panel = panel.title(&t);
        }
        if let Some(b) = body {
            panel = panel.body(&b);
        }
        panel = panel.closable(closable);
        Ok(Some(Value::Str(panel.render())))
    }
}

pub struct ArnikoToastCap;

impl HostCap for ArnikoToastCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.toast".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.is_empty() {
            return Err("arniko.toast requires a message".to_string());
        }
        let message = text_arg(&args[0]);
        let variant = args
            .get(1)
            .map(text_arg)
            .as_deref()
            .map(parse_toast_variant)
            .unwrap_or_default();

        let toast = arniko::Toast::new(&message).variant(variant);
        Ok(Some(Value::Str(toast.render())))
    }
}

fn parse_toast_variant(s: &str) -> arniko::ToastVariant {
    parse_variant(
        s,
        &[
            ("info", arniko::ToastVariant::Info),
            ("success", arniko::ToastVariant::Success),
            ("warning", arniko::ToastVariant::Warning),
            ("error", arniko::ToastVariant::Error),
        ],
    )
}

pub struct ArnikoMetricCardCap;

impl HostCap for ArnikoMetricCardCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.metric_card".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.len() < 2 {
            return Err("arniko.metric_card requires title and value".to_string());
        }
        let title = text_arg(&args[0]);
        let value = text_arg(&args[1]);
        let color = args
            .get(2)
            .map(text_arg)
            .as_deref()
            .map(parse_metric_color)
            .unwrap_or_default();

        let card = arniko::MetricCard::new(&title, &value).color(color);
        Ok(Some(Value::Str(card.render())))
    }
}

fn parse_metric_color(s: &str) -> arniko::MetricColor {
    parse_variant(
        s,
        &[
            ("blue", arniko::MetricColor::Blue),
            ("green", arniko::MetricColor::Green),
            ("purple", arniko::MetricColor::Purple),
            ("orange", arniko::MetricColor::Orange),
            ("red", arniko::MetricColor::Red),
            ("cyan", arniko::MetricColor::Cyan),
        ],
    )
}

pub struct ArnikoProgressBarCap;

impl HostCap for ArnikoProgressBarCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.progress_bar".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.is_empty() {
            return Err("arniko.progress_bar requires a value".to_string());
        }
        let value = text_arg(&args[0]).parse::<f32>().unwrap_or(0.0);
        let max = args
            .get(1)
            .map(|a| text_arg(a).parse::<f32>().unwrap_or(100.0))
            .unwrap_or(100.0);
        let color = args
            .get(2)
            .map(text_arg)
            .as_deref()
            .map(parse_progress_color)
            .unwrap_or_default();

        let bar = arniko::ProgressBar::new(value).max(max).color(color);
        Ok(Some(Value::Str(bar.render())))
    }
}

fn parse_progress_color(s: &str) -> arniko::ProgressColor {
    parse_variant(
        s,
        &[
            ("accent", arniko::ProgressColor::Accent),
            ("blue", arniko::ProgressColor::Blue),
            ("green", arniko::ProgressColor::Green),
            ("purple", arniko::ProgressColor::Purple),
            ("orange", arniko::ProgressColor::Orange),
            ("red", arniko::ProgressColor::Red),
        ],
    )
}

pub struct ArnikoTooltipCap;

impl HostCap for ArnikoTooltipCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.tooltip".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        if args.len() < 2 {
            return Err("arniko.tooltip requires text and tooltip content".to_string());
        }
        let text = text_arg(&args[0]);
        let tooltip = text_arg(&args[1]);
        let position = args
            .get(2)
            .map(text_arg)
            .as_deref()
            .map(parse_tooltip_position)
            .unwrap_or_default();

        let tt = arniko::Tooltip::new(&text, &tooltip).position(position);
        Ok(Some(Value::Str(tt.render())))
    }
}

fn parse_tooltip_position(s: &str) -> arniko::TooltipPosition {
    parse_variant(
        s,
        &[
            ("top", arniko::TooltipPosition::Top),
            ("bottom", arniko::TooltipPosition::Bottom),
            ("left", arniko::TooltipPosition::Left),
            ("right", arniko::TooltipPosition::Right),
        ],
    )
}

pub struct ArnikoGlassCardCap;

impl HostCap for ArnikoGlassCardCap {
    fn spec(&self) -> HostCapSpec {
        HostCapSpec {
            name: "arniko.glass_card".to_string(),
            argc: None,
            returns: true,
        }
    }

    fn call(&self, args: Vec<Value>) -> Result<Option<Value>, String> {
        let title = args.get(0).map(text_arg).filter(|s| !s.is_empty());
        let body = args.get(1).map(text_arg).filter(|s| !s.is_empty());

        let mut card = arniko::Card::new().class("arniko-glass");
        if let Some(t) = title {
            card = card.title(&t);
        }
        if let Some(b) = body {
            card = card.body(&b);
        }
        Ok(Some(Value::Str(card.render())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_renders_html() {
        let cap = ArnikoButtonCap;
        let html = cap.call(vec![Value::Str("OK".into())]).unwrap().unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-btn"));
        assert!(s.contains("OK"));
    }

    #[test]
    fn button_accepts_variant_and_size() {
        let cap = ArnikoButtonCap;
        let html = cap
            .call(vec![
                Value::Str("Save".into()),
                Value::Str("accent".into()),
                Value::Str("lg".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-btn-accent"));
        assert!(s.contains("arniko-btn-lg"));
    }

    #[test]
    fn card_renders_html() {
        let cap = ArnikoCardCap;
        let html = cap
            .call(vec![
                Value::Str("Title".into()),
                Value::Str("Body text".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-card"));
        assert!(s.contains("Title"));
        assert!(s.contains("Body text"));
    }

    #[test]
    fn alert_renders_html() {
        let cap = ArnikoAlertCap;
        let html = cap
            .call(vec![Value::Str("Danger!".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-alert"));
        assert!(s.contains("Danger!"));
    }

    #[test]
    fn alert_with_variant() {
        let cap = ArnikoAlertCap;
        let html = cap
            .call(vec![
                Value::Str("Success".into()),
                Value::Str("success".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-alert-success"));
    }

    #[test]
    fn badge_renders_html() {
        let cap = ArnikoBadgeCap;
        let html = cap.call(vec![Value::Str("New".into())]).unwrap().unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-badge"));
        assert!(s.contains("New"));
    }

    #[test]
    fn badge_with_variant() {
        let cap = ArnikoBadgeCap;
        let html = cap
            .call(vec![
                Value::Str("v1.0".into()),
                Value::Str("success".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-badge-success"));
    }

    #[test]
    fn input_renders_html() {
        let cap = ArnikoInputCap;
        let html = cap.call(vec![]).unwrap().unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-input"));
    }

    #[test]
    fn input_with_placeholder() {
        let cap = ArnikoInputCap;
        let html = cap
            .call(vec![Value::Str("Enter name".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("Enter name"));
    }

    #[test]
    fn styles_returns_css() {
        let cap = ArnikoStylesCap;
        let css = cap.call(vec![]).unwrap().unwrap();
        let s = text_arg(&css);
        assert!(s.contains("arniko"));
    }

    #[test]
    fn separator_renders_html() {
        let cap = ArnikoSeparatorCap;
        let html = cap.call(vec![]).unwrap().unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-separator"));
    }

    #[test]
    fn spinner_renders_html() {
        let cap = ArnikoSpinnerCap;
        let html = cap.call(vec![]).unwrap().unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-spinner"));
    }

    #[test]
    fn spinner_with_size() {
        let cap = ArnikoSpinnerCap;
        let html = cap.call(vec![Value::Str("lg".into())]).unwrap().unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-spinner-lg"));
    }

    #[test]
    fn kbd_renders_html() {
        let cap = ArnikoKbdCap;
        let html = cap
            .call(vec![Value::Str("Ctrl+C".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-kbd"));
        assert!(s.contains("Ctrl+C"));
    }

    #[test]
    fn status_badge_renders_html() {
        let cap = ArnikoStatusBadgeCap;
        let html = cap
            .call(vec![Value::Str("Online".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-status-badge"));
        assert!(s.contains("Online"));
    }

    #[test]
    fn status_badge_with_variant_and_pulse() {
        let cap = ArnikoStatusBadgeCap;
        let html = cap
            .call(vec![
                Value::Str("Live".into()),
                Value::Str("success".into()),
                Value::Str("true".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-status-success"));
        assert!(s.contains("arniko-status-pulse"));
    }

    #[test]
    fn empty_state_renders_html() {
        let cap = ArnikoEmptyStateCap;
        let html = cap
            .call(vec![Value::Str("No items".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-empty-state"));
        assert!(s.contains("No items"));
    }

    #[test]
    fn empty_state_with_icon_and_desc() {
        let cap = ArnikoEmptyStateCap;
        let html = cap
            .call(vec![
                Value::Str("Empty".into()),
                Value::Str("📦".into()),
                Value::Str("Nothing here".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("📦"));
        assert!(s.contains("Nothing here"));
    }

    #[test]
    fn panel_renders_html() {
        let cap = ArnikoPanelCap;
        let html = cap
            .call(vec![
                Value::Str("Settings".into()),
                Value::Str("Body content".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-panel"));
        assert!(s.contains("Settings"));
        assert!(s.contains("Body content"));
    }

    #[test]
    fn panel_closable() {
        let cap = ArnikoPanelCap;
        let html = cap
            .call(vec![
                Value::Str("Dialog".into()),
                Value::Str("".into()),
                Value::Str("true".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-panel-close"));
    }

    #[test]
    fn toast_renders_html() {
        let cap = ArnikoToastCap;
        let html = cap
            .call(vec![Value::Str("Saved!".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-toast"));
        assert!(s.contains("Saved!"));
    }

    #[test]
    fn toast_with_variant() {
        let cap = ArnikoToastCap;
        let html = cap
            .call(vec![Value::Str("Error".into()), Value::Str("error".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-toast-error"));
    }

    #[test]
    fn metric_card_renders_html() {
        let cap = ArnikoMetricCardCap;
        let html = cap
            .call(vec![Value::Str("CPU".into()), Value::Str("85%".into())])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-metric-card"));
        assert!(s.contains("CPU"));
        assert!(s.contains("85%"));
    }

    #[test]
    fn metric_card_with_color() {
        let cap = ArnikoMetricCardCap;
        let html = cap
            .call(vec![
                Value::Str("Memory".into()),
                Value::Str("8GB".into()),
                Value::Str("green".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-metric-green"));
    }

    #[test]
    fn progress_bar_renders_html() {
        let cap = ArnikoProgressBarCap;
        let html = cap.call(vec![Value::Str("75".into())]).unwrap().unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-progress"));
        assert!(s.contains("75%"));
    }

    #[test]
    fn progress_bar_with_color() {
        let cap = ArnikoProgressBarCap;
        let html = cap
            .call(vec![
                Value::Str("50".into()),
                Value::Str("100".into()),
                Value::Str("green".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-progress-green"));
    }

    #[test]
    fn tooltip_renders_html() {
        let cap = ArnikoTooltipCap;
        let html = cap
            .call(vec![
                Value::Str("Hover me".into()),
                Value::Str("Tooltip text".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-tooltip"));
        assert!(s.contains("Hover me"));
        assert!(s.contains("Tooltip text"));
    }

    #[test]
    fn tooltip_with_position() {
        let cap = ArnikoTooltipCap;
        let html = cap
            .call(vec![
                Value::Str("Text".into()),
                Value::Str("Tip".into()),
                Value::Str("bottom".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("data-tooltip-position=\"bottom\""));
    }

    #[test]
    fn glass_card_renders_html() {
        let cap = ArnikoGlassCardCap;
        let html = cap
            .call(vec![
                Value::Str("Glass".into()),
                Value::Str("Content".into()),
            ])
            .unwrap()
            .unwrap();
        let s = text_arg(&html);
        assert!(s.contains("arniko-glass"));
        assert!(s.contains("Glass"));
        assert!(s.contains("Content"));
    }
}
