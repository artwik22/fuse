use libadwaita::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Switch, Align};
use std::sync::{Arc, Mutex};
use crate::core::config::ColorConfig;

pub struct LockScreenTab {
    widget: gtk4::ScrolledWindow,
}

impl LockScreenTab {
    pub fn new(config: Arc<Mutex<ColorConfig>>) -> Self {
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_overlay_scrolling(false);
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);

        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);
        main_box.set_margin_start(24);
        main_box.set_margin_end(24);
        main_box.set_margin_top(24);
        main_box.set_margin_bottom(48);

        // Header Title
        let title = gtk4::Label::new(Some("Lock Screen"));
        title.add_css_class("title");
        title.set_halign(gtk4::Align::Start);
        title.set_margin_bottom(24);
        main_box.append(&title);

        let input_group = libadwaita::PreferencesGroup::builder()
            .title("Widgets")
            .description("Manage widgets displayed on the lock screen.")
            .build();

        let current_config = config.lock().unwrap();

        // 1. Media Player
        let media_config = Arc::clone(&config);
        let row_media = create_switch_row(
            "Media Player",
            "Show media controls when music is playing",
            current_config.lockscreen_media_enabled.unwrap_or(true),
            move |active| {
                if let Ok(mut c) = media_config.lock() {
                    c.set_lockscreen_media_enabled(active);
                    let _ = c.save();
                }
            }
        );
        input_group.add(&row_media);

        // 2. Weather
        let weather_config = Arc::clone(&config);
        let row_weather = create_switch_row(
            "Weather",
            "Show current weather conditions",
            current_config.lockscreen_weather_enabled.unwrap_or(true),
            move |active| {
                if let Ok(mut c) = weather_config.lock() {
                    c.set_lockscreen_weather_enabled(active);
                    let _ = c.save();
                }
            }
        );
        input_group.add(&row_weather);

        // 3. Battery
        let battery_config = Arc::clone(&config);
        let row_battery = create_switch_row(
            "Battery",
            "Show battery status and percentage",
            current_config.lockscreen_battery_enabled.unwrap_or(true),
            move |active| {
                if let Ok(mut c) = battery_config.lock() {
                    c.set_lockscreen_battery_enabled(active);
                    let _ = c.save();
                }
            }
        );
        input_group.add(&row_battery);



        // 5. Calendar
        let calendar_config = Arc::clone(&config);
        let row_calendar = create_switch_row(
            "Calendar",
            "Show upcoming events",
            current_config.lockscreen_calendar_enabled.unwrap_or(true),
            move |active| {
                if let Ok(mut c) = calendar_config.lock() {
                    c.set_lockscreen_calendar_enabled(active);
                    let _ = c.save();
                }
            }
        );
        input_group.add(&row_calendar);

        // 6. Network
        let network_config = Arc::clone(&config);
        let row_network = create_switch_row(
            "Network",
            "Show network connection status",
            current_config.lockscreen_network_enabled.unwrap_or(false),
            move |active| {
                if let Ok(mut c) = network_config.lock() {
                    c.set_lockscreen_network_enabled(active);
                    let _ = c.save();
                }
            }
        );
        input_group.add(&row_network);

        main_box.append(&input_group);
        scrolled.set_child(Some(&main_box));

        Self {
            widget: scrolled,
        }
    }

    pub fn widget(&self) -> &gtk4::ScrolledWindow {
        &self.widget
    }
}

fn create_switch_row<F>(title: &str, subtitle: &str, initial: bool, callback: F) -> libadwaita::ActionRow
where
    F: Fn(bool) + 'static,
{
    let row = libadwaita::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    
    let switch = Switch::builder()
        .valign(Align::Center)
        .active(initial)
        .build();
    
    switch.connect_active_notify(move |s| {
        callback(s.is_active());
    });
    
    row.add_suffix(&switch);
    row
}
