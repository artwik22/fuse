use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Switch, Button, Entry};
use std::sync::{Arc, Mutex};

use crate::core::config::ColorConfig;
use crate::core::quickshell;

fn schedule_notify_color_change_ms(ms: u32) {
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
        let _ = quickshell::notify_color_change();
        gtk4::glib::ControlFlow::Break
    });
}

pub struct QuickshellSidebarTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl QuickshellSidebarTab {
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
        let title = Label::new(Some("Sidebar Settings"));
        title.add_css_class("title");
        title.set_halign(gtk4::Align::Start);
        title.set_margin_bottom(24);
        content.append(&title);

        let sidebar_card = GtkBox::new(Orientation::Vertical, 0);
        sidebar_card.add_css_class("card");

        sidebar_card.append(&create_sidebar_visible_row(Arc::clone(&config)));
        sidebar_card.append(&create_clock_blink_colon_row(Arc::clone(&config)));
        sidebar_card.append(&create_sidebar_position_row(Arc::clone(&config)));
        sidebar_card.append(&create_sidebar_workspace_mode_row(Arc::clone(&config)));
        sidebar_card.append(&create_sidebar_style_row(Arc::clone(&config)));
        sidebar_card.append(&create_dynamic_sidebar_background_row(Arc::clone(&config)));
        sidebar_card.append(&create_sidebar_battery_enabled_row(Arc::clone(&config)));
        sidebar_card.append(&create_sidepanel_content_row(Arc::clone(&config)));
        sidebar_card.append(&create_github_username_row(Arc::clone(&config)));
        
        content.append(&sidebar_card);

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

fn create_sidebar_visible_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let switch = Switch::new();
    let current = config.lock().unwrap().sidebar_visible.unwrap_or(true);
    switch.set_active(current);
    switch.set_valign(gtk4::Align::Center);

    {
        let config = config.clone();
        switch.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_visible(s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Show Sidebar", switch)
}

fn create_clock_blink_colon_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let switch = Switch::new();
    let current = config.lock().unwrap().clock_blink_colon.unwrap_or(true);
    switch.set_active(current);
    switch.set_valign(gtk4::Align::Center);

    {
        let config = config.clone();
        switch.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            cfg.set_clock_blink_colon(s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Blink Clock Colon", switch)
}

fn create_sidebar_position_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().sidebar_position.clone().unwrap_or_else(|| "left".to_string());

    let positions = vec!["Left", "Bottom", "Top", "Right"];
    let mut buttons = Vec::new();

    for pos in positions {
        let btn = Button::with_label(pos);
        if current.eq_ignore_ascii_case(pos) {
            btn.add_css_class("suggested-action");
        }
        buttons.push(btn.clone());
        box_.append(&btn);
    }
    
    let btn_left = buttons[0].clone();
    let btn_bottom = buttons[1].clone();
    let btn_top = buttons[2].clone();
    let btn_right = buttons[3].clone();

    let update_visuals = {
        let bl = btn_left.clone();
        let bb = btn_bottom.clone();
        let bt = btn_top.clone();
        let br = btn_right.clone();
        move |new_pos: &str| {
            bl.remove_css_class("suggested-action");
            bb.remove_css_class("suggested-action");
            bt.remove_css_class("suggested-action");
            br.remove_css_class("suggested-action");
            match new_pos.to_lowercase().as_str() {
                "left" => bl.add_css_class("suggested-action"),
                "bottom" => bb.add_css_class("suggested-action"),
                "top" => bt.add_css_class("suggested-action"),
                "right" => br.add_css_class("suggested-action"),
                _ => {}
            }
        }
    };

    let bind_click = |btn: &Button, val: &'static str, config: Arc<Mutex<ColorConfig>>, updater: Box<dyn Fn(&str)>| {
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_position(val.to_lowercase().as_str());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                updater(val);
                schedule_notify_color_change_ms(200);
            }
        });
    };

    bind_click(&btn_left, "Left", config.clone(), Box::new(update_visuals.clone()));
    bind_click(&btn_bottom, "Bottom", config.clone(), Box::new(update_visuals.clone()));
    bind_click(&btn_top, "Top", config.clone(), Box::new(update_visuals.clone()));
    bind_click(&btn_right, "Right", config.clone(), Box::new(update_visuals.clone()));

    create_card_row("Position", box_)
}

