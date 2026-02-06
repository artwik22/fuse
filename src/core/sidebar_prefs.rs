use std::path::PathBuf;
use dirs;

#[allow(dead_code)]
pub struct SidebarPrefs;

impl SidebarPrefs {
    #[allow(dead_code)]
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("index")
            .join(".sidebar_prefs")
    }
}
