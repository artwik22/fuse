use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use std::fs;
use dirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub background: String,
    pub primary: String,
    pub secondary: String,
    pub text: String,
    pub accent: String,
    #[serde(rename = "lastWallpaper", skip_serializing_if = "Option::is_none")]
    pub last_wallpaper: Option<String>,
    #[serde(rename = "colorPreset", skip_serializing_if = "Option::is_none")]
    pub color_preset: Option<String>,
    #[serde(rename = "sidebarPosition", skip_serializing_if = "Option::is_none")]
    pub sidebar_position: Option<String>,
    #[serde(rename = "notificationsEnabled", skip_serializing_if = "Option::is_none")]
    pub notifications_enabled: Option<bool>,
    #[serde(rename = "notificationSoundsEnabled", skip_serializing_if = "Option::is_none")]
    pub notification_sounds_enabled: Option<bool>,
    #[serde(rename = "sidebarVisible", skip_serializing_if = "Option::is_none")]
    pub sidebar_visible: Option<bool>,
    #[serde(rename = "rounding", skip_serializing_if = "Option::is_none")]
    pub rounding: Option<String>,
    #[serde(rename = "showHiddenFiles", skip_serializing_if = "Option::is_none")]
    pub show_hidden_files: Option<bool>,
    #[serde(rename = "uiScale", skip_serializing_if = "Option::is_none")]
    pub ui_scale: Option<u8>,
    #[serde(rename = "dashboardTileLeft", skip_serializing_if = "Option::is_none")]
    pub dashboard_tile_left: Option<String>,
    #[serde(rename = "sidepanelContent", skip_serializing_if = "Option::is_none")]
    pub sidepanel_content: Option<String>,
    #[serde(rename = "githubUsername", skip_serializing_if = "Option::is_none")]
    pub github_username: Option<String>,
    #[serde(rename = "dashboardPosition", skip_serializing_if = "Option::is_none")]
    pub dashboard_position: Option<String>,
    #[serde(rename = "scriptsAutostartBattery", skip_serializing_if = "Option::is_none")]
    pub scripts_autostart_battery: Option<bool>,
    #[serde(rename = "scriptsAutostartScreensaver", skip_serializing_if = "Option::is_none")]
    pub scripts_autostart_screensaver: Option<bool>,
    #[serde(rename = "batteryThreshold", skip_serializing_if = "Option::is_none")]
    pub battery_threshold: Option<u8>,
    #[serde(rename = "screensaverTimeout", skip_serializing_if = "Option::is_none")]
    pub screensaver_timeout: Option<u32>,
    #[serde(rename = "dashboardResource1", skip_serializing_if = "Option::is_none")]
    pub dashboard_resource_1: Option<String>,
    #[serde(rename = "dashboardResource2", skip_serializing_if = "Option::is_none")]
    pub dashboard_resource_2: Option<String>,
    #[serde(rename = "scriptsAutostartAutofloat", skip_serializing_if = "Option::is_none")]
    pub scripts_autostart_autofloat: Option<bool>,
    #[serde(rename = "autofloatWidth", skip_serializing_if = "Option::is_none")]
    pub autofloat_width: Option<u32>,
    #[serde(rename = "autofloatHeight", skip_serializing_if = "Option::is_none")]
    pub autofloat_height: Option<u32>,
    #[serde(rename = "scriptsUseLockscreen", skip_serializing_if = "Option::is_none")]
    pub scripts_use_lockscreen: Option<bool>,
    #[serde(rename = "notificationPosition", skip_serializing_if = "Option::is_none")]
    pub notification_position: Option<String>,
    #[serde(rename = "notificationRounding", skip_serializing_if = "Option::is_none")]
    pub notification_rounding: Option<String>,
    #[serde(rename = "quickshellBorderRadius", skip_serializing_if = "Option::is_none")]
    pub quickshell_border_radius: Option<u8>,
    #[serde(rename = "notificationSound", skip_serializing_if = "Option::is_none")]
    pub notification_sound: Option<String>,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            background: "#0a0a0a".to_string(),
            primary: "#1a1a1a".to_string(),
            secondary: "#121212".to_string(),
            text: "#ffffff".to_string(),
            accent: "#4a9eff".to_string(),
            last_wallpaper: None,
            color_preset: None,
            sidebar_position: Some("left".to_string()),
            notifications_enabled: Some(true),
            notification_sounds_enabled: Some(true),
            sidebar_visible: Some(true),
            rounding: Some("rounded".to_string()),
            show_hidden_files: Some(false),
            ui_scale: Some(100),
            dashboard_tile_left: Some("battery".to_string()),
            sidepanel_content: Some("calendar".to_string()),
            github_username: None,
            dashboard_position: Some("right".to_string()), // Default to right like sidebar
            scripts_autostart_battery: Some(false),
            scripts_autostart_screensaver: Some(false),
            battery_threshold: Some(10),
            screensaver_timeout: Some(30),
            dashboard_resource_1: Some("cpu".to_string()),
            dashboard_resource_2: Some("ram".to_string()),
            scripts_autostart_autofloat: Some(false),
            autofloat_width: Some(1000),
            autofloat_height: Some(700),
            scripts_use_lockscreen: Some(false),
            notification_position: Some("top".to_string()),
            notification_rounding: Some("standard".to_string()),
            quickshell_border_radius: Some(0),
            notification_sound: Some("message.oga".to_string()),
        }
    }
}

