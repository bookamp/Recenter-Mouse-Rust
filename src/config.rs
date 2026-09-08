use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub win: bool,
    pub key: String,
    pub move_window: bool,
    pub start_with_windows: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            ctrl: true,
            alt: true,
            shift: false,
            win: false,
            key: "C".to_string(),
            move_window: true,
            start_with_windows: false,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(appdata) = std::env::var("APPDATA") {
            let mut p = PathBuf::from(appdata);
            p.push("RecenterMouse");
            p.push("config.json");
            return p;
        }
        PathBuf::from("recenter_mouse_config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                    return cfg;
                }
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn hotkey_display(&self) -> String {
        let mut parts = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.win {
            parts.push("Win");
        }
        parts.push(&self.key);
        parts.join(" + ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_display() {
        let cfg = AppConfig::default();
        assert_eq!(cfg.hotkey_display(), "Ctrl + Alt + C");

        let custom = AppConfig {
            ctrl: true,
            alt: false,
            shift: true,
            win: true,
            key: "F1".to_string(),
            move_window: true,
            start_with_windows: false,
        };
        assert_eq!(custom.hotkey_display(), "Ctrl + Shift + Win + F1");
    }

    #[test]
    fn test_serialization() {
        let cfg = AppConfig::default();
        let json = serde_json::to_string(&cfg).expect("Serialization failed");
        let deserialized: AppConfig = serde_json::from_str(&json).expect("Deserialization failed");
        assert_eq!(cfg.ctrl, deserialized.ctrl);
        assert_eq!(cfg.alt, deserialized.alt);
        assert_eq!(cfg.key, deserialized.key);
    }
}
