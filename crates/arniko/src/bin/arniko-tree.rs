//! `arniko-tree` — dump a document's DOM + resolved layout as JSON.
//!
//! Track 2 of `docs/RUNE_BORROW.md`. Reads HTML from a file, `--html`, or
//! stdin, lays it out at a viewport, and writes the JSON tree from
//! [`arniko::inspect`] to `--out` or stdout. It is the headless, agent-facing
//! counterpart to a running window: the same tree, with rects, as text.
//!
//! ```text
//! arniko-tree page.html --width 1024 --height 768 --out tree.json
//! arniko-tree --html '<div class="card">Hi</div>'
//! echo "<p>hi</p>" | arniko-tree
//! ```
//!
//! Enabled by the `inspect-cli` feature:
//! `cargo run -p arniko --features inspect-cli --bin arniko-tree -- --html '<b>x</b>'`

use std::io::Read;
use std::process::ExitCode;
use std::str::FromStr;
use std::sync::Arc;

use arniko::inspect::document_tree_string;
use bliss_dom::DocumentConfig;
use bliss_html::{HtmlDocument, HtmlProvider};
use bliss_traits::net::DummyNetProvider;
use bliss_traits::shell::{ColorScheme, Viewport};

const USAGE: &str = "\
arniko-tree — dump a document's DOM + resolved layout as JSON

USAGE:
    arniko-tree [FILE] [OPTIONS]

INPUT (first match wins):
    FILE                an HTML file to read
    --html <HTML>       inline HTML
    (stdin)             HTML piped in, when neither is given

OPTIONS:
    --width  <PX>       viewport width  (default 800)
    --height <PX>       viewport height (default 600)
    --scale  <FACTOR>   device scale    (default 1.0)
    --out    <FILE>     write JSON here (default stdout)
    -h, --help          show this help";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("arniko-tree: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut html_arg: Option<String> = None;
    let mut file: Option<String> = None;
    let mut width = 800_u32;
    let mut height = 600_u32;
    let mut scale = 1.0_f32;
    let mut out: Option<String> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--html" => html_arg = Some(args.next().ok_or("--html needs a value")?),
            "--width" => width = parse(args.next())?,
            "--height" => height = parse(args.next())?,
            "--scale" => scale = parse(args.next())?,
            "--out" => out = Some(args.next().ok_or("--out needs a value")?),
            "-h" | "--help" => {
                println!("{USAGE}");
                return Ok(());
            }
            other if other.starts_with('-') => {
                return Err(format!("unknown option `{other}`\n\n{USAGE}"));
            }
            other => file = Some(other.to_string()),
        }
    }

    let html = match (html_arg, file) {
        (Some(html), _) => html,
        (None, Some(path)) => std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?,
        (None, None) => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| format!("stdin: {e}"))?;
            buf
        }
    };

    let config = DocumentConfig {
        viewport: Some(Viewport::new(width, height, scale, ColorScheme::Light)),
        // `Some(vec![])` makes `from_html` append the user-agent stylesheet, so
        // the layout has block/flow defaults rather than a bare document.
        ua_stylesheets: Some(Vec::new()),
        net_provider: Some(Arc::new(DummyNetProvider)),
        html_parser_provider: Some(Arc::new(HtmlProvider)),
        ..Default::default()
    };

    let mut doc = HtmlDocument::from_html(&html, config);
    doc.resolve(0.0);
    let json = document_tree_string(&doc);

    match out {
        Some(path) => std::fs::write(&path, json).map_err(|e| format!("{path}: {e}"))?,
        None => println!("{json}"),
    }
    Ok(())
}

fn parse<T: FromStr>(value: Option<String>) -> Result<T, String> {
    let raw = value.ok_or("missing value")?;
    raw.parse().map_err(|_| format!("invalid value: {raw}"))
}
