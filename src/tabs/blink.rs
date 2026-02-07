use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Switch, ScrolledWindow, Entry, CheckButton};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use dirs;

use crate::core::config::ColorConfig;

pub struct BlinkTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl BlinkTab {
    pub fn new(config: Arc<Mutex<ColorConfig>>) -> Self {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_overlay_scrolling(false);
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        // GNOME spacing: 24px section gap, 12px container margins
        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        // Title
        let title = Label::new(Some("Blink Settings"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // File Manager section
        let file_manager_section = create_file_manager_section(Arc::clone(&config));
        file_manager_section.set_hexpand(true);
        content.append(&file_manager_section);

        // Keyboard Shortcuts section
        let keybinds_section = create_keybinds_section();
        keybinds_section.set_hexpand(true);
        content.append(&keybinds_section);

        scrolled.set_child(Some(&content));

        Self {
            widget: scrolled,
            _config: config,
        }
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}

fn create_file_manager_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 16);
    section.add_css_class("content-section");

    let section_title = Label::new(Some("File Manager"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section.append(&section_title);

    // Show Hidden Files toggle
    let current_show_hidden = config.lock().unwrap().show_hidden_files.unwrap_or(false);
    let hidden_files_toggle = Switch::new();
    hidden_files_toggle.set_active(current_show_hidden);
    hidden_files_toggle.set_halign(gtk4::Align::End);
    hidden_files_toggle.set_valign(gtk4::Align::Center);
    hidden_files_toggle.set_hexpand(false);
    hidden_files_toggle.set_vexpand(false);
    
    {
        let config = Arc::clone(&config);
        let hidden_files_toggle_clone = hidden_files_toggle.clone();
        hidden_files_toggle.connect_active_notify(move |toggle| {
            let enabled = toggle.is_active();
            // Reload config from disk to preserve existing settings
            let mut cfg = ColorConfig::load();
            cfg.set_show_hidden_files(enabled);
            if let Err(_e) = cfg.save() {
                // Revert the toggle state on error
                hidden_files_toggle_clone.set_active(!enabled);
            } else {
                // Update the shared config
                *config.lock().unwrap() = cfg.clone();
            }
        });
    }
    
    let hidden_files_row = create_toggle_row_with_switch(
        "Show Hidden Files",
        "Display hidden files and folders",
        hidden_files_toggle,
    );
    section.append(&hidden_files_row);

    section
}

#[allow(dead_code)]
fn create_toggle_row(
    title: &str,
    description: &str,
    on_toggle: impl Fn(bool) + 'static,
    initial_value: bool,
) -> GtkBox {
    // GNOME spacing: 12px internal spacing
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(0);
    row.set_margin_end(0);
    row.set_margin_top(0);
    row.set_margin_bottom(0);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);

    // GNOME: 2px gap between title and description
    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk4::Align::Fill);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    text_box.append(&title_label);

    let desc_label = Label::new(Some(description));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    desc_label.set_halign(gtk4::Align::Start);
    text_box.append(&desc_label);

    row.append(&text_box);

    let toggle = Switch::new();
    toggle.set_active(initial_value);
    toggle.set_halign(gtk4::Align::End);
    toggle.set_valign(gtk4::Align::Center);
    toggle.set_hexpand(false);
    toggle.set_vexpand(false);
    toggle.connect_active_notify(move |toggle| {
        on_toggle(toggle.is_active());
    });
    row.append(&toggle);

    row
}

fn create_toggle_row_with_switch(
    title: &str,
    description: &str,
    toggle: Switch,
) -> GtkBox {
    // GNOME spacing: 12px internal spacing
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(0);
    row.set_margin_end(0);
    row.set_margin_top(0);
    row.set_margin_bottom(0);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);

    // GNOME: 2px gap between title and description
    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk4::Align::Fill);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    text_box.append(&title_label);

    let desc_label = Label::new(Some(description));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    desc_label.set_halign(gtk4::Align::Start);
    text_box.append(&desc_label);

    row.append(&text_box);
    row.append(&toggle);

    row
}

