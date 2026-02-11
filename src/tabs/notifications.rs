use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, Switch, ScrolledWindow, Button};
use std::sync::{Arc, Mutex};

use crate::core::config::ColorConfig;
use crate::core::quickshell;

fn schedule_notify_color_change_ms(ms: u32) {
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
        let _ = quickshell::notify_color_change();
        gtk4::glib::ControlFlow::Break
    });
}

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
        
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(48);
        content.set_hexpand(true);
        content.set_vexpand(true);

        // Title
        let title = Label::new(Some("Notifications Settings"));
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
        
        behavior_card.append(&create_toggle_row(
            "Show Notifications",
            Arc::clone(&config),
            |cfg, val| cfg.set_notifications_enabled(val),
            |cfg| cfg.notifications_enabled.unwrap_or(true)
        ));

        behavior_card.append(&create_toggle_row(
            "Notification Sounds",
            Arc::clone(&config),
            |cfg, val| cfg.set_notification_sounds_enabled(val),
            |cfg| cfg.notification_sounds_enabled.unwrap_or(true)
        ));

        content.append(&behavior_card);

        // --- Layout Group ---
        add_group_header(&content, "Layout & Style");
        let layout_card = GtkBox::new(Orientation::Vertical, 0);
        layout_card.add_css_class("card");

        layout_card.append(&create_position_row(Arc::clone(&config)));
        layout_card.append(&create_rounding_row(Arc::clone(&config)));
        layout_card.append(&create_sound_selector_row(Arc::clone(&config)));

        content.append(&layout_card);

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

fn create_toggle_row(
    label: &str, 
    config: Arc<Mutex<ColorConfig>>, 
    setter: fn(&mut ColorConfig, bool),
    getter: fn(&ColorConfig) -> bool
) -> GtkBox {
    let switch = Switch::new();
    let current = getter(&config.lock().unwrap());
    switch.set_active(current);
    switch.set_valign(gtk4::Align::Center);

    {
        let config = config.clone();
        switch.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            setter(&mut cfg, s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row(label, switch)
}

fn create_position_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().notification_position.clone().unwrap_or_else(|| "top".to_string());

    let positions = [
        ("Top Left", "top-left"),
        ("Top", "top"),
        ("Top Right", "top-right"),
    ];

    let mut buttons = Vec::new();

    for (label, value) in positions {
        let btn = Button::with_label(label);
        if current == value {
            btn.add_css_class("suggested-action");
        }
        buttons.push((btn.clone(), value.to_string()));
        box_.append(&btn);
    }

    for (btn, value) in buttons.clone() {
        let config = Arc::clone(&config);
        let value_clone = value.clone();
        let buttons_clone = buttons.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_notification_position(&value_clone);
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg;
                for (b, v) in buttons_clone.iter() {
                    if v == &value_clone { b.add_css_class("suggested-action"); }
                    else { b.remove_css_class("suggested-action"); }
                }
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Position", box_)
}

fn create_rounding_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().notification_rounding.clone().unwrap_or_else(|| "standard".to_string());

    let styles = [
        ("None", "none"),
        ("Standard", "standard"),
        ("Capsule", "pill"),
    ];

    let mut buttons = Vec::new();

    for (label, value) in styles {
        let btn = Button::with_label(label);
        if current == value {
            btn.add_css_class("suggested-action");
        }
        buttons.push((btn.clone(), value.to_string()));
        box_.append(&btn);
    }

    for (btn, value) in buttons.clone() {
        let config = Arc::clone(&config);
        let value_clone = value.to_string();
        let buttons_clone = buttons.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_notification_rounding(&value_clone);
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg;
                for (b, v) in buttons_clone.iter() {
                    if v == &value_clone { b.add_css_class("suggested-action"); }
                    else { b.remove_css_class("suggested-action"); }
                }
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Rounding", box_)
}

fn create_sound_selector_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().notification_sound.clone().unwrap_or_else(|| "message.oga".to_string());

    let sounds = [
        ("Default", "message.oga"),
        ("Glass", "bell.oga"),
        ("Crystal", "complete.oga"),
        ("Sonar", "message-new-instant.oga"),
        ("Pop", "audio-volume-change.oga"),
    ];

    let mut buttons = Vec::new();

    for (label, value) in sounds {
        let btn = Button::with_label(label);
        if current == value {
            btn.add_css_class("suggested-action");
        }
        buttons.push((btn.clone(), value.to_string()));
        box_.append(&btn);
    }

    for (btn, value) in buttons.clone() {
        let config = Arc::clone(&config);
        let value_clone = value.clone();
        let buttons_clone = buttons.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_notification_sound(&value_clone);
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg;
                for (b, v) in buttons_clone.iter() {
                    if v == &value_clone { b.add_css_class("suggested-action"); }
                    else { b.remove_css_class("suggested-action"); }
                }
                
                // Play preview
                std::process::Command::new("paplay")
                    .arg(format!("/usr/share/sounds/freedesktop/stereo/{}", value_clone))
                    .spawn()
                    .ok();
                    
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Sound", box_)
}
