use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Switch, Button, Entry, Revealer, RevealerTransitionType, Popover, ListBox};
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
        let title = Label::new(Some("QuickShell Settings"));
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

        // --- System Group ---
        add_group_header(&content, "System");
        let system_card = GtkBox::new(Orientation::Vertical, 0);
        system_card.add_css_class("card");
        
        // Scaling Row
        let scaling_row = create_scaling_row(Arc::clone(&config));
        system_card.append(&scaling_row);

        // Notifications Logic
        let (notif_row, notif_switch) = create_notifications_row(Arc::clone(&config));
        system_card.append(&notif_row);

        // Notification Sounds Row (Conditional)
        let sounds_row = create_notification_sounds_row(Arc::clone(&config));
        
        // Wrap sounds row in a Revealer
        let revealer = Revealer::new();
        revealer.set_transition_type(RevealerTransitionType::SlideDown);
        revealer.set_transition_duration(250);
        revealer.set_child(Some(&sounds_row));
        
        // Bind visibility
        let is_active = notif_switch.is_active();
        revealer.set_reveal_child(is_active);
        
        {
            let revealer = revealer.clone();
            notif_switch.connect_active_notify(move |sw| {
                revealer.set_reveal_child(sw.is_active());
            });
        }

        system_card.append(&revealer);
        content.append(&system_card);


        // --- Sidebar Group ---
        add_group_header(&content, "Sidebar");
        let sidebar_card = GtkBox::new(Orientation::Vertical, 0);
        sidebar_card.add_css_class("card");

        sidebar_card.append(&create_sidebar_visible_row(Arc::clone(&config)));
        sidebar_card.append(&create_sidebar_position_row(Arc::clone(&config)));
        sidebar_card.append(&create_sidepanel_content_row(Arc::clone(&config)));
        sidebar_card.append(&create_github_username_row(Arc::clone(&config)));
        
        content.append(&sidebar_card);

        // --- Dashboard Group ---
        add_group_header(&content, "Dashboard");
        let dashboard_card = GtkBox::new(Orientation::Vertical, 0);
        dashboard_card.add_css_class("card");

        dashboard_card.append(&create_dashboard_position_row(Arc::clone(&config)));
        dashboard_card.append(&create_dashboard_tile_row(Arc::clone(&config)));
        dashboard_card.append(&create_dashboard_resource_row("Resource 1", true, Arc::clone(&config)));
        dashboard_card.append(&create_dashboard_resource_row("Resource 2", false, Arc::clone(&config)));
        dashboard_card.append(&create_weather_location_row(Arc::clone(&config)));

        content.append(&dashboard_card);

        // --- Appearance Group ---
        add_group_header(&content, "Appearance");
        let appearance_card = GtkBox::new(Orientation::Vertical, 0);
        appearance_card.add_css_class("card");

        appearance_card.append(&create_border_radius_row(Arc::clone(&config)));

        content.append(&appearance_card);

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

// --- Helper for consistent rows ---
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

// --- Row Creators ---

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
    
    // Init styles
    update_btn_styles(current_scale as u32);

    // 75%
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

    // 100%
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

    // 125%
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

fn create_notifications_row(config: Arc<Mutex<ColorConfig>>) -> (GtkBox, Switch) {
    let switch = Switch::new();
    let current = config.lock().unwrap().notifications_enabled.unwrap_or(true);
    switch.set_active(current);
    switch.set_valign(gtk4::Align::Center);

    {
        let config = config.clone();
        switch.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            cfg.set_notifications_enabled(s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                schedule_notify_color_change_ms(200);
            }
        });
    }

    (create_card_row("Show Notifications", switch.clone()), switch)
}

