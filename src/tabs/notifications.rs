use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Switch, ScrolledWindow};
use std::sync::{Arc, Mutex};

use crate::core::config::ColorConfig;

pub struct NotificationsTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl NotificationsTab {
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
        let title = Label::new(Some("Notification Settings"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // Notifications section
        let notifications_section = create_notifications_section(Arc::clone(&config));
        notifications_section.set_hexpand(true);
        content.append(&notifications_section);

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

fn create_notifications_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 12);
    section.add_css_class("content-section");

    let section_title = Label::new(Some("Notifications"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section.append(&section_title);

    let section_description = Label::new(Some("Configure notification display and sound settings"));
    section_description.add_css_class("section-description");
    section_description.set_xalign(0.0);
    section_description.set_wrap(true);
    section.append(&section_description);

    // Show Notifications toggle
    let notifications_row = create_toggle_row(
        "Show Notifications",
        "Enable or disable notification display",
        {
            let config = Arc::clone(&config);
            move |enabled| {
                let mut cfg = config.lock().unwrap();
                cfg.set_notifications_enabled(enabled);
                let _ = cfg.save();
            }
        },
        {
            let cfg = config.lock().unwrap();
            cfg.notifications_enabled.unwrap_or(true)
        },
    );
    section.append(&notifications_row);

    // Notification Sounds toggle
    let sounds_row = create_toggle_row(
        "Notification Sounds",
        "Play sound when notification arrives",
        {
            let config_clone = Arc::clone(&config);
            move |enabled| {
                let mut cfg = config_clone.lock().unwrap();
                cfg.set_notification_sounds_enabled(enabled);
                let _ = cfg.save();
            }
        },
        {
            let cfg = config.lock().unwrap();
            cfg.notification_sounds_enabled.unwrap_or(true)
        },
    );
    section.append(&sounds_row);

    // Notification Position section
    let position_row = create_position_row(Arc::clone(&config));
    section.append(&position_row);

    // Notification Rounding section
    let rounding_row = create_rounding_row(Arc::clone(&config));
    section.append(&rounding_row);

    // Notification Sound section
    let sound_row = create_sound_row(Arc::clone(&config));
    section.append(&sound_row);

    section
}

fn create_rounding_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_hexpand(true);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let title_label = Label::new(Some("Notification Rounding"));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    text_box.append(&title_label);

    let desc_label = Label::new(Some("Choose the corner rounding style for notifications"));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    text_box.append(&desc_label);

    row.append(&text_box);

    let button_box = GtkBox::new(Orientation::Horizontal, 6);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_valign(gtk4::Align::Center);

    let current_rounding = config.lock().unwrap().notification_rounding.clone().unwrap_or_else(|| "standard".to_string());

    let styles = [
        ("None", "none"),
        ("Standard", "standard"),
        ("Capsule", "pill"),
    ];

    let mut buttons = Vec::new();

    for (label, value) in styles {
        let btn = gtk4::Button::with_label(label);
        if current_rounding == value {
            btn.add_css_class("suggested-action");
        }
        buttons.push((btn.clone(), value.to_string()));
        button_box.append(&btn);
    }

    for (btn, value) in buttons.clone() {
        let config = Arc::clone(&config);
        let value_clone = value.to_string();
        let buttons_clone = buttons.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_notification_rounding(&value_clone);
            if let Err(_) = cfg.save() {
            } else {
                *config.lock().unwrap() = cfg;

                // Update UI visually
                for (b, v) in buttons_clone.iter() {
                    if v == &value_clone {
                        b.add_css_class("suggested-action");
                    } else {
                        b.remove_css_class("suggested-action");
                    }
                }

                gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    let _ = crate::core::quickshell::notify_color_change();
                    gtk4::glib::ControlFlow::Break
                });
            }
        });
    }

    row.append(&button_box);
    row
}

