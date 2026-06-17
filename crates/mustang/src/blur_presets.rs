//! MacTahoe-inspired blur presets for Arniko's Mustang GPU compositor
//!
//! This module provides MacTahoe-style blur configurations inspired by:
//! - Transparent blur versions of GTK themes
//! - blur-my-shell GNOME extension settings
//! - Configurable blur with multiple presets
//!
//! These presets use the same blur effect infrastructure in Mustang but
//! provide convenient, MacTahoe-inspired configuration values.

use crate::effect::{BlurParams, BlurQuality};

/// MacTahoe-inspired blur preset configurations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MacTahoeBlurPreset {
    /// No blur - default for non-glass surfaces
    None,
    /// Subtle blur - light frosted effect (blur(4px), 1 pass)
    Subtle,
    /// Medium blur - standard glass effect (blur(8px), 2 passes)
    Medium,
    /// Strong blur - heavy frosted glass (blur(12px), 2 passes)
    Strong,
    /// Ultra blur - maximum frosted effect (blur(16px), 3 passes)
    Ultra,
    /// MacTahoe "Frosted" - classic glassmorphism (blur(10px), 2 passes)
    Frosted,
    /// MacTahoe "Aurora" - pastel glass blur (blur(8px), 2 passes)
    Aurora,
    /// MacTahoe "Cyberpunk" - neon blur effect (blur(12px), 2 passes)
    Cyberpunk,
}

impl Default for MacTahoeBlurPreset {
    fn default() -> Self {
        MacTahoeBlurPreset::Medium
    }
}

impl MacTahoeBlurPreset {
    /// Get the blur radius for this preset
    pub fn radius(&self) -> f32 {
        match self {
            MacTahoeBlurPreset::None => 0.0,
            MacTahoeBlurPreset::Subtle => 4.0,
            MacTahoeBlurPreset::Medium => 8.0,
            MacTahoeBlurPreset::Strong => 12.0,
            MacTahoeBlurPreset::Ultra => 16.0,
            MacTahoeBlurPreset::Frosted => 10.0,
            MacTahoeBlurPreset::Aurora => 8.0,
            MacTahoeBlurPreset::Cyberpunk => 12.0,
        }
    }

    /// Get the number of blur passes for this preset
    pub fn passes(&self) -> u32 {
        match self {
            MacTahoeBlurPreset::None => 1,
            MacTahoeBlurPreset::Subtle => 1,
            MacTahoeBlurPreset::Medium => 2,
            MacTahoeBlurPreset::Strong => 2,
            MacTahoeBlurPreset::Ultra => 3,
            MacTahoeBlurPreset::Frosted => 2,
            MacTahoeBlurPreset::Aurora => 2,
            MacTahoeBlurPreset::Cyberpunk => 2,
        }
    }

    /// Get the blur quality level for this preset
    pub fn quality(&self) -> BlurQuality {
        match self {
            MacTahoeBlurPreset::None => BlurQuality::Low,
            MacTahoeBlurPreset::Subtle => BlurQuality::Low,
            MacTahoeBlurPreset::Medium => BlurQuality::Medium,
            MacTahoeBlurPreset::Strong => BlurQuality::High,
            MacTahoeBlurPreset::Ultra => BlurQuality::Ultra,
            MacTahoeBlurPreset::Frosted => BlurQuality::High,
            MacTahoeBlurPreset::Aurora => BlurQuality::Medium,
            MacTahoeBlurPreset::Cyberpunk => BlurQuality::High,
        }
    }

    /// Create BlurParams for this preset
    pub fn params(&self) -> BlurParams {
        BlurParams {
            radius: self.radius(),
            passes: self.passes(),
            quality: self.quality(),
        }
    }

    /// Create an Effect with this preset
    pub fn effect(&self, selector: &str, viewport_width: u32, viewport_height: u32) -> crate::effect::Effect {
        crate::effect::Effect::blur(
            selector,
            self.radius(),
            viewport_width,
            viewport_height,
        )
    }

    /// Get a display name for this preset
    pub fn name(&self) -> &'static str {
        match self {
            MacTahoeBlurPreset::None => "None",
            MacTahoeBlurPreset::Subtle => "Subtle",
            MacTahoeBlurPreset::Medium => "Medium",
            MacTahoeBlurPreset::Strong => "Strong",
            MacTahoeBlurPreset::Ultra => "Ultra",
            MacTahoeBlurPreset::Frosted => "Frosted",
            MacTahoeBlurPreset::Aurora => "Aurora",
            MacTahoeBlurPreset::Cyberpunk => "Cyberpunk",
        }
    }

    /// Get a description for this preset
    pub fn description(&self) -> &'static str {
        match self {
            MacTahoeBlurPreset::None => "No blur effect",
            MacTahoeBlurPreset::Subtle => "Light frosted effect for subtle glass",
            MacTahoeBlurPreset::Medium => "Standard glass effect for general use",
            MacTahoeBlurPreset::Strong => "Heavy frosted glass for dramatic effect",
            MacTahoeBlurPreset::Ultra => "Maximum frosted effect, slowest",
            MacTahoeBlurPreset::Frosted => "Classic MacTahoe frosted glass",
            MacTahoeBlurPreset::Aurora => "MacTahoe Aurora pastel glass",
            MacTahoeBlurPreset::Cyberpunk => "MacTahoe Cyberpunk neon blur",
        }
    }
}

