use libadwaita::prelude::*;
use libadwaita::Application;
use gtk4::{gio, CssProvider};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::window::FuseWindow;
use crate::core::config::ColorConfig;

const APP_ID: &str = "com.alloy.fuse";

pub struct FuseApp {
    app: Application,
    _config: Arc<Mutex<ColorConfig>>,
    _css_provider: Rc<RefCell<Option<CssProvider>>>,
    _monitors: Vec<gio::FileMonitor>,
}

impl FuseApp {
    pub fn new(config: ColorConfig) -> Self {
        let config = Arc::new(Mutex::new(config));
        let app = Application::builder()
            .application_id(APP_ID)
            .build();
        let css_provider = Rc::new(RefCell::new(None));

        let css_provider_clone = css_provider.clone();
        let config_startup = Arc::clone(&config);
        app.connect_startup(move |_| {
            load_css_with_colors(&css_provider_clone, &config_startup);
        });

        let config_activate = Arc::clone(&config);
        app.connect_activate(move |app| {
            let window = FuseWindow::new(app, &config_activate);
            window.present();
        });

        // Start monitoring for color changes
        let css_provider_monitor = css_provider.clone();
        let config_monitor = Arc::clone(&config);
        let monitors = start_color_monitoring(css_provider_monitor, config_monitor);

        Self { 
            app,
            _config: config,
            _css_provider: css_provider,
            _monitors: monitors,
        }
    }

    pub fn run(&self) {
        self.app.run();
    }
}

fn load_css_with_colors(css_provider_rc: &Rc<RefCell<Option<CssProvider>>>, config: &Arc<Mutex<ColorConfig>>) {
    let config = config.lock().unwrap().clone();
    
    // Load base CSS
    let base_css = include_str!("resources/style.css");
    
    // Construct dynamic color definitions
    let mut dynamic_css = format!(
        "@define-color window_bg_color {};\n\
         @define-color window_fg_color {};\n\
         @define-color headerbar_bg_color {};\n\
         @define-color headerbar_fg_color {};\n\
         @define-color card_bg_color {};\n\
         @define-color card_fg_color {};\n\
         @define-color accent_bg_color {};\n\
         @define-color accent_color {};\n\
         @define-color accent_fg_color {};\n\
         @define-color sidebar_bg_color {};\n\
         @define-color view_bg_color {};\n\n",
        config.background,
        config.text,
        config.background, // header matches window
        config.text,
        config.secondary,
        config.text,
        config.accent,
        config.accent,
        get_contrasting_text_color(&config.accent), // calculated accent fg
        config.secondary,
        config.background // view matches window
    );

    // Append base CSS
    dynamic_css.push_str(base_css);
    
    // Apply rounding setting (simple replace to avoid regex on large CSS at startup)
    let rounding = config.rounding.as_deref().unwrap_or("rounded");
    if rounding == "sharp" {
        dynamic_css = dynamic_css.replace("border-radius: 8px", "border-radius: 0px")
            .replace("border-radius: 6px", "border-radius: 0px")
            .replace("border-radius: 12px", "border-radius: 0px")
            .replace("border-radius: 4px", "border-radius: 0px");
    }
    
    let provider = CssProvider::new();
    provider.load_from_string(&dynamic_css);
    
    let display = gtk4::gdk::Display::default().expect("Could not connect to display");
    
    // Remove old provider if exists
    if let Some(old_provider) = css_provider_rc.borrow().as_ref() {
        gtk4::style_context_remove_provider_for_display(&display, old_provider);
    }
    
    // Add new provider
    gtk4::style_context_add_provider_for_display(
        &display,
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    
    // Store provider reference
    *css_provider_rc.borrow_mut() = Some(provider);
}

fn start_color_monitoring(css_provider_rc: Rc<RefCell<Option<CssProvider>>>, config: Arc<Mutex<ColorConfig>>) -> Vec<gio::FileMonitor> {
    let mut monitors = Vec::new();
    let config_path = ColorConfig::get_config_path();
    
    // Monitor colors.json
    let file = gio::File::for_path(&config_path);
    if let Ok(monitor) = file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
        let css_provider_rc_clone = css_provider_rc.clone();
        let config_clone = Arc::clone(&config);
        monitor.connect_changed(move |_, _, _, event_type| {
            if matches!(event_type, gio::FileMonitorEvent::Changed | gio::FileMonitorEvent::ChangesDoneHint) {
                *config_clone.lock().unwrap() = ColorConfig::load();
                load_css_with_colors(&css_provider_rc_clone, &config_clone);
            }
        });
        monitors.push(monitor);
    }
    
    // Monitor /tmp/quickshell_color_change notification file
    let notification_file = gio::File::for_path("/tmp/quickshell_color_change");
    if let Ok(monitor) = notification_file.monitor_file(gio::FileMonitorFlags::NONE, gio::Cancellable::NONE) {
        let css_provider_rc_clone = css_provider_rc.clone();
        let config_clone = Arc::clone(&config);
        monitor.connect_changed(move |_, _, _, event_type| {
            if matches!(event_type, gio::FileMonitorEvent::Changed | gio::FileMonitorEvent::ChangesDoneHint | gio::FileMonitorEvent::Created) {
                *config_clone.lock().unwrap() = ColorConfig::load();
                load_css_with_colors(&css_provider_rc_clone, &config_clone);
            }
        });
        monitors.push(monitor);
    }
    
    monitors
}

fn get_contrasting_text_color(hex: &str) -> String {
    let hex = hex.trim().trim_start_matches('#');
    
    // Explicit handle for common white/black (optimization and safety)
    if hex.eq_ignore_ascii_case("ffffff") || hex.eq_ignore_ascii_case("fff") || hex.eq_ignore_ascii_case("white") {
        return "#000000".to_string();
    }
    if hex.eq_ignore_ascii_case("000000") || hex.eq_ignore_ascii_case("000") || hex.eq_ignore_ascii_case("black") {
        return "#ffffff".to_string();
    }

    // Handle 3-digit hex
    if hex.len() == 3 {
        let r_char = &hex[0..1];
        let g_char = &hex[1..2];
        let b_char = &hex[2..3];
        let full_hex = format!("{}{}{}{}{}{}", r_char, r_char, g_char, g_char, b_char, b_char);
        return get_contrasting_text_color(&full_hex);
    }

    if hex.len() == 6 {
        if let (Ok(r), Ok(g), Ok(b)) = (
            u8::from_str_radix(&hex[0..2], 16),
            u8::from_str_radix(&hex[2..4], 16),
            u8::from_str_radix(&hex[4..6], 16),
        ) {
            // Calculate brightness using standard formula
            let brightness = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;
            if brightness > 0.5 { // Lowered threshold slightly to 0.5 for better white detection
                return "#000000".to_string(); // Bright background -> Dark text
            }
        }
    }
    "#ffffff".to_string() // Default to white text
}