// Keybind management functions
fn keybinds_config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("blink")
        .join("keybinds.conf")
}

fn load_keybinds() -> HashMap<String, (String, Vec<String>)> {
    let config_path = keybinds_config_path();
    let mut keybinds: HashMap<String, (String, Vec<String>)> = HashMap::new();
    
    // Default keybinds
    let defaults = vec![
        ("toggle_hidden", ("h".to_string(), vec!["Control".to_string()])),
        ("open_terminal", ("h".to_string(), vec![])),
        ("select_all", ("a".to_string(), vec!["Control".to_string()])),
        ("refresh", ("F5".to_string(), vec![])),
        ("open_with_micro", ("m".to_string(), vec![])),
        ("back", ("Mouse8".to_string(), vec![])),
        ("forward", ("Mouse9".to_string(), vec![])),
        ("up", ("Up".to_string(), vec![])),
        ("home", ("Home".to_string(), vec![])),
        ("copy", ("c".to_string(), vec!["Control".to_string()])),
        ("cut", ("x".to_string(), vec!["Control".to_string()])),
        ("paste", ("v".to_string(), vec!["Control".to_string()])),
        ("delete", ("Delete".to_string(), vec![])),
        ("rename", ("F2".to_string(), vec![])),
    ];
    
    for (action, (key, mods)) in defaults {
        keybinds.insert(action.to_string(), (key, mods));
    }
    
    // Load from file if exists
    if config_path.exists() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                
                if let Some((action_str, keybind_str)) = trimmed.split_once('=') {
                    let action_str = action_str.trim();
                    let keybind_str = keybind_str.trim();
                    
                    if let Some((key, mods_str)) = keybind_str.split_once(':') {
                        let key = key.trim().to_string();
                        let modifiers: Vec<String> = if mods_str.trim().is_empty() {
                            vec![]
                        } else {
                            mods_str.split(',').map(|s| s.trim().to_string()).collect()
                        };
                        keybinds.insert(action_str.to_string(), (key, modifiers));
                    } else {
                        keybinds.insert(action_str.to_string(), (keybind_str.to_string(), vec![]));
                    }
                }
            }
        }
    }
    
    keybinds
}

fn save_keybinds(keybinds: &HashMap<String, (String, Vec<String>)>) -> Result<(), std::io::Error> {
    let config_path = keybinds_config_path();
    
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let mut content = String::from("# Blink keybinds configuration\n");
    content.push_str("# Format: action=key:modifier1,modifier2\n");
    content.push_str("# Modifiers: Control, Shift, Alt, Super\n");
    content.push_str("# Special keys: F1-F12, Up, Down, Left, Right, Home, End, Delete, etc.\n");
    content.push_str("# Mouse buttons: Mouse8, Mouse9\n\n");
    
    let actions = vec![
        "toggle_hidden", "open_terminal", "select_all", "refresh",
        "open_with_micro", "back", "forward", "up", "home",
        "copy", "cut", "paste", "delete", "rename",
    ];
    
    for action in actions {
        if let Some((key, mods)) = keybinds.get(action) {
            let mods_str = if mods.is_empty() {
                String::new()
            } else {
                format!(":{}", mods.join(","))
            };
            content.push_str(&format!("{}={}{}\n", action, key, mods_str));
        } else {
        }
    }
    
    fs::write(&config_path, content)?;
    Ok(())
}

fn action_display_name(action: &str) -> &'static str {
    match action {
        "toggle_hidden" => "Toggle Hidden Files",
        "open_terminal" => "Open Terminal",
        "select_all" => "Select All",
        "refresh" => "Refresh",
        "open_with_micro" => "Open with Micro",
        "back" => "Back",
        "forward" => "Forward",
        "up" => "Up (Parent)",
        "home" => "Home",
        "copy" => "Copy",
        "cut" => "Cut",
        "paste" => "Paste",
        "delete" => "Delete",
        "rename" => "Rename",
        _ => "Unknown Action",
    }
}

