/// Bliss-safe CSS transformer.
///
/// Performs regex/string-based transformations on CSS that has already had
/// var() references resolved. This avoids the complexity of lightning-css's
/// visitor API while achieving the same result for our use case.
///
/// Transforms:
/// - rem/em units → px (×16)
/// - linear-gradient(...) → first color stop
/// - display: grid → display: flex
/// - grid-template-columns → dropped
/// - backdrop-filter → dropped (tagged as synthetic)
/// - transform → dropped (tagged as synthetic)
/// - transition → dropped
/// - animation / @keyframes → dropped
/// - ::before / ::after rules → dropped
/// - :focus-within / :focus-visible rules → dropped
/// - text-overflow: ellipsis → dropped
/// - multi-layer box-shadow → first layer only
use crate::css::{DroppedFeature, FeatureType, NormalizationMetadata, SyntheticFeature};

pub fn transform(css: &str) -> (String, NormalizationMetadata) {
    let mut metadata = NormalizationMetadata::default();
    let mut output = String::with_capacity(css.len());

    // First pass: handle @keyframes removal
    let css = remove_keyframes(&css, &mut metadata);

    // Process rule by rule
    let mut rest = css.as_str();
    while !rest.is_empty() {
        // Skip whitespace
        let trimmed = rest.trim_start();
        if trimmed.is_empty() {
            break;
        }
        let skipped = rest.len() - trimmed.len();
        output.push_str(&rest[..skipped]);
        rest = trimmed;

        // Check if this is a rule block (selector { ... })
        if let Some(brace_pos) = rest.find('{') {
            let selector = rest[..brace_pos].trim();

            // Check if entire rule should be dropped
            if should_drop_rule(selector) {
                let block_end = find_block_end(&rest[brace_pos..]);
                let full_rule = &rest[..brace_pos + block_end + 1];
                tag_dropped_rule(selector, full_rule, &mut metadata);
                rest = &rest[brace_pos + block_end + 1..];
                continue;
            }

            // Keep the rule but transform its declarations
            output.push_str(selector);
            output.push_str(" {\n");

            let block_end = find_block_end(&rest[brace_pos..]);
            let block_body = &rest[brace_pos + 1..brace_pos + block_end];

            transform_declarations(block_body, selector, &mut output, &mut metadata);

            output.push_str("}\n");
            rest = &rest[brace_pos + block_end + 1..];
        } else {
            // No more rules, append rest
            output.push_str(rest);
            break;
        }
    }

    (output, metadata)
}

fn should_drop_rule(selector: &str) -> bool {
    selector.contains("::before")
        || selector.contains("::after")
        || selector.contains(":focus-within")
        || selector.contains(":focus-visible")
}

fn tag_dropped_rule(selector: &str, _full_rule: &str, metadata: &mut NormalizationMetadata) {
    let feature_type = if selector.contains("::before") || selector.contains("::after") {
        FeatureType::PseudoElement
    } else if selector.contains(":focus-within") {
        FeatureType::FocusWithin
    } else {
        FeatureType::FocusVisible
    };

    metadata.dropped_features.push(DroppedFeature {
        feature_type,
        selector: selector.to_string(),
        original_value: String::new(),
    });
}

fn find_block_end(s: &str) -> usize {
    let mut depth = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    s.len().saturating_sub(1)
}

fn remove_keyframes(css: &str, metadata: &mut NormalizationMetadata) -> String {
    let mut result = String::with_capacity(css.len());
    let mut rest = css;

    while let Some(kf_pos) = rest.find("@keyframes") {
        result.push_str(&rest[..kf_pos]);

        let after = &rest[kf_pos..];
        if let Some(brace_pos) = after.find('{') {
            let block_end = find_block_end(&after[brace_pos..]);
            let name = after[10..brace_pos].trim();
            metadata.dropped_features.push(DroppedFeature {
                feature_type: FeatureType::Animation,
                selector: format!("@keyframes {}", name),
                original_value: String::new(),
            });
            rest = &rest[kf_pos + brace_pos + block_end + 1..];
        } else {
            result.push_str(after);
            rest = "";
        }
    }

    result.push_str(rest);
    result
}

