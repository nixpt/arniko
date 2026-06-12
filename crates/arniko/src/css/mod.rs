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

/// Normalize CSS for Bliss compatibility.
///
/// 1. Extract and resolve CSS custom properties (var())
/// 2. Transform/drop unsupported features (gradients, transforms, etc.)
/// 3. Convert relative units (rem/em) to px
///
/// Returns the safe CSS string and metadata about dropped features.
pub fn normalize(css: &str) -> Result<NormalizationResult, String> {
    // Step 1: Extract :root variables and resolve var() references
    let (vars, stripped) = resolver::extract_root_variables(css);
    let resolved = resolver::resolve_variables(&stripped, &vars);

    // Step 2: Transform/drop unsupported features
    let (safe_css, metadata) = visitor::transform(&resolved);

    Ok(NormalizationResult {
        css: safe_css,
        metadata,
    })
}
