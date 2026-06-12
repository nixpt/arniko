//! Types for tracking what CSS features were dropped or transformed during normalization.

#[cfg(feature = "gpu")]
use mustang::{EffectMetadata, FeatureType as MFeatureType, SyntheticFeature as MSyntheticFeature};

/// Types of CSS features that may be dropped or transformed
#[derive(Debug, Clone, PartialEq)]
pub enum FeatureType {
    BackdropFilter,
    Transform,
    Animation,
    Transition,
    PseudoElement,
    BoxShadow,
    Gradient,
    Grid,
    FocusWithin,
    FocusVisible,
    TextOverflow,
}

/// A feature that was dropped during normalization
#[derive(Debug, Clone)]
pub struct DroppedFeature {
    pub feature_type: FeatureType,
    pub selector: String,
    pub original_value: String,
}

/// A feature that was transformed but needs compositor handling
#[derive(Debug, Clone)]
pub struct SyntheticFeature {
    pub feature_type: FeatureType,
    pub selector: String,
    pub original_value: String,
}

/// Metadata about CSS normalization
#[derive(Debug, Clone, Default)]
pub struct NormalizationMetadata {
    pub dropped_features: Vec<DroppedFeature>,
    pub synthetic_features: Vec<SyntheticFeature>,
}

/// Result of CSS normalization
#[derive(Debug, Clone)]
pub struct NormalizationResult {
    pub css: String,
    pub metadata: NormalizationMetadata,
}

#[cfg(feature = "gpu")]
impl EffectMetadata for NormalizationMetadata {
    fn extract_features(&self) -> Vec<MSyntheticFeature> {
        let mut features = Vec::new();
        for f in &self.synthetic_features {
            let m_type = match f.feature_type {
                FeatureType::BackdropFilter => Some(MFeatureType::BackdropFilter),
                FeatureType::Transform => Some(MFeatureType::Transform),
                // Others are not supported by Mustang yet
                _ => None,
            };

            if let Some(t) = m_type {
                features.push(MSyntheticFeature {
                    feature_type: t,
                    selector: f.selector.clone(),
                    original_value: f.original_value.clone(),
                });
            }
        }
        features
    }
}
