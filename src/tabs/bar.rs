use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Switch, ScrolledWindow, Button};
use std::sync::{Arc, Mutex};

use crate::core::config::ColorConfig;
use crate::core::quickshell;

pub struct BarTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl BarTab {
    pub fn new(config: Arc<Mutex<ColorConfig>>) -> Self {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_overlay_scrolling(false);
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let content = GtkBox::new(Orientation::Vertical, 18);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        let title = Label::new(Some("Bar Settings"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        content.append(&title);

        // Sidebar Visibility toggle
        let current_visible = config.lock().unwrap().sidebar_visible.unwrap_or(true);
        let toggle_switch = Switch::new();
        toggle_switch.set_active(current_visible);
        toggle_switch.set_halign(gtk4::Align::End);
        toggle_switch.set_hexpand(false);
        toggle_switch.set_valign(gtk4::Align::Center);
        toggle_switch.set_vexpand(false);
        toggle_switch.set_sensitive(true);
        toggle_switch.set_can_focus(true);
        toggle_switch.set_focusable(true);
        
        {
            let config = Arc::clone(&config);
            let toggle_switch_clone = toggle_switch.clone();
            toggle_switch.connect_active_notify(move |toggle| {
                let enabled = toggle.is_active();
                // Reload config from disk to preserve existing settings
                let mut cfg = ColorConfig::load();
                cfg.set_sidebar_visible(enabled);
                if let Err(e) = cfg.save() {
                    // Revert the toggle state on error
                    toggle_switch_clone.set_active(!enabled);
                } else {
                    // Update the shared config
                    *config.lock().unwrap() = cfg.clone();
                    // Notify quickshell about change after a short delay
                    let config_clone = Arc::clone(&config);
                    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                        // Notify quickshell about change
                        if let Err(e) = quickshell::notify_color_change() {
                        }
                        gtk4::glib::ControlFlow::Break
                    });
                }
            });
        }
        
        let sidebar_visible_row = create_toggle_row_with_switch(
            "Sidebar Visibility",
            "Show or hide the sidebar",
            toggle_switch,
        );
        content.append(&sidebar_visible_row);

        // Sidebar Position
        let position_section = create_sidebar_position_section(Arc::clone(&config));
        content.append(&position_section);

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

fn create_toggle_row(
    title: &str,
    description: &str,
    on_toggle: impl Fn(bool) + 'static,
    initial_value: bool,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 15);
    row.add_css_class("settings-row");
    row.set_margin_start(16);
    row.set_margin_end(16);
    row.set_margin_top(16);
    row.set_margin_bottom(16);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);

    let text_box = GtkBox::new(Orientation::Vertical, 4);
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
    let row = GtkBox::new(Orientation::Horizontal, 15);
    row.add_css_class("settings-row");
    row.set_margin_start(16);
    row.set_margin_end(16);
    row.set_margin_top(16);
    row.set_margin_bottom(16);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);
    row.set_can_focus(false);

    let text_box = GtkBox::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk4::Align::Fill);
    text_box.set_can_focus(false);

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

fn create_sidebar_position_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 12);
    section.add_css_class("settings-row");
    section.set_margin_start(0);
    section.set_margin_end(0);
    section.set_margin_top(12);
    section.set_margin_bottom(12);

    let header = GtkBox::new(Orientation::Horizontal, 15);
    
    let icon = Label::new(Some("󰍇"));
    icon.set_margin_end(12);
    header.append(&icon);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let title = Label::new(Some("Sidebar Position"));
    title.add_css_class("row-title");
    title.set_xalign(0.0);
    text_box.append(&title);

    let desc = Label::new(Some("Choose sidebar position: Left or Top"));
    desc.add_css_class("row-description");
    desc.set_xalign(0.0);
    text_box.append(&desc);

    header.append(&text_box);

    let button_box = GtkBox::new(Orientation::Horizontal, 10);
    
    let current_pos = config.lock().unwrap().sidebar_position.clone().unwrap_or_else(|| "left".to_string());
    let is_left = current_pos == "left";
    let is_top = current_pos == "top";

    let left_button = Button::with_label("Left");
    if is_left {
        left_button.add_css_class("suggested-action");
    }
    {
        let config = Arc::clone(&config);
        left_button.connect_clicked(move |_| {
            // Reload config from disk to preserve existing settings (like color preset)
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_position("left");
            if let Err(e) = cfg.save() {
            } else {
                // Update the shared config
                *config.lock().unwrap() = cfg.clone();
                // Wait a bit for file to be written and synced to disk
                std::thread::sleep(std::time::Duration::from_millis(200));
                if let Err(e) = quickshell::notify_color_change() {
                }
            }
        });
    }
    button_box.append(&left_button);

    let top_button = Button::with_label("Top");
    if is_top {
        top_button.add_css_class("suggested-action");
    }
    {
        let config = Arc::clone(&config);
        top_button.connect_clicked(move |_| {
            // Reload config from disk to preserve existing settings (like color preset)
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_position("top");
            if let Err(e) = cfg.save() {
            } else {
                // Update the shared config
                *config.lock().unwrap() = cfg.clone();
                // Wait a bit for file to be written and synced to disk
                std::thread::sleep(std::time::Duration::from_millis(200));
                if let Err(e) = quickshell::notify_color_change() {
                }
            }
        });
    }
    button_box.append(&top_button);

    header.append(&button_box);
    section.append(&header);

    section
}