fn transform_declarations(
    block: &str,
    selector: &str,
    output: &mut String,
    metadata: &mut NormalizationMetadata,
) {
    for decl in split_declarations(block) {
        let decl = decl.trim();
        if decl.is_empty() || decl.starts_with("/*") {
            continue;
        }

        let Some(colon) = decl.find(':') else {
            continue;
        };

        let prop = decl[..colon].trim();
        let value = decl[colon + 1..].trim();

        match prop {
            "backdrop-filter" | "-webkit-backdrop-filter" => {
                metadata.synthetic_features.push(SyntheticFeature {
                    feature_type: FeatureType::BackdropFilter,
                    selector: selector.to_string(),
                    original_value: format!("{}: {}", prop, value),
                });
                continue;
            }
            "transition"
            | "transition-property"
            | "transition-duration"
            | "transition-timing-function"
            | "transition-delay" => {
                metadata.dropped_features.push(DroppedFeature {
                    feature_type: FeatureType::Transition,
                    selector: selector.to_string(),
                    original_value: format!("{}: {}", prop, value),
                });
                continue;
            }
            "animation"
            | "animation-name"
            | "animation-duration"
            | "animation-timing-function"
            | "animation-delay"
            | "animation-iteration-count"
            | "animation-direction" => {
                metadata.dropped_features.push(DroppedFeature {
                    feature_type: FeatureType::Animation,
                    selector: selector.to_string(),
                    original_value: format!("{}: {}", prop, value),
                });
                continue;
            }
            "text-overflow" => {
                metadata.dropped_features.push(DroppedFeature {
                    feature_type: FeatureType::TextOverflow,
                    selector: selector.to_string(),
                    original_value: format!("text-overflow: {}", value),
                });
                continue;
            }
            "grid-template-columns"
            | "grid-template-rows"
            | "grid-template-areas"
            | "grid-column"
            | "grid-row"
            | "grid-area"
            | "grid-gap" => {
                metadata.dropped_features.push(DroppedFeature {
                    feature_type: FeatureType::Grid,
                    selector: selector.to_string(),
                    original_value: format!("{}: {}", prop, value),
                });
                continue;
            }
            _ => {}
        }

        // Transform values
        let mut transformed_value = value.to_string();

        // display: grid → display: flex
        if prop == "display" && value.trim() == "grid" {
            metadata.dropped_features.push(DroppedFeature {
                feature_type: FeatureType::Grid,
                selector: selector.to_string(),
                original_value: "display: grid".to_string(),
            });
            transformed_value = "flex".to_string();
        }

        // linear-gradient → first color stop
        if transformed_value.contains("linear-gradient(") {
            let original = transformed_value.clone();
            transformed_value = extract_gradient_color(&transformed_value);
            metadata.dropped_features.push(DroppedFeature {
                feature_type: FeatureType::Gradient,
                selector: selector.to_string(),
                original_value: format!("{}: {}", prop, original),
            });
        }

        // box-shadow: simplify to first layer
        if prop == "box-shadow" {
            let original = transformed_value.clone();
            transformed_value = simplify_box_shadow(&transformed_value);
            if transformed_value != original {
                metadata.dropped_features.push(DroppedFeature {
                    feature_type: FeatureType::BoxShadow,
                    selector: selector.to_string(),
                    original_value: format!("box-shadow: {}", original),
                });
            }
        }

        // rem/em → px conversion
        transformed_value = convert_relative_units(&transformed_value);

        output.push_str("    ");
        output.push_str(prop);
        output.push_str(": ");
        output.push_str(&transformed_value);
        output.push_str(";\n");
    }
}

