//! CSS normalization pipeline for Bliss compatibility.
//!
//! Transforms modern authoring CSS (var(), gradients, rem/em, grid, etc.)
//! into the safe subset that Bliss can render.

pub mod metadata;
mod resolver;
mod visitor;

pub use metadata::{
    DroppedFeature, FeatureType, NormalizationMetadata, NormalizationResult, SyntheticFeature,
};

/// Options for [`normalize`].
#[derive(Clone, Copy, Debug)]
pub struct NormalizeOptions {
    /// Keep CSS `transition`, `animation` and `@keyframes` so the engine's own
    /// animation runtime can run them.
    ///
    /// Default `true`. Set `false` to restore the historical behaviour — strip
    /// them, recording each in `NormalizationMetadata::dropped_features` — for a
    /// renderer that cannot animate.
    pub keep_animations: bool,
}

impl Default for NormalizeOptions {
    fn default() -> Self {
        NormalizeOptions {
            keep_animations: true,
        }
    }
}

/// Normalize CSS for Bliss compatibility.
///
/// 1. Extract and resolve CSS custom properties (var())
/// 2. Transform/drop unsupported features (gradients, transforms, etc.)
/// 3. Convert relative units (rem/em) to px
///
/// Returns the safe CSS string and metadata about dropped features.
///
/// `transition`, `animation` and `@keyframes` are **kept** by default so the
/// engine animates them; use [`normalize_with_options`] with
/// `keep_animations: false` to strip them.
pub fn normalize(css: &str) -> Result<NormalizationResult, String> {
    normalize_with_options(css, NormalizeOptions::default())
}

/// [`normalize`] with explicit [`NormalizeOptions`].
pub fn normalize_with_options(
    css: &str,
    options: NormalizeOptions,
) -> Result<NormalizationResult, String> {
    // Step 1: Extract :root variables and resolve var() references
    let (vars, stripped) = resolver::extract_root_variables(css);
    let resolved = resolver::resolve_variables(&stripped, &vars);

    // Step 2: Transform/drop unsupported features
    let (safe_css, metadata) = visitor::transform(&resolved, &options);

    Ok(NormalizationResult {
        css: safe_css,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "a { transition: opacity 0.2s; animation: spin 1s linear infinite; }\n\
                          @keyframes spin { to { transform: rotate(360deg); } }";

    #[test]
    fn keeps_transitions_animations_and_keyframes_by_default() {
        let out = normalize(SAMPLE).expect("normalize").css;
        assert!(out.contains("transition: opacity 0.2s"), "{out}");
        assert!(out.contains("animation: spin 1s linear infinite"), "{out}");
        assert!(out.contains("@keyframes spin"), "{out}");
        // The keyframe body is passed through verbatim (its transform survives).
        assert!(out.contains("rotate(360deg)"), "{out}");
    }

    #[test]
    fn can_drop_animations_when_opted_out() {
        let result = normalize_with_options(
            SAMPLE,
            NormalizeOptions {
                keep_animations: false,
            },
        )
        .expect("normalize");
        assert!(!result.css.contains("transition"), "{}", result.css);
        assert!(!result.css.contains("animation:"), "{}", result.css);
        assert!(!result.css.contains("@keyframes"), "{}", result.css);
        // The drops are still recorded as metadata.
        assert!(
            result
                .metadata
                .dropped_features
                .iter()
                .any(|f| f.feature_type == FeatureType::Transition)
        );
        assert!(
            result
                .metadata
                .dropped_features
                .iter()
                .any(|f| f.feature_type == FeatureType::Animation)
        );
    }

    #[cfg(feature = "components")]
    #[test]
    fn keeps_arniko_component_animations() {
        // The component stylesheet ships a spinner and a status pulse, both driven
        // by `@keyframes`; they must survive normalization.
        let out = normalize(crate::components::ARNIKO_STYLES)
            .expect("normalize")
            .css;
        assert!(
            out.contains("@keyframes arniko-spin"),
            "spinner keyframes stripped"
        );
        assert!(
            out.contains("animation: arniko-spin"),
            "spinner animation stripped"
        );
        assert!(
            out.contains("@keyframes arniko-pulse"),
            "pulse keyframes stripped"
        );
    }
}
