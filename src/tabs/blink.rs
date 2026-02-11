use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Switch, ScrolledWindow, Entry, CheckButton};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::fs;

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
        
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(48);

        // Title
        let title = Label::new(Some("Blink File Manager Settings"));
        title.add_css_class("title");
        title.set_halign(gtk4::Align::Start);
        title.set_margin_bottom(24);
        content.append(&title);

        let add_group_header = |box_: &GtkBox, label: &str| {
            let l = Label::new(Some(label));
            l.add_css_class("group-header");
            l.set_halign(gtk4::Align::Start);
            box_.append(&l);
        };

        // --- Behavior Group ---
        add_group_header(&content, "Behavior");
        let behavior_card = GtkBox::new(Orientation::Vertical, 0);
        behavior_card.add_css_class("card");
        
        behavior_card.append(&create_hidden_files_row(Arc::clone(&config)));
        content.append(&behavior_card);

        // --- Keybinds Group ---
        add_group_header(&content, "Keyboard Shortcuts");
        let keybinds_card = GtkBox::new(Orientation::Vertical, 0);
        keybinds_card.add_css_class("card");

        let keybinds = load_keybinds();
        let keybinds_arc = Arc::new(Mutex::new(keybinds));
        let actions = vec![
            "toggle_hidden", "open_terminal", "select_all", "refresh",
            "open_with_micro", "copy", "cut", "paste", "delete", "rename",
        ];
        
        for action in actions {
            keybinds_card.append(&create_keybind_row(action, Arc::clone(&keybinds_arc)));
        }

        content.append(&keybinds_card);

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

fn create_card_row(label: &str, widget: impl IsA<gtk4::Widget>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    row.set_valign(gtk4::Align::Center);

    let l = Label::new(Some(label));
    l.add_css_class("row-title");
    l.set_hexpand(true);
    l.set_halign(gtk4::Align::Start);
    
    row.append(&l);
    row.append(&widget);
    row
}

fn create_hidden_files_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let sw = Switch::new();
    let current = config.lock().unwrap().show_hidden_files.unwrap_or(false);
    sw.set_active(current);
    
    {
        let config = Arc::clone(&config);
        sw.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            cfg.set_show_hidden_files(s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg;
            }
        });
    }

    create_card_row("Show Hidden Files", sw)
}

fn create_keybind_row(action: &str, keybinds: Arc<Mutex<HashMap<String, (String, Vec<String>)>>>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    
    let label = Label::new(Some(action_display_name(action)));
    label.add_css_class("row-title");
    label.set_halign(gtk4::Align::Start);
    label.set_width_request(140);
    row.append(&label);

    let (key, mods) = keybinds.lock().unwrap().get(action).cloned().unwrap_or(("".into(), vec![]));
    
    let entry = Entry::new();
    entry.set_text(&key);
    entry.set_width_chars(5);
    entry.set_hexpand(false);
    
    {
        let keybinds = Arc::clone(&keybinds);
        let action = action.to_string();
        entry.connect_changed(move |e| {
            let mut kb = keybinds.lock().unwrap();
            let (_, m) = kb.get(&action).cloned().unwrap_or(("".into(), vec![]));
            kb.insert(action.clone(), (e.text().to_string(), m));
            save_keybinds(&kb).ok();
        });
    }
    row.append(&entry);

    let mods_box = GtkBox::new(Orientation::Horizontal, 8);
    for m in &["Control", "Shift", "Alt"] {
        let cb = CheckButton::with_label(m);
        cb.set_active(mods.contains(&m.to_string()));
        {
            let keybinds = Arc::clone(&keybinds);
            let action = action.to_string();
            let m = m.to_string();
            cb.connect_toggled(move |c| {
                let mut kb = keybinds.lock().unwrap();
                let (k, mut ms) = kb.get(&action).cloned().unwrap_or(("".into(), vec![]));
                if c.is_active() { if !ms.contains(&m) { ms.push(m.clone()); } }
                else { ms.retain(|x| x != &m); }
                kb.insert(action.clone(), (k, ms));
                save_keybinds(&kb).ok();
            });
        }
        mods_box.append(&cb);
    }
    row.append(&mods_box);

    row
}

fn action_display_name(action: &str) -> &'static str {
    match action {
        "toggle_hidden" => "Toggle Hidden",
        "open_terminal" => "Terminal",
        "select_all" => "Select All",
        "refresh" => "Refresh",
        "open_with_micro" => "Edit (Micro)",
        "copy" => "Copy",
        "cut" => "Cut",
        "paste" => "Paste",
        "delete" => "Delete",
        "rename" => "Rename",
        _ => "Unknown",
    }
}

fn load_keybinds() -> HashMap<String, (String, Vec<String>)> {
    let mut kb = HashMap::new();
    let path = dirs::config_dir().unwrap().join("blink/keybinds.conf");
    if let Ok(c) = fs::read_to_string(path) {
        for l in c.lines() {
            if let Some((a, k_m)) = l.split_once('=') {
                if let Some((k, m)) = k_m.split_once(':') {
                    kb.insert(a.trim().to_string(), (k.trim().to_string(), m.split(',').map(|s| s.trim().to_string()).collect()));
                } else {
                    kb.insert(a.trim().to_string(), (k_m.trim().to_string(), vec![]));
                }
            }
        }
    }
    kb
}

fn save_keybinds(kb: &HashMap<String, (String, Vec<String>)>) -> Result<(), std::io::Error> {
    let path = dirs::config_dir().unwrap().join("blink/keybinds.conf");
    fs::create_dir_all(path.parent().unwrap())?;
    let mut c = String::new();
    for (a, (k, m)) in kb {
        let ms = if m.is_empty() { "".into() } else { format!(":{}", m.join(",")) };
        c.push_str(&format!("{}={}{}\n", a, k, ms));
    }
    fs::write(path, c)
}