/// Split declarations by `;` but respect parentheses (for rgba(), etc.)
fn split_declarations(block: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, ch) in block.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ';' if depth == 0 => {
                result.push(&block[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }

    let trailing = block[start..].trim();
    if !trailing.is_empty() {
        result.push(&block[start..]);
    }

    result
}

/// Extract the first color stop from a linear-gradient.
fn extract_gradient_color(value: &str) -> String {
    let Some(start) = value.find("linear-gradient(") else {
        return value.to_string();
    };

    let after = &value[start + 16..];
    let Some(close) = find_matching_paren_str(after) else {
        return value.to_string();
    };

    let inner = &after[..close];

    // Split by commas respecting parens
    let parts = split_respecting_parens(inner, ',');

    // Find first part that looks like a color (skip angle like "180deg", "to right", etc.)
    for part in &parts {
        let p = part.trim();
        if looks_like_color(p) {
            // Extract just the color part (strip percentage/length if present)
            let color = extract_color_from_stop(p);
            return color;
        }
    }

    // Fallback: try second part (first is usually the angle)
    if parts.len() >= 2 {
        let p = parts[1].trim();
        let color = extract_color_from_stop(p);
        return color;
    }

    value.to_string()
}

fn looks_like_color(s: &str) -> bool {
    let s = s.trim();
    s.starts_with('#')
        || s.starts_with("rgb")
        || s.starts_with("rgba")
        || s.starts_with("hsl")
        || is_named_color(s.split_whitespace().next().unwrap_or(""))
}

fn is_named_color(s: &str) -> bool {
    matches!(
        s.to_lowercase().as_str(),
        "red"
            | "blue"
            | "green"
            | "white"
            | "black"
            | "gray"
            | "grey"
            | "yellow"
            | "orange"
            | "purple"
            | "pink"
            | "cyan"
            | "transparent"
    )
}

fn extract_color_from_stop(s: &str) -> String {
    let s = s.trim();
    // If it contains rgba/rgb, extract that
    if let Some(start) = s.find("rgba(").or_else(|| s.find("rgb(")) {
        let after = &s[start..];
        if let Some(end) = find_matching_paren_str(&after[after.find('(').unwrap() + 1..]) {
            let fn_name_len = after.find('(').unwrap() + 1;
            return after[..fn_name_len + end + 1].to_string();
        }
    }
    // If it starts with #, take the hex color
    if s.starts_with('#') {
        return s.split_whitespace().next().unwrap_or(s).to_string();
    }
    // Named color
    s.split_whitespace().next().unwrap_or(s).to_string()
}

fn find_matching_paren_str(s: &str) -> Option<usize> {
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

fn split_respecting_parens(s: &str, delim: char) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c == delim && depth == 0 => {
                result.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    result.push(&s[start..]);
    result
}

/// Keep only the first layer of a multi-layer box-shadow.
fn simplify_box_shadow(value: &str) -> String {
    let layers = split_respecting_parens(value, ',');
    if layers.len() > 1 {
        layers[0].trim().to_string()
    } else {
        value.to_string()
    }
}

/// Convert rem and em units to px (base = 16px).
fn convert_relative_units(value: &str) -> String {
    let mut result = String::new();
    let mut rest = value;

    while !rest.is_empty() {
        // Find a number potentially followed by rem or em
        if let Some((before, num, unit, after)) = find_relative_unit(rest) {
            result.push_str(before);
            let px = (num * 16.0).round() as i64;
            result.push_str(&format!("{}px", px));
            if unit == "rem" {
                rest = after;
            } else {
                rest = after;
            }
        } else {
            result.push_str(rest);
            break;
        }
    }

    result
}

/// Find the next occurrence of a number followed by `rem` or `em`.
fn find_relative_unit(s: &str) -> Option<(&str, f64, &str, &str)> {
    // Look for patterns like "1.5rem" or "2em" or ".5rem"
    let bytes = s.as_bytes();
    let len = bytes.len();

    for i in 0..len {
        // Check for "rem" or "em" at position i
        let (unit, unit_len) = if i + 3 <= len && &s[i..i + 3] == "rem" {
            ("rem", 3)
        } else if i + 2 <= len && &s[i..i + 2] == "em" {
            // Make sure it's not "rem"
            if i > 0 && bytes[i - 1] == b'r' {
                continue;
            }
            ("em", 2)
        } else {
            continue;
        };

        // Check that what follows is not an alpha char (avoid matching "system" etc)
        let after_unit = i + unit_len;
        if after_unit < len && bytes[after_unit].is_ascii_alphabetic() {
            continue;
        }

        // Walk backwards to find the number
        let mut num_start = i;
        while num_start > 0 {
            let ch = bytes[num_start - 1];
            if ch.is_ascii_digit() || ch == b'.' || ch == b'-' {
                num_start -= 1;
            } else {
                break;
            }
        }

        if num_start < i {
            let num_str = &s[num_start..i];
            if let Ok(num) = num_str.parse::<f64>() {
                return Some((&s[..num_start], num, unit, &s[i + unit_len..]));
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rem_conversion() {
        assert_eq!(convert_relative_units("1rem"), "16px");
        assert_eq!(convert_relative_units("1.5rem"), "24px");
        assert_eq!(convert_relative_units("0.5em"), "8px");
        assert_eq!(convert_relative_units("10px"), "10px");
    }

    #[test]
    fn test_gradient_extraction() {
        assert_eq!(
            extract_gradient_color("linear-gradient(180deg, #ff0000, #0000ff)"),
            "#ff0000"
        );
        assert_eq!(
            extract_gradient_color("linear-gradient(to right, rgba(0,0,0,0.5), transparent)"),
            "rgba(0,0,0,0.5)"
        );
    }

    #[test]
    fn test_box_shadow_simplify() {
        assert_eq!(
            simplify_box_shadow("0 2px 4px rgba(0,0,0,0.1), 0 4px 8px rgba(0,0,0,0.2)"),
            "0 2px 4px rgba(0,0,0,0.1)"
        );
        assert_eq!(
            simplify_box_shadow("0 2px 4px rgba(0,0,0,0.1)"),
            "0 2px 4px rgba(0,0,0,0.1)"
        );
    }

    #[test]
    fn test_drop_rules() {
        let (output, metadata) = transform("div::before { content: ''; } div { color: red; }");
        assert!(!output.contains("::before"));
        assert!(output.contains("color: red"));
        assert!(
            metadata
                .dropped_features
                .iter()
                .any(|f| f.feature_type == FeatureType::PseudoElement)
        );
    }

    #[test]
    fn test_transform_properties_dropped() {
        let (output, metadata) = transform(".box { transform: scale(1.1); color: red; }");
        assert!(!output.contains("transform:"));
        assert!(output.contains("color: red"));
        assert!(
            metadata
                .synthetic_features
                .iter()
                .any(|f| f.feature_type == FeatureType::Transform)
        );
    }

    #[test]
    fn test_grid_to_flex() {
        let (output, metadata) =
            transform(".grid { display: grid; grid-template-columns: 1fr 1fr; color: red; }");
        assert!(output.contains("display: flex"));
        assert!(!output.contains("grid-template-columns"));
        assert!(!output.contains("display: grid"));
        assert!(
            metadata
                .dropped_features
                .iter()
                .any(|f| f.feature_type == FeatureType::Grid)
        );
    }
}
