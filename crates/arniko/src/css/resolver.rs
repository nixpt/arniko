/// CSS custom property (var()) resolver.
///
/// Strategy: string-level pre-pass before lightning-css parsing.
/// 1. Extract `:root { --name: value; }` definitions into a HashMap
/// 2. Substitute `var(--name)` and `var(--name, fallback)` references
/// 3. Handle nested var() references by iterating until stable
/// 4. Strip `:root` blocks from output
use std::collections::HashMap;

/// Extract all `:root { ... }` blocks, collecting custom property definitions.
/// Returns the definitions map and the CSS with `:root` blocks removed.
pub fn extract_root_variables(css: &str) -> (HashMap<String, String>, String) {
    let mut vars: HashMap<String, String> = HashMap::new();
    let mut output = String::with_capacity(css.len());
    let mut rest = css;

    while let Some(root_start) = find_root_block(rest) {
        // Append everything before this :root block
        output.push_str(&rest[..root_start]);

        // Find the opening brace
        let after_root = &rest[root_start..];
        let brace_offset = match after_root.find('{') {
            Some(o) => o,
            None => {
                output.push_str(after_root);
                rest = "";
                break;
            }
        };

        // Find matching closing brace
        let block_start = root_start + brace_offset;
        let close = match find_matching_brace(&rest[block_start..]) {
            Some(o) => block_start + o,
            None => {
                output.push_str(after_root);
                rest = "";
                break;
            }
        };

        // Parse the block contents for custom properties
        let block_contents = &rest[block_start + 1..close];
        parse_custom_properties(block_contents, &mut vars);

        rest = &rest[close + 1..];
    }

    output.push_str(rest);
    (vars, output)
}

/// Find the start index of a `:root` selector block.
fn find_root_block(css: &str) -> Option<usize> {
    let mut search_from = 0;
    while search_from < css.len() {
        let idx = css[search_from..].find(":root")?;
        let abs = search_from + idx;

        // Check it's not inside a selector like `.foo:root` — must be start of line or after whitespace/;/}
        let is_start = abs == 0 || {
            let prev = css.as_bytes()[abs - 1];
            prev == b'\n'
                || prev == b'\r'
                || prev == b' '
                || prev == b'\t'
                || prev == b'}'
                || prev == b';'
        };

        // Check what follows `:root` — should be whitespace or `{`
        let after = &css[abs + 5..];
        let next_is_valid = after.starts_with('{')
            || after.starts_with(' ')
            || after.starts_with('\t')
            || after.starts_with('\n')
            || after.starts_with('\r');

        if is_start && next_is_valid {
            return Some(abs);
        }

        search_from = abs + 5;
    }
    None
}

/// Find the matching `}` for an opening `{` at position 0 of `s`.
fn find_matching_brace(s: &str) -> Option<usize> {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Parse `--name: value;` declarations from a CSS block body.
fn parse_custom_properties(block: &str, vars: &mut HashMap<String, String>) {
    // Simple line-by-line parse for `--name: value;`
    for line in block.split(';') {
        let line = line.trim();
        if let Some(colon_pos) = line.find(':') {
            let name = line[..colon_pos].trim();
            if name.starts_with("--") {
                let value = line[colon_pos + 1..].trim().to_string();
                vars.insert(name.to_string(), value);
            }
        }
    }
}

/// Resolve all `var(--name)` and `var(--name, fallback)` references in the CSS string.
/// Iterates until no more substitutions are made (handles nested references).
pub fn resolve_variables(css: &str, vars: &HashMap<String, String>) -> String {
    let mut result = css.to_string();

    // Iterate to handle nested var() references (e.g., --a references --b)
    for _ in 0..10 {
        let next = substitute_vars(&result, vars);
        if next == result {
            break;
        }
        result = next;
    }

    result
}

/// Single pass of var() substitution.
fn substitute_vars(css: &str, vars: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(css.len());
    let mut rest = css;

    while let Some(var_pos) = rest.find("var(") {
        result.push_str(&rest[..var_pos]);

        let after_var = &rest[var_pos + 4..];
        // Find the matching closing paren, respecting nested parens
        match find_matching_paren(after_var) {
            Some(close_pos) => {
                let inner = &after_var[..close_pos];
                let resolved = resolve_single_var(inner.trim(), vars);
                result.push_str(&resolved);
                rest = &after_var[close_pos + 1..];
            }
            None => {
                // Malformed, pass through
                result.push_str("var(");
                rest = after_var;
            }
        }
    }

    result.push_str(rest);
    result
}

/// Find matching `)` for a `var(` where we're positioned after the `(`.
fn find_matching_paren(s: &str) -> Option<usize> {
    let mut depth = 1;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Resolve a single var() inner content like `--name` or `--name, fallback`.
fn resolve_single_var(inner: &str, vars: &HashMap<String, String>) -> String {
    // Split on first comma for fallback
    let (name, fallback) = match inner.find(',') {
        Some(comma) => (inner[..comma].trim(), Some(inner[comma + 1..].trim())),
        None => (inner.trim(), None),
    };

    if let Some(value) = vars.get(name) {
        value.clone()
    } else if let Some(fb) = fallback {
        fb.to_string()
    } else {
        // Unresolvable — pass through as-is (shouldn't happen with well-formed CSS)
        format!("var({})", inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_root_variables() {
        let css = r#"
:root {
    --bg: #ff0000;
    --fg: #00ff00;
}
body { color: var(--fg); background: var(--bg); }
"#;
        let (vars, stripped) = extract_root_variables(css);
        assert_eq!(vars.get("--bg").unwrap(), "#ff0000");
        assert_eq!(vars.get("--fg").unwrap(), "#00ff00");
        assert!(!stripped.contains(":root"));
        assert!(stripped.contains("var(--fg)"));
    }

    #[test]
    fn test_resolve_simple() {
        let mut vars = HashMap::new();
        vars.insert("--bg".to_string(), "#ff0000".to_string());
        let result = resolve_variables("body { background: var(--bg); }", &vars);
        assert_eq!(result, "body { background: #ff0000; }");
    }

    #[test]
    fn test_resolve_with_fallback() {
        let vars = HashMap::new();
        let result = resolve_variables("body { color: var(--missing, blue); }", &vars);
        assert_eq!(result, "body { color: blue; }");
    }

    #[test]
    fn test_resolve_nested_vars() {
        let mut vars = HashMap::new();
        vars.insert("--base".to_string(), "#111".to_string());
        vars.insert("--bg".to_string(), "var(--base)".to_string());
        let result = resolve_variables("body { background: var(--bg); }", &vars);
        assert_eq!(result, "body { background: #111; }");
    }

    #[test]
    fn test_multiple_root_blocks() {
        let css = r#"
:root { --a: red; }
body { color: var(--a); }
:root { --a: blue; }
div { color: var(--a); }
"#;
        let (vars, stripped) = extract_root_variables(css);
        // Second :root overrides first
        assert_eq!(vars.get("--a").unwrap(), "blue");
        assert!(!stripped.contains(":root"));
    }
}