fn create_sidebar_workspace_mode_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().sidebar_workspace_mode.clone().unwrap_or_else(|| "top".to_string());
    
    let is_top = current == "top";
    let is_center = current == "center";
    let _is_bottom = current == "bottom";
    
    let btn_top = Button::with_label("Top");
    let btn_center = Button::with_label("Center");
    let btn_bottom = Button::with_label("Bottom");
    
    if is_top { btn_top.add_css_class("suggested-action"); }
    else if is_center { btn_center.add_css_class("suggested-action"); }
    else { btn_bottom.add_css_class("suggested-action"); }

    let update = {
        let t = btn_top.clone();
        let c = btn_center.clone();
        let b = btn_bottom.clone();
        move |mode: &str| {
            t.remove_css_class("suggested-action");
            c.remove_css_class("suggested-action");
            b.remove_css_class("suggested-action");
            match mode {
                "top" => t.add_css_class("suggested-action"),
                "center" => c.add_css_class("suggested-action"),
                "bottom" => b.add_css_class("suggested-action"),
                _ => {}
            }
        }
    };

    let connect_btn = |btn: &Button, mode: &'static str| {
        let cfg_ref = config.clone();
        let up = update.clone();
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_workspace_mode(mode);
            if cfg.save().is_ok() {
                *cfg_ref.lock().unwrap() = cfg.clone();
                up(mode);
                schedule_notify_color_change_ms(200);
            }
        });
    };

    connect_btn(&btn_top, "top");
    connect_btn(&btn_center, "center");
    connect_btn(&btn_bottom, "bottom");

    box_.append(&btn_top);
    box_.append(&btn_center);
    box_.append(&btn_bottom);

    create_card_row("Workspace Pos", box_)
}

fn create_sidebar_style_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().sidebar_style.clone().unwrap_or_else(|| "dots".to_string());
    
    let is_dots = current == "dots";
    
    let btn_dots = Button::with_label("Dots");
    let btn_lines = Button::with_label("Lines");
    
    if is_dots { btn_dots.add_css_class("suggested-action"); }
    else { btn_lines.add_css_class("suggested-action"); }

    let update = {
        let d = btn_dots.clone();
        let l = btn_lines.clone();
        move |dots: bool| {
            if dots {
                d.add_css_class("suggested-action");
                l.remove_css_class("suggested-action");
            } else {
                l.add_css_class("suggested-action");
                d.remove_css_class("suggested-action");
            }
        }
    };

    {
        let cfg_ref = config.clone();
        let up = update.clone();
        btn_dots.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_style("dots");
            if cfg.save().is_ok() {
                *cfg_ref.lock().unwrap() = cfg.clone();
                up(true);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    {
        let cfg_ref = config.clone();
        let up = update.clone();
        btn_lines.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_style("lines");
            if cfg.save().is_ok() {
                *cfg_ref.lock().unwrap() = cfg.clone();
                up(false);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    box_.append(&btn_dots);
    box_.append(&btn_lines);

    create_card_row("Style", box_)
}

fn create_sidepanel_content_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().sidepanel_content.clone().unwrap_or_else(|| "calendar".to_string());
    
    let is_cal = current == "calendar";
    
    let btn_cal = Button::with_label("Calendar");
    let btn_gh = Button::with_label("GitHub");
    
    if is_cal { btn_cal.add_css_class("suggested-action"); }
    else { btn_gh.add_css_class("suggested-action"); }

    let update = {
        let c = btn_cal.clone();
        let g = btn_gh.clone();
        move |cal: bool| {
            if cal {
                c.add_css_class("suggested-action");
                g.remove_css_class("suggested-action");
            } else {
                g.add_css_class("suggested-action");
                c.remove_css_class("suggested-action");
            }
        }
    };

    {
        let cfg_ref = config.clone();
        let up = update.clone();
        btn_cal.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidepanel_content("calendar");
            if cfg.save().is_ok() {
                *cfg_ref.lock().unwrap() = cfg.clone();
                up(true);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    {
        let cfg_ref = config.clone();
        let up = update.clone();
        btn_gh.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidepanel_content("github");
            if cfg.save().is_ok() {
                *cfg_ref.lock().unwrap() = cfg.clone();
                up(false);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    box_.append(&btn_cal);
    box_.append(&btn_gh);

    create_card_row("Content", box_)
}

fn create_github_username_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let entry = Entry::new();
    let current = config.lock().unwrap().github_username.clone().unwrap_or_default();
    entry.set_text(&current);
    entry.set_placeholder_text(Some("GitHub Username"));
    entry.set_width_chars(15);
    entry.set_valign(gtk4::Align::Center);

    entry.connect_changed(move |e| {
        let mut cfg = ColorConfig::load();
        cfg.set_github_username(&e.text());
        if cfg.save().is_ok() {
            *config.lock().unwrap() = cfg.clone();
            schedule_notify_color_change_ms(500);
        }
    });

    create_card_row("GitHub User", entry)
}

fn create_dynamic_sidebar_background_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let switch = Switch::new();
    let current = config.lock().unwrap().dynamic_sidebar_background.unwrap_or(false);
    switch.set_active(current);
    switch.set_valign(gtk4::Align::Center);

    {
        let config = config.clone();
        switch.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            cfg.set_dynamic_sidebar_background(s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Dynamic Background", switch)
}

fn create_sidebar_battery_enabled_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let switch = Switch::new();
    let current = config.lock().unwrap().sidebar_battery_enabled.unwrap_or(true);
    switch.set_active(current);
    switch.set_valign(gtk4::Align::Center);

    {
        let config = config.clone();
        switch.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_battery_enabled(s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Show Battery Widget", switch)
}
