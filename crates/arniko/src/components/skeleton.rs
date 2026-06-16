//! Skeleton component for Arniko

use crate::{Component, ComponentMetadata};

pub struct Skeleton {
    width: String,
    height: String,
    class: String,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            width: "100%".to_string(),
            height: "20px".to_string(),
            class: String::new(),
        }
    }

    pub fn width(mut self, width: &str) -> Self {
        self.width = width.to_string();
        self
    }

    pub fn height(mut self, height: &str) -> Self {
        self.height = height.to_string();
        self
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        format!(
            r#"<div class="arniko-skeleton {}" style="width: {}; height: {};" role="status" aria-busy="true"></div>"#,
            self.class, self.width, self.height
        )
    }
}

impl Component for Skeleton {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

pub struct SkeletonCard {
    class: String,
}

impl SkeletonCard {
    pub fn new() -> Self {
        Self {
            class: String::new(),
        }
    }

    pub fn class(mut self, c: &str) -> Self {
        self.class = c.to_string();
        self
    }

    pub fn render(&self) -> String {
        format!(
            r#"<div class="arniko-skeleton-card {}">
                <div class="arniko-skeleton" style="width: 60%; height: 16px; margin-bottom: 8px;"></div>
                <div class="arniko-skeleton" style="width: 100%; height: 12px; margin-bottom: 4px;"></div>
                <div class="arniko-skeleton" style="width: 80%; height: 12px;"></div>
            </div>"#,
            self.class
        )
    }
}

impl Component for SkeletonCard {
    fn render(&self) -> String {
        self.render()
    }

    fn metadata(&self) -> ComponentMetadata {
        ComponentMetadata::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skeleton_default() {
        let skel = Skeleton::new();
        let html = skel.render();
        assert!(html.contains("arniko-skeleton"));
        assert!(html.contains(r#"role="status""#));
        assert!(html.contains(r#"aria-busy="true""#));
    }

    #[test]
    fn test_skeleton_custom_size() {
        let skel = Skeleton::new().width("50%").height("40px");
        let html = skel.render();
        assert!(html.contains("width: 50%"));
        assert!(html.contains("height: 40px"));
    }

    #[test]
    fn test_skeleton_card() {
        let card = SkeletonCard::new();
        let html = card.render();
        assert!(html.contains("arniko-skeleton-card"));
    }
}