fn create_notification_sounds_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let switch = Switch::new();
    let current = config.lock().unwrap().notification_sounds_enabled.unwrap_or(true);
    switch.set_active(current);
    switch.set_valign(gtk4::Align::Center);

    {
        let config = config.clone();
        switch.connect_active_notify(move |s| {
            let mut cfg = ColorConfig::load();
            cfg.set_notification_sounds_enabled(s.is_active());
            if cfg.save().is_ok() {
                *config.lock().unwrap() = cfg.clone();
                schedule_notify_color_change_ms(200);
            }
        });
    }

    create_card_row("Notification Sounds", switch)
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
    
    // Logic to update buttons (simple closure approach is tricky with vec, using a simpler pattern)
    // We rebuild the buttons logic manually because it's easier than extensive cloning
    
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
            schedule_notify_color_change_ms(500); // slightly longer delay for typing
        }
    });

    create_card_row("GitHub User", entry)
}
fn create_weather_location_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let entry = Entry::new();
    let current = config.lock().unwrap().weather_location.clone().unwrap_or_else(|| "London".to_string());
    entry.set_text(&current);
    entry.set_placeholder_text(Some("City Name (e.g. London)"));
    entry.set_width_chars(20);
    entry.set_valign(gtk4::Align::Center);

    let popover = Popover::new();
    popover.set_parent(&entry);
    popover.set_autohide(true);
    popover.set_position(gtk4::PositionType::Bottom);

    let listbox = ListBox::new();
    listbox.set_selection_mode(gtk4::SelectionMode::Single);
    popover.set_child(Some(&listbox));

    // Debounce state
    let debounce_id = Arc::new(Mutex::new(Option::<gtk4::glib::SourceId>::None));

    {
        let config = config.clone();
        let listbox = listbox.clone();
        let popover = popover.clone();
        let debounce_id = debounce_id.clone();
        let entry_weak = entry.downgrade();

        entry.connect_changed(move |e| {
            let mut db_id = debounce_id.lock().unwrap();
            if let Some(id) = db_id.take() {
                id.remove();
            }

            let query = e.text().to_string();
            if query.len() < 3 {
                popover.popdown();
                return;
            }

            let entry_weak = entry_weak.clone();
            let config = config.clone();
            let listbox = listbox.clone();
            let popover = popover.clone();
            let query_clone = query.clone();

            let new_id = gtk4::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                let query = query_clone.clone();
                let listbox = listbox.clone();
                let popover = popover.clone();
                let config = config.clone();
                let entry_weak = entry_weak.clone();

                // Clear existing suggestions
                while let Some(row) = listbox.first_child() {
                    listbox.remove(&row);
                }

                // Fetch suggestions using curl (to avoid adding reqwest dependency)
                let url = format!("https://nominatim.openstreetmap.org/search?q={}&format=json&limit=5", query);
                
                // Create a channel to send results back to the main thread
                let (sender, receiver) = gtk4::glib::MainContext::channel::<Vec<String>>(gtk4::glib::Priority::default());
                
                // Set up the receiver on the main thread
                let listbox_clone = listbox.clone();
                let popover_clone = popover.clone();
                let entry_weak_clone = entry_weak.clone();

                receiver.attach(None, move |suggestions| {
                    if let Some(_entry) = entry_weak_clone.upgrade() {
                        for sug in suggestions {
                            let label = Label::new(Some(&sug));
                            label.set_margin_start(8);
                            label.set_margin_end(8);
                            label.set_margin_top(4);
                            label.set_margin_bottom(4);
                            label.set_halign(gtk4::Align::Start);
                            listbox_clone.append(&label);
                        }

                        if listbox_clone.first_child().is_some() {
                            popover_clone.popup();
                        } else {
                            popover_clone.popdown();
                        }
                    }
                    gtk4::glib::ControlFlow::Break
                });

                let url = url.clone();
                std::thread::spawn(move || {
                    let output = std::process::Command::new("curl")
                        .arg("-s")
                        .arg("-H")
                        .arg("User-Agent: Alloy-Fuse/1.0")
                        .arg(&url)
                        .output();

                    if let Ok(output) = output {
                        if output.status.success() {
                            let body = String::from_utf8_lossy(&output.stdout).to_string();
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                                if let Some(arr) = json.as_array() {
                                    let mut suggestions = Vec::new();
                                    for item in arr {
                                        if let Some(display_name) = item.get("display_name").and_then(|v| v.as_str()) {
                                            suggestions.push(display_name.to_string());
                                        }
                                    }
                                    let _ = sender.send(suggestions);
                                }
                            }
                        }
                    }
                });

                gtk4::glib::ControlFlow::Break
            });

            *db_id = Some(new_id);
        });
    }

    {
        let config = config.clone();
        let entry_weak = entry.downgrade();
        let popover = popover.clone();
        listbox.connect_row_activated(move |_, row| {
            if let Some(label) = row.child().and_then(|w| w.downcast::<Label>().ok()) {
                let full_name = label.text().to_string();
                // Take only the first part before the comma as the location for wttr.in
                let city = full_name.split(',').next().unwrap_or(&full_name).trim();
                
                if let Some(entry) = entry_weak.upgrade() {
                    entry.set_text(city);
                    
                    let mut cfg = ColorConfig::load();
                    cfg.set_weather_location(city);
                    if cfg.save().is_ok() {
                        *config.lock().unwrap() = cfg.clone();
                        schedule_notify_color_change_ms(200);
                    }
                }
                popover.popdown();
            }
        });
    }

    // Also update config on manual entry enter
    entry.connect_activate(move |e| {
        let city = e.text().to_string();
        let mut cfg = ColorConfig::load();
        cfg.set_weather_location(&city);
        if cfg.save().is_ok() {
            *config.lock().unwrap() = cfg.clone();
            schedule_notify_color_change_ms(200);
        }
    });

    create_card_row("Weather City", entry)
}

fn create_dashboard_position_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    // Reuse sidebar position logic pattern for dashboard
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().dashboard_position.clone().unwrap_or_else(|| "right".to_string());

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
            cfg.set_dashboard_position(val.to_lowercase().as_str());
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

