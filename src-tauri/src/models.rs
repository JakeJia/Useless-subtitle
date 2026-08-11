use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const CONFIG_SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_WIDTH: f64 = 600.0;
pub const DEFAULT_HEIGHT: f64 = 120.0;
pub const MIN_WIDTH: f64 = 96.0;
pub const MIN_HEIGHT: f64 = 32.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub schema_version: u32,
    pub initialized: bool,
    pub masks: Vec<MaskConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: CONFIG_SCHEMA_VERSION,
            initialized: false,
            masks: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn validate(mut self) -> Self {
        self.schema_version = CONFIG_SCHEMA_VERSION;

        let mut seen = std::collections::HashSet::new();
        self.masks.retain_mut(|mask| {
            mask.validate();
            !mask.id.is_empty() && seen.insert(mask.id.clone())
        });

        self
    }

    pub fn next_name(&self) -> String {
        let mut number = 1usize;
        loop {
            let candidate = format!("Mask {number}");
            if self.masks.iter().all(|mask| mask.name != candidate) {
                return candidate;
            }
            number += 1;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaskConfig {
    pub id: String,
    pub name: String,
    pub appearance: MaskAppearance,
    pub geometry: MaskGeometry,
}

impl MaskConfig {
    pub fn new(name: String, geometry: MaskGeometry) -> Self {
        Self {
            id: Uuid::new_v4().simple().to_string(),
            name,
            appearance: MaskAppearance::default(),
            geometry,
        }
    }

    pub fn label(&self) -> String {
        format!("mask-{}", self.id)
    }

    pub fn validate(&mut self) {
        if Uuid::parse_str(&self.id).is_err() {
            self.id = Uuid::new_v4().simple().to_string();
        }
        if self.name.trim().is_empty() {
            self.name = "Mask".to_string();
        }
        self.appearance.validate();
        self.geometry.validate();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaskAppearance {
    pub color: String,
    pub opacity: u8,
}

impl Default for MaskAppearance {
    fn default() -> Self {
        Self {
            color: "#000000".to_string(),
            opacity: 90,
        }
    }
}

impl MaskAppearance {
    pub fn validate(&mut self) {
        if !is_valid_color(&self.color) {
            self.color = "#000000".to_string();
        } else {
            self.color = self.color.to_ascii_uppercase();
        }
        self.opacity = self.opacity.clamp(10, 100);
        self.opacity -= self.opacity % 10;
        if self.opacity == 0 {
            self.opacity = 10;
        }
    }
}

pub fn is_valid_color(color: &str) -> bool {
    color.len() == 7
        && color.starts_with('#')
        && color[1..]
            .chars()
            .all(|character| character.is_ascii_hexdigit())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MaskGeometry {
    pub monitor_key: Option<String>,
    pub offset_x_logical: f64,
    pub offset_y_logical: f64,
    pub width_logical: f64,
    pub height_logical: f64,
    pub saved_scale_factor: f64,
}

impl Default for MaskGeometry {
    fn default() -> Self {
        Self {
            monitor_key: None,
            offset_x_logical: 100.0,
            offset_y_logical: 100.0,
            width_logical: DEFAULT_WIDTH,
            height_logical: DEFAULT_HEIGHT,
            saved_scale_factor: 1.0,
        }
    }
}

impl MaskGeometry {
    pub fn validate(&mut self) {
        if !self.offset_x_logical.is_finite() {
            self.offset_x_logical = 100.0;
        }
        if !self.offset_y_logical.is_finite() {
            self.offset_y_logical = 100.0;
        }
        if !self.width_logical.is_finite() {
            self.width_logical = DEFAULT_WIDTH;
        }
        if !self.height_logical.is_finite() {
            self.height_logical = DEFAULT_HEIGHT;
        }
        if !self.saved_scale_factor.is_finite() || self.saved_scale_factor <= 0.0 {
            self.saved_scale_factor = 1.0;
        }
        self.width_logical = self.width_logical.max(MIN_WIDTH);
        self.height_logical = self.height_logical.max(MIN_HEIGHT);
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MaskViewState {
    pub id: String,
    pub name: String,
    pub color: String,
    pub opacity: u8,
    pub locked: bool,
    pub visible: bool,
    pub tray_ready: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeMaskState {
    pub locked: bool,
    pub visible: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_appearance_values() {
        let mut appearance = MaskAppearance {
            color: "#ab12ef".into(),
            opacity: 97,
        };
        appearance.validate();
        assert_eq!(appearance.color, "#AB12EF");
        assert_eq!(appearance.opacity, 90);

        appearance.color = "invalid".into();
        appearance.opacity = 0;
        appearance.validate();
        assert_eq!(
            appearance,
            MaskAppearance {
                color: "#000000".into(),
                opacity: 10
            }
        );
    }

    #[test]
    fn removes_duplicate_identifiers() {
        let mask = MaskConfig::new("Mask 1".into(), MaskGeometry::default());
        let config = AppConfig {
            schema_version: 0,
            initialized: true,
            masks: vec![mask.clone(), mask],
        }
        .validate();

        assert_eq!(config.schema_version, CONFIG_SCHEMA_VERSION);
        assert_eq!(config.masks.len(), 1);
    }

    #[test]
    fn generates_an_unused_display_name() {
        let mut config = AppConfig::default();
        config
            .masks
            .push(MaskConfig::new("Mask 1".into(), MaskGeometry::default()));
        config
            .masks
            .push(MaskConfig::new("Mask 3".into(), MaskGeometry::default()));
        assert_eq!(config.next_name(), "Mask 2");
    }
}
