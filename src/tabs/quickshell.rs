use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Button};
use std::sync::{Arc, Mutex};

use crate::core::config::ColorConfig;
use crate::core::quickshell;

fn schedule_notify_color_change_ms(ms: u32) {
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
        let _ = quickshell::notify_color_change();
        gtk4::glib::ControlFlow::Break
    });
}

pub struct QuickshellTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl QuickshellTab {
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
        let title = Label::new(Some("General QuickShell Settings"));
        title.add_css_class("title");
        title.set_halign(gtk4::Align::Start);
        title.set_margin_bottom(24);
        content.append(&title);

        let system_card = GtkBox::new(Orientation::Vertical, 0);
        system_card.add_css_class("card");
        
        // Scaling Row
        let scaling_row = create_scaling_row(Arc::clone(&config));
        system_card.append(&scaling_row);

        content.append(&system_card);

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

fn create_scaling_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 8);
    
    let current_scale = config.lock().unwrap().ui_scale.unwrap_or(100);
    
    let btn_75 = Button::with_label("75%");
    let btn_100 = Button::with_label("100%");
    let btn_125 = Button::with_label("125%");
    
    let update_btn_styles = {
        let btn_75 = btn_75.clone();
        let btn_100 = btn_100.clone();
        let btn_125 = btn_125.clone();
        
        move |scale: u32| {
            btn_75.remove_css_class("suggested-action");
            btn_100.remove_css_class("suggested-action");
            btn_125.remove_css_class("suggested-action");
            
            match scale {
                75 => btn_75.add_css_class("suggested-action"),
                100 => btn_100.add_css_class("suggested-action"),
                125 => btn_125.add_css_class("suggested-action"),
                _ => {}
            }
        }
    };
    
    update_btn_styles(current_scale as u32);

    {
        let config = config.clone();
        let update_btn_styles = update_btn_styles.clone();
        btn_75.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_ui_scale(75);
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                update_btn_styles(75);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    {
        let config = config.clone();
        let update_btn_styles = update_btn_styles.clone();
        btn_100.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_ui_scale(100);
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                update_btn_styles(100);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    {
        let config = config.clone();
        let update_btn_styles = update_btn_styles.clone();
        btn_125.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_ui_scale(125);
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                update_btn_styles(125);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    box_.append(&btn_75);
    box_.append(&btn_100);
    box_.append(&btn_125);

    create_card_row("UI Scale", box_)
}