fn create_position_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_hexpand(true);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let title_label = Label::new(Some("Notification Position"));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    text_box.append(&title_label);

    let desc_label = Label::new(Some("Choose where notifications appear on the screen"));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    text_box.append(&desc_label);

    row.append(&text_box);

    let button_box = GtkBox::new(Orientation::Horizontal, 6);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_valign(gtk4::Align::Center);

    let current_pos = config.lock().unwrap().notification_position.clone().unwrap_or_else(|| "top".to_string());

    let positions = [
        ("Top Left", "top-left"),
        ("Top", "top"),
        ("Top Right", "top-right"),
    ];

    let mut buttons = Vec::new();

    for (label, value) in positions {
        let btn = gtk4::Button::with_label(label);
        if current_pos == value {
            btn.add_css_class("suggested-action");
        }
        buttons.push((btn.clone(), value.to_string()));
        button_box.append(&btn);
    }

    for (btn, value) in buttons.clone() {
        let config = Arc::clone(&config);
        let value_clone = value.clone();
        let buttons_clone = buttons.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_notification_position(&value_clone);
            if let Err(_) = cfg.save() {
                // Handle error
            } else {
                *config.lock().unwrap() = cfg;
                
                // Update UI visually
                for (b, v) in buttons_clone.iter() {
                    if v == &value_clone {
                        b.add_css_class("suggested-action");
                    } else {
                        b.remove_css_class("suggested-action");
                    }
                }

                // Notify quickshell after a short delay
                gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    let _ = crate::core::quickshell::notify_color_change();
                    gtk4::glib::ControlFlow::Break
                });
            }
        });
    }

    row.append(&button_box);
    row
}

fn create_sound_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_hexpand(true);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let title_label = Label::new(Some("Notification Sound"));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    text_box.append(&title_label);

    let desc_label = Label::new(Some("Choose the sound for new notifications"));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    text_box.append(&desc_label);

    row.append(&text_box);

    let button_box = GtkBox::new(Orientation::Horizontal, 6);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_valign(gtk4::Align::Center);

    let current_sound = config.lock().unwrap().notification_sound.clone().unwrap_or_else(|| "message.oga".to_string());

    let sounds = [
        ("Default", "message.oga"),
        ("Glass", "bell.oga"),
        ("Crystal", "complete.oga"),
        ("Sonar", "message-new-instant.oga"),
        ("Pop", "audio-volume-change.oga"),
    ];

    let mut buttons = Vec::new();

    for (label, value) in sounds {
        let btn = gtk4::Button::with_label(label);
        if current_sound == value {
            btn.add_css_class("suggested-action");
        }
        buttons.push((btn.clone(), value.to_string()));
        button_box.append(&btn);
    }

    for (btn, value) in buttons.clone() {
        let config = Arc::clone(&config);
        let value_clone = value.clone();
        let buttons_clone = buttons.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_notification_sound(&value_clone);
            if let Err(_) = cfg.save() {
                // Handle error
            } else {
                *config.lock().unwrap() = cfg;
                
                // Update UI visually
                for (b, v) in buttons_clone.iter() {
                    if v == &value_clone {
                        b.add_css_class("suggested-action");
                    } else {
                        b.remove_css_class("suggested-action");
                    }
                }

                // Play a preview
                let preview_sound = value_clone.clone();
                std::process::Command::new("paplay")
                    .arg(format!("/usr/share/sounds/freedesktop/stereo/{}", preview_sound))
                    .spawn()
                    .ok();

                // Notify quickshell after a short delay
                gtk4::glib::timeout_add_local(std::time::Duration::from_millis(100), move || {
                    let _ = crate::core::quickshell::notify_color_change();
                    gtk4::glib::ControlFlow::Break
                });
            }
        });
    }

    row.append(&button_box);
    row
}

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
    row.set_can_focus(false);

    // GNOME: 2px gap between title and description
    let text_box = GtkBox::new(Orientation::Vertical, 2);
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

    let toggle = Switch::new();
    toggle.set_active(initial_value);
    toggle.set_halign(gtk4::Align::End);
    toggle.set_valign(gtk4::Align::Center);
    toggle.set_hexpand(false);
    toggle.set_vexpand(false);
    toggle.set_sensitive(true);
    toggle.set_can_focus(true);
    toggle.set_focusable(true);
    toggle.connect_active_notify(move |toggle| {
        on_toggle(toggle.is_active());
    });
    row.append(&toggle);

    row
}