impl ColorConfig {
    pub fn get_config_path() -> PathBuf {
        // 1. Try ~/.config/alloy/colors.json (Global Alloy Config)
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".config").join("alloy").join("colors.json");
            if path.exists() {
                return path;
            }
        }

        // 2. Try QUICKSHELL_PROJECT_PATH first
        if let Ok(project_path) = std::env::var("QUICKSHELL_PROJECT_PATH") {
            let path = PathBuf::from(project_path).join("colors.json");
            if path.exists() {
                return path;
            }
        }

        // 3. Fallback to ~/.config/sharpshell/colors.json
        if let Some(home) = dirs::home_dir() {
            let path = home.join(".config").join("sharpshell").join("colors.json");
            if path.exists() {
                return path;
            }
            // Create directory if it doesn't exist
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            return path;
        }

        // Last resort: /tmp/sharpshell/colors.json
        PathBuf::from("/tmp/sharpshell/colors.json")
    }

    pub fn load() -> Self {
        let path = Self::get_config_path();
        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => {
                let mut config = match serde_json::from_str::<ColorConfig>(&content) {
                    Ok(c) => c,
                    Err(_) => return Self::default(),
                };
                // Resolve preset from presets[...] when colorPreset is set – same logic as Quickshell shell.qml
                if let Some(ref preset_name) = config.color_preset {
                    if let Ok(root) = serde_json::from_str::<Value>(&content) {
                        if let Some(preset) = root
                            .get("presets")
                            .and_then(|p| p.get(preset_name))
                            .and_then(|p| p.as_object())
                        {
                            if let (Some(bg), Some(pr), Some(sec), Some(txt), Some(acc)) = (
                                preset.get("background").and_then(|v| v.as_str()),
                                preset.get("primary").and_then(|v| v.as_str()),
                                preset.get("secondary").and_then(|v| v.as_str()),
                                preset.get("text").and_then(|v| v.as_str()),
                                preset.get("accent").and_then(|v| v.as_str()),
                            ) {
                                config.background = bg.to_string();
                                config.primary = pr.to_string();
                                config.secondary = sec.to_string();
                                config.text = txt.to_string();
                                config.accent = acc.to_string();
                            }
                        }
                    }
                }
                config
            }
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Use Python script to save (same as quickshell) for compatibility
        self.save_via_python_script()
    }

    fn save_via_python_script(&self) -> Result<(), Box<dyn std::error::Error>> {
        use std::process::Command;
        
        let path = Self::get_config_path();
        let path_str = path.to_string_lossy();
        
        // Try to find Python script: alloy config -> alloy/spark/scripts, then QUICKSHELL_PROJECT_PATH, then sharpshell
        let script_path = {
            let path = Self::get_config_path();
            if let Some(parent) = path.parent() {
                if parent.ends_with("alloy") {
                    let alloy_script = parent.join("spark").join("scripts").join("save-colors.py");
                    if alloy_script.exists() {
                        alloy_script
                    } else if let Ok(project_path) = std::env::var("QUICKSHELL_PROJECT_PATH") {
                        PathBuf::from(project_path).join("scripts").join("save-colors.py")
                    } else if let Some(home) = dirs::home_dir() {
                        home.join(".config").join("sharpshell").join("scripts").join("save-colors.py")
                    } else {
                        return self.save_direct();
                    }
                } else if let Ok(project_path) = std::env::var("QUICKSHELL_PROJECT_PATH") {
                    PathBuf::from(project_path).join("scripts").join("save-colors.py")
                } else if let Some(home) = dirs::home_dir() {
                    home.join(".config").join("sharpshell").join("scripts").join("save-colors.py")
                } else {
                    return self.save_direct();
                }
            } else if let Ok(project_path) = std::env::var("QUICKSHELL_PROJECT_PATH") {
                PathBuf::from(project_path).join("scripts").join("save-colors.py")
            } else if let Some(home) = dirs::home_dir() {
                home.join(".config").join("sharpshell").join("scripts").join("save-colors.py")
            } else {
                return self.save_direct();
            }
        };

        if !script_path.exists() {
            // Fallback to direct save if script doesn't exist
            return self.save_direct();
        }

        let script_str = script_path.to_string_lossy();
        
        // Build command with all arguments
        let mut cmd = Command::new("python3");
        cmd.arg(script_str.as_ref());
        cmd.arg(&self.background);
        cmd.arg(&self.primary);
        cmd.arg(&self.secondary);
        cmd.arg(&self.text);
        cmd.arg(&self.accent);
        cmd.arg(path_str.as_ref());
        
        // Add optional arguments
        if let Some(ref wp) = self.last_wallpaper {
            cmd.arg(wp);
        } else {
            cmd.arg("");
        }
        
        if let Some(ref preset) = self.color_preset {
            cmd.arg(preset);
        } else {
            cmd.arg("");
        }
        
        if let Some(ref pos) = self.sidebar_position {
            cmd.arg(pos);
        } else {
            cmd.arg("");
        }
        
        if let Some(enabled) = self.notifications_enabled {
            cmd.arg(if enabled { "true" } else { "false" });
        } else {
            cmd.arg("");
        }
        
        if let Some(enabled) = self.notification_sounds_enabled {
            cmd.arg(if enabled { "true" } else { "false" });
        } else {
            cmd.arg("");
        }
        
        if let Some(visible) = self.sidebar_visible {
            cmd.arg(if visible { "true" } else { "false" });
        } else {
            cmd.arg("");
        }
        
        if let Some(ref rounding) = self.rounding {
            cmd.arg(rounding);
        } else {
            cmd.arg("");
        }
        
        if let Some(show_hidden) = self.show_hidden_files {
            cmd.arg(if show_hidden { "true" } else { "false" });
        } else {
            cmd.arg("");
        }
        
        if let Some(scale) = self.ui_scale {
            cmd.arg(scale.to_string());
        } else {
            cmd.arg("");
        }
        
        if let Some(ref tile) = self.dashboard_tile_left {
            cmd.arg(tile);
        } else {
            cmd.arg("");
        }
        
        // Argument 17: sidepanelContent ("calendar" or "github")
        if let Some(ref sidepanel) = self.sidepanel_content {
            cmd.arg(sidepanel);
        } else {
            cmd.arg("");
        }
        
        // Argument 18: githubUsername (string)
        if let Some(ref username) = self.github_username {
            cmd.arg(username);
        } else {
            cmd.arg("");
        }

        if let Some(ref pos) = self.dashboard_position {
            cmd.arg(pos);
        } else {
            cmd.arg("");
        }

        // Argument 20: scriptsAutostartBattery
        if let Some(enabled) = self.scripts_autostart_battery {
            cmd.arg(if enabled { "true" } else { "false" });
        } else {
            cmd.arg("");
        }

        // Argument 21: scriptsAutostartScreensaver
        if let Some(enabled) = self.scripts_autostart_screensaver {
            cmd.arg(if enabled { "true" } else { "false" });
        } else {
            cmd.arg("");
        }

        // Argument 22: batteryThreshold
        if let Some(val) = self.battery_threshold {
            cmd.arg(val.to_string());
        } else {
            cmd.arg("");
        }

        // Argument 23: screensaverTimeout
        if let Some(val) = self.screensaver_timeout {
            cmd.arg(val.to_string());
        } else {
            cmd.arg("");
        }

        // Argument 24: dashboardResource1
        if let Some(ref val) = self.dashboard_resource_1 {
            cmd.arg(val);
        } else {
            cmd.arg("");
        }
        
        // Argument 25: dashboardResource2
        if let Some(ref val) = self.dashboard_resource_2 {
            cmd.arg(val);
        } else {
            cmd.arg("");
        }
        
        // Argument 26: scriptsAutostartAutofloat
        if let Some(enabled) = self.scripts_autostart_autofloat {
            cmd.arg(if enabled { "true" } else { "false" });
        } else {
            cmd.arg("");
        }

        // Argument 27: autofloatWidth
        if let Some(val) = self.autofloat_width {
            cmd.arg(val.to_string());
        } else {
            cmd.arg("");
        }

        // Argument 28: autofloatHeight
        if let Some(val) = self.autofloat_height {
            cmd.arg(val.to_string());
        } else {
            cmd.arg("");
        }
        
        // Argument 29: scriptsUseLockscreen
        if let Some(enabled) = self.scripts_use_lockscreen {
            cmd.arg(if enabled { "true" } else { "false" });
        } else {
            cmd.arg("");
        }

        // Argument 30: notificationPosition ("top", "top-left", "top-right")
        if let Some(ref pos) = self.notification_position {
            cmd.arg(pos);
        } else {
            cmd.arg("");
        }

        // Argument 31: notificationRounding ("none", "standard", "pill")
        if let Some(ref rounding) = self.notification_rounding {
            cmd.arg(rounding);
        } else {
            cmd.arg("");
        }

        // Argument 32: quickshellBorderRadius (0=disabled, 2-8 typical)
        if let Some(val) = self.quickshell_border_radius {
            cmd.arg(val.to_string());
        } else {
            cmd.arg("");
        }

        // Argument 33: notificationSound
        if let Some(ref sound) = self.notification_sound {
            cmd.arg(sound);
        } else {
            cmd.arg("");
        }
        
        let output = cmd.output()?;
        if !output.status.success() {
            // Fallback to direct save on error
            return self.save_direct();
        }
        
        // Ensure file is synced to disk
        use std::fs::OpenOptions;
        if let Ok(file) = OpenOptions::new().write(true).open(&path) {
            file.sync_all().ok();
        }
        
        Ok(())
    }

    fn save_direct(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_config_path();
        
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&path, json)?;
        Ok(())
    }

    pub fn update_colors(&mut self, background: &str, primary: &str, secondary: &str, text: &str, accent: &str) {
        self.background = background.to_string();
        self.primary = primary.to_string();
        self.secondary = secondary.to_string();
        self.text = text.to_string();
        self.accent = accent.to_string();
    }

    pub fn set_wallpaper(&mut self, wallpaper_path: &str) {
        self.last_wallpaper = Some(wallpaper_path.to_string());
    }

    pub fn set_preset(&mut self, preset_name: &str) {
        self.color_preset = Some(preset_name.to_string());
    }

    pub fn set_sidebar_position(&mut self, position: &str) {
        self.sidebar_position = Some(position.to_string());
    }

    pub fn set_notifications_enabled(&mut self, enabled: bool) {
        self.notifications_enabled = Some(enabled);
    }

    pub fn set_notification_sounds_enabled(&mut self, enabled: bool) {
        self.notification_sounds_enabled = Some(enabled);
    }

    pub fn set_sidebar_visible(&mut self, visible: bool) {
        self.sidebar_visible = Some(visible);
    }

    pub fn set_rounding(&mut self, rounding: &str) {
        self.rounding = Some(rounding.to_string());
    }

    pub fn set_show_hidden_files(&mut self, show_hidden: bool) {
        self.show_hidden_files = Some(show_hidden);
    }

    pub fn set_ui_scale(&mut self, value: u8) {
        self.ui_scale = Some(value);
    }

    pub fn set_dashboard_tile_left(&mut self, value: &str) {
        self.dashboard_tile_left = Some(value.to_string());
    }

    pub fn set_sidepanel_content(&mut self, value: &str) {
        self.sidepanel_content = Some(value.to_string());
    }

    pub fn set_github_username(&mut self, value: &str) {
        self.github_username = if value.is_empty() {
            None
        } else {
            Some(value.to_string())
        };
    }

    pub fn set_dashboard_position(&mut self, value: &str) {
        self.dashboard_position = Some(value.to_string());
    }

    pub fn set_scripts_autostart_battery(&mut self, enabled: bool) {
        self.scripts_autostart_battery = Some(enabled);
    }

    pub fn set_scripts_autostart_screensaver(&mut self, enabled: bool) {
        self.scripts_autostart_screensaver = Some(enabled);
    }

    pub fn set_battery_threshold(&mut self, value: u8) {
        self.battery_threshold = Some(value);
    }

    pub fn set_screensaver_timeout(&mut self, value: u32) {
        self.screensaver_timeout = Some(value);
    }

    pub fn set_dashboard_resource_1(&mut self, value: &str) {
        self.dashboard_resource_1 = Some(value.to_string());
    }

    pub fn set_dashboard_resource_2(&mut self, value: &str) {
        self.dashboard_resource_2 = Some(value.to_string());
    }

    pub fn set_scripts_autostart_autofloat(&mut self, enabled: bool) {
        self.scripts_autostart_autofloat = Some(enabled);
    }

    pub fn set_autofloat_width(&mut self, value: u32) {
        self.autofloat_width = Some(value);
    }

    pub fn set_autofloat_height(&mut self, value: u32) {
        self.autofloat_height = Some(value);
    }

    pub fn set_scripts_use_lockscreen(&mut self, enabled: bool) {
        self.scripts_use_lockscreen = Some(enabled);
    }

    pub fn set_notification_position(&mut self, position: &str) {
        self.notification_position = Some(position.to_string());
    }

    pub fn set_notification_rounding(&mut self, rounding: &str) {
        self.notification_rounding = Some(rounding.to_string());
    }

    pub fn set_quickshell_border_radius(&mut self, value: u8) {
        self.quickshell_border_radius = Some(value);
    }

    pub fn set_notification_sound(&mut self, sound: &str) {
        self.notification_sound = Some(sound.to_string());
    }

    /// Set GTK_SCALE_FACTOR from ui_scale (75 -> 0.75, 100 -> 1.0, 125 -> 1.25). Call before gtk_init.
    pub fn apply_scale_env_to_process() {
        let config = Self::load();
        Self::apply_scale_env_from_config(&config);
    }

    /// Set GTK_SCALE_FACTOR from existing config. Use when config is already loaded.
    pub fn apply_scale_env_from_config(config: &Self) {
        if let Some(scale) = config.ui_scale {
            let factor = match scale {
                75 => "0.75",
                100 => "1.0",
                125 => "1.25",
                _ => "1.0",
            };
            std::env::set_var("GTK_SCALE_FACTOR", factor);
        }
    }
}