/// Default blur configuration matching MacTahoe's typical usage
///
/// MacTahoe's blur-my-shell extension typically uses:
/// - `blur(4px)` for subtle glass
/// - `blur(8px)` for standard glass
/// - `blur(10px)` for frosted effects
/// - `blur(12px)` for heavy frosted effects
///
/// This configuration is suitable as a default for Arniko's Mustang compositor
pub const MACTAHOE_DEFAULT_BLUR_PARAMS: BlurParams = BlurParams {
    radius: 10.0,
    passes: 2,
    quality: BlurQuality::High,
};

/// All available MacTahoe blur presets
pub fn all_presets() -> &'static [MacTahoeBlurPreset] {
    &[
        MacTahoeBlurPreset::None,
        MacTahoeBlurPreset::Subtle,
        MacTahoeBlurPreset::Medium,
        MacTahoeBlurPreset::Frosted,
        MacTahoeBlurPreset::Strong,
        MacTahoeBlurPreset::Aurora,
        MacTahoeBlurPreset::Cyberpunk,
        MacTahoeBlurPreset::Ultra,
    ]
}

/// Builder for creating custom blur configurations
///
/// Provides a fluent API for customizing blur parameters
/// for MacTahoe-style effects
pub struct BlurPresetBuilder {
    radius: f32,
    passes: u32,
    quality: BlurQuality,
}

impl BlurPresetBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            radius: 10.0,
            passes: 2,
            quality: BlurQuality::High,
        }
    }

    /// Set blur radius
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Set number of blur passes
    pub fn passes(mut self, passes: u32) -> Self {
        self.passes = passes;
        self
    }

    /// Set blur quality
    pub fn quality(mut self, quality: BlurQuality) -> Self {
        self.quality = quality;
        self
    }

    /// Build the BlurParams
    pub fn build(self) -> BlurParams {
        BlurParams {
            radius: self.radius,
            passes: self.passes,
            quality: self.quality,
        }
    }

    /// Create a blur effect with the built parameters
    pub fn effect(self, selector: &str, viewport_width: u32, viewport_height: u32) -> crate::effect::Effect {
        crate::effect::Effect::blur(selector, self.build().radius, viewport_width, viewport_height)
    }
}

impl Default for BlurPresetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blur_presets() {
        // Test all preset values
        assert_eq!(MacTahoeBlurPreset::None.radius(), 0.0);
        assert_eq!(MacTahoeBlurPreset::None.passes(), 1);
        assert_eq!(MacTahoeBlurPreset::None.quality(), BlurQuality::Low);

        assert_eq!(MacTahoeBlurPreset::Subtle.radius(), 4.0);
        assert_eq!(MacTahoeBlurPreset::Subtle.passes(), 1);
        assert_eq!(MacTahoeBlurPreset::Subtle.quality(), BlurQuality::Low);

        assert_eq!(MacTahoeBlurPreset::Medium.radius(), 8.0);
        assert_eq!(MacTahoeBlurPreset::Medium.passes(), 2);
        assert_eq!(MacTahoeBlurPreset::Medium.quality(), BlurQuality::Medium);

        assert_eq!(MacTahoeBlurPreset::Frosted.radius(), 10.0);
        assert_eq!(MacTahoeBlurPreset::Frosted.passes(), 2);
        assert_eq!(MacTahoeBlurPreset::Frosted.quality(), BlurQuality::High);

        assert_eq!(MacTahoeBlurPreset::Strong.radius(), 12.0);
        assert_eq!(MacTahoeBlurPreset::Strong.passes(), 2);
        assert_eq!(MacTahoeBlurPreset::Strong.quality(), BlurQuality::High);

        assert_eq!(MacTahoeBlurPreset::Ultra.radius(), 16.0);
        assert_eq!(MacTahoeBlurPreset::Ultra.passes(), 3);
        assert_eq!(MacTahoeBlurPreset::Ultra.quality(), BlurQuality::Ultra);
    }

    #[test]
    fn test_blur_builder() {
        let params = BlurPresetBuilder::new()
            .radius(15.0)
            .passes(3)
            .quality(BlurQuality::Ultra)
            .build();

        assert_eq!(params.radius, 15.0);
        assert_eq!(params.passes, 3);
        assert_eq!(params.quality, BlurQuality::Ultra);
    }

    #[test]
    fn test_all_presets() {
        let presets = all_presets();
        assert_eq!(presets.len(), 8);
        assert!(presets.contains(&MacTahoeBlurPreset::Medium));
        assert!(presets.contains(&MacTahoeBlurPreset::Frosted));
    }
}
