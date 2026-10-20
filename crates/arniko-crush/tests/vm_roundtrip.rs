//! End to end: a Crush program calls the arniko capabilities through the
//! crush-lang-sdk compiler and VM, not just `HostCap::call` directly.

use crush_lang_sdk::{HostCaps, Runtime, Value, compile::compile_crush_source};

fn run(source: &str) -> crush_lang_sdk::VmResult {
    let mut caps = HostCaps::new();
    arniko_crush::register(&mut caps);
    let program = compile_crush_source(source).expect("compiles");
    Runtime::new()
        .with_host_caps(caps)
        .run(&program)
        .expect("runs")
}

#[test]
fn crush_program_renders_a_button() {
    let result = run(r#"
        fn main() {
            let html = arniko.button("Save", "accent", "lg");
            print(html);
            return html;
        }
    "#);
    let Some(Value::Str(html)) = result.return_value else {
        panic!("expected a string, got {:?}", result.return_value);
    };
    assert!(html.contains("arniko-btn-accent"), "{html}");
    assert!(html.contains("arniko-btn-lg"), "{html}");
    assert!(html.contains("Save"), "{html}");
    assert_eq!(result.output, format!("{html}\n"));
}

#[test]
fn crush_program_composes_a_card() {
    let result = run(r#"
        fn main() {
            let badge = arniko.badge("new");
            return arniko.card("Title", badge);
        }
    "#);
    let Some(Value::Str(html)) = result.return_value else {
        panic!("expected a string, got {:?}", result.return_value);
    };
    assert!(html.contains("Title"), "{html}");
    assert!(html.contains("new"), "{html}");
}