fn create_dashboard_tile_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().dashboard_tile_left.clone().unwrap_or_else(|| "battery".to_string());
    
    let is_bat = current == "battery";
    
    let btn_bat = Button::with_label("Battery");
    let btn_net = Button::with_label("Network");
    
    if is_bat { btn_bat.add_css_class("suggested-action"); }
    else { btn_net.add_css_class("suggested-action"); }

    let update = {
        let b = btn_bat.clone();
        let n = btn_net.clone();
        move |bat: bool| {
            if bat {
                b.add_css_class("suggested-action");
                n.remove_css_class("suggested-action");
            } else {
                n.add_css_class("suggested-action");
                b.remove_css_class("suggested-action");
            }
        }
    };

    {
        let cfg_ref = config.clone();
        let up = update.clone();
        btn_bat.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_dashboard_tile_left("battery");
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
        btn_net.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_dashboard_tile_left("network");
            if cfg.save().is_ok() {
                *cfg_ref.lock().unwrap() = cfg.clone();
                up(false);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    box_.append(&btn_bat);
    box_.append(&btn_net);

    create_card_row("Info Tile", box_)
}

fn create_dashboard_resource_row(label: &str, is_res1: bool, config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = if is_res1 {
        config.lock().unwrap().dashboard_resource_1.clone().unwrap_or_else(|| "cpu".to_string())
    } else {
        config.lock().unwrap().dashboard_resource_2.clone().unwrap_or_else(|| "ram".to_string())
    };

    let resources = vec!["CPU", "RAM", "GPU", "Network"];
    let mut buttons = Vec::new();

    for res in &resources {
        let btn = Button::with_label(res);
        if current.eq_ignore_ascii_case(res) {
            btn.add_css_class("suggested-action");
        }
        buttons.push(btn.clone());
        box_.append(&btn);
    }

    let btn_cpu = buttons[0].clone();
    let btn_ram = buttons[1].clone();
    let btn_gpu = buttons[2].clone();
    let btn_net = buttons[3].clone();

    let update_visuals = {
        let bc = btn_cpu.clone();
        let br = btn_ram.clone();
        let bg = btn_gpu.clone();
        let bn = btn_net.clone();
        move |new_val: &str| {
            bc.remove_css_class("suggested-action");
            br.remove_css_class("suggested-action");
            bg.remove_css_class("suggested-action");
            bn.remove_css_class("suggested-action");
            match new_val.to_lowercase().as_str() {
                "cpu" => bc.add_css_class("suggested-action"),
                "ram" => br.add_css_class("suggested-action"),
                "gpu" => bg.add_css_class("suggested-action"),
                "network" => bn.add_css_class("suggested-action"),
                _ => {}
            }
        }
    };

    let bind_click = |btn: &Button, val: &'static str, is_r1: bool, config: Arc<Mutex<ColorConfig>>, updater: Box<dyn Fn(&str)>| {
        btn.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            if is_r1 {
                cfg.set_dashboard_resource_1(val.to_lowercase().as_str());
            } else {
                cfg.set_dashboard_resource_2(val.to_lowercase().as_str());
            }
            if cfg.save().is_ok() {
                if is_r1 {
                   config.lock().unwrap().dashboard_resource_1 = Some(val.to_lowercase());
                } else {
                   config.lock().unwrap().dashboard_resource_2 = Some(val.to_lowercase());
                }
                updater(val);
                schedule_notify_color_change_ms(200);
            }
        });
    };

    bind_click(&btn_cpu, "CPU", is_res1, config.clone(), Box::new(update_visuals.clone()));
    bind_click(&btn_ram, "RAM", is_res1, config.clone(), Box::new(update_visuals.clone()));
    bind_click(&btn_gpu, "GPU", is_res1, config.clone(), Box::new(update_visuals.clone()));
    bind_click(&btn_net, "Network", is_res1, config.clone(), Box::new(update_visuals.clone()));

    create_card_row(label, box_)
}

fn create_border_radius_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Horizontal, 6);
    let current = config.lock().unwrap().quickshell_border_radius.unwrap_or(0);
    
    let is_none = current == 0;
    
    let btn_none = Button::with_label("None");
    let btn_slight = Button::with_label("Slight");
    
    if is_none { btn_none.add_css_class("suggested-action"); }
    else { btn_slight.add_css_class("suggested-action"); }

    let update = {
        let n = btn_none.clone();
        let s = btn_slight.clone();
        move |none: bool| {
            if none {
                n.add_css_class("suggested-action");
                s.remove_css_class("suggested-action");
            } else {
                s.add_css_class("suggested-action");
                n.remove_css_class("suggested-action");
            }
        }
    };

    {
        let cfg_ref = config.clone();
        let up = update.clone();
        btn_none.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_quickshell_border_radius(0);
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
        btn_slight.connect_clicked(move |_| {
            let mut cfg = ColorConfig::load();
            cfg.set_quickshell_border_radius(4);
            if cfg.save().is_ok() {
                *cfg_ref.lock().unwrap() = cfg.clone();
                up(false);
                schedule_notify_color_change_ms(200);
            }
        });
    }

    box_.append(&btn_none);
    box_.append(&btn_slight);

    create_card_row("Border Radius", box_)
}