fn create_keybinds_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 12);
    section.add_css_class("keybind-section");
    
    let section_title = Label::new(Some("Keyboard Shortcuts"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section.append(&section_title);
    
    let keybinds = load_keybinds();
    let keybinds_arc = Arc::new(Mutex::new(keybinds));
    
    let actions = vec![
        "toggle_hidden", "open_terminal", "select_all", "refresh",
        "open_with_micro", "back", "forward", "up", "home",
        "copy", "cut", "paste", "delete", "rename",
    ];
    
    for action in actions {
        let row = create_keybind_row(action, Arc::clone(&keybinds_arc));
        section.append(&row);
    }
    
    section
}

fn create_keybind_row(action: &str, keybinds: Arc<Mutex<HashMap<String, (String, Vec<String>)>>>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.add_css_class("keybind-row");
    row.set_margin_start(0);
    row.set_margin_end(0);
    row.set_margin_top(4);
    row.set_margin_bottom(4);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);
    
    // Title: narrower min than before so keybind rows fit on small windows
    let title_label = Label::new(Some(action_display_name(action)));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    title_label.set_size_request(80, -1);
    row.append(&title_label);
    
    // Key entry
    let keybinds_clone = Arc::clone(&keybinds);
    let action_str = action.to_string();
    let (current_key, current_mods) = keybinds.lock().unwrap()
        .get(action)
        .cloned()
        .unwrap_or_else(|| ("".to_string(), vec![]));
    
    let key_entry = Entry::new();
    key_entry.set_text(&current_key);
    key_entry.set_placeholder_text(Some("Key"));
    key_entry.set_hexpand(true);
    
    {
        let keybinds = Arc::clone(&keybinds_clone);
        let action_str = action_str.clone();
        key_entry.connect_changed(move |entry| {
            let key = entry.text().to_string();
            let mut kb = keybinds.lock().unwrap();
            if let Some((_, mods)) = kb.get(&action_str) {
                let mods_clone = mods.clone();
                kb.insert(action_str.clone(), (key, mods_clone));
            } else {
                kb.insert(action_str.clone(), (key, vec![]));
            }
            if let Err(_e) = save_keybinds(&kb) {
            } else {
            }
        });
    }
    row.append(&key_entry);
    
    // Modifiers checkboxes
    let mods_box = GtkBox::new(Orientation::Horizontal, 8);
    mods_box.add_css_class("modifiers-box");
    let modifiers = vec!["Control", "Shift", "Alt", "Super"];
    let mut checkboxes = Vec::new();
    
    for mod_name in modifiers {
        let is_active = current_mods.contains(&mod_name.to_string());
        let checkbox = CheckButton::with_label(mod_name);
        checkbox.set_active(is_active);
        
        {
            let keybinds = Arc::clone(&keybinds_clone);
            let action_str = action_str.clone();
            let mod_name = mod_name.to_string();
            checkbox.connect_toggled(move |cb| {
                let mut kb = keybinds.lock().unwrap();
                let (key, mods) = kb.get(&action_str)
                    .map(|(k, m)| (k.clone(), m.clone()))
                    .unwrap_or_else(|| ("".to_string(), vec![]));
                
                let mut new_mods = mods;
                if cb.is_active() {
                    if !new_mods.contains(&mod_name) {
                        new_mods.push(mod_name.clone());
                    }
                } else {
                    new_mods.retain(|m| m != &mod_name);
                }
                kb.insert(action_str.clone(), (key, new_mods));
                
                if let Err(_e) = save_keybinds(&kb) {
                } else {
                }
            });
        }
        
        checkboxes.push(checkbox.clone());
        mods_box.append(&checkbox);
    }
    
    row.append(&mods_box);
    
    row
}
