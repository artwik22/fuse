use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Button, Switch, gdk};
use std::sync::{Arc, Mutex};

use crate::core::config::ColorConfig;
use crate::core::quickshell;

// Helper function to set background color on a Box using CSS provider
fn set_box_background_color(box_widget: &gtk4::Box, color: &str) {
    // Create a unique CSS class name based on color
    let color_class = format!("color-bar-{}", color.replace("#", "c").replace(" ", ""));
    box_widget.add_css_class(&color_class);
    
    // Create CSS provider with the color
    let css_provider = gtk4::CssProvider::new();
    let css = format!(".{} {{ background-color: {}; }}", color_class, color);
    
    // load_from_string takes &str and returns Result
    css_provider.load_from_string(&css);
    
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

const PRESETS: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("Professional Modern", "#0a0a0a", "#1a1a1a", "#151515", "#ffffff", "#4a9eff"),
    ("Dark Warm", "#0d0d0d", "#1f1f1f", "#181818", "#f5f5f5", "#ff6b35"),
    ("Cool Blue", "#080d14", "#0f1419", "#0a1016", "#e1e5e9", "#00d4ff"),
    ("Minimal Gray", "#0c0c0c", "#161616", "#121212", "#f0f0f0", "#a0a0a0"),
    ("Forest Green", "#0a0f0a", "#141914", "#0e120e", "#e8f5e8", "#4ade80"),
    ("Sunset Orange", "#0f0a05", "#1a140d", "#140f09", "#f5e8d8", "#ff9500"),
    ("Ocean Blue", "#050a0f", "#0d1419", "#091116", "#d8e8f5", "#3b82f6"),
    ("Deep Purple", "#0a0514", "#140d1f", "#0f0916", "#e8d8f5", "#8b5cf6"),
    ("GNOME Monochrome", "#242424", "#303030", "#2a2a2a", "#ffffff", "#3584e4"),
    ("Pure Black", "#030303", "#0a0a0a", "#060606", "#ffffff", "#c0c0c0"),
];

pub struct LookAndFeelTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl LookAndFeelTab {
    pub fn new(config: Arc<Mutex<ColorConfig>>) -> Self {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_overlay_scrolling(false);
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        let title = Label::new(Some("Look & Feel"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        content.append(&title);

        // Colors section
        let colors_section = create_colors_section(Arc::clone(&config));
        content.append(&colors_section);

        // Sidebar section
        let sidebar_section = create_sidebar_section(Arc::clone(&config));
        content.append(&sidebar_section);

        // Misc section
        let misc_section = create_misc_section(Arc::clone(&config));
        content.append(&misc_section);

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

fn create_colors_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");

    let section_title = Label::new(Some("Colors"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_margin_start(18);
    section_title.set_margin_end(18);
    section_title.set_margin_top(18);
    section_title.set_margin_bottom(8);
    section.append(&section_title);

    let desc = Label::new(Some("Choose from predefined color schemes"));
    desc.add_css_class("section-description");
    desc.set_xalign(0.0);
    desc.set_margin_start(18);
    desc.set_margin_end(18);
    desc.set_margin_bottom(18);
    section.append(&desc);

    // Presets grid
    let presets_container = GtkBox::new(Orientation::Vertical, 0);
    presets_container.set_margin_start(18);
    presets_container.set_margin_end(18);
    presets_container.set_margin_bottom(18);
    
    let presets_section = create_presets_section(Arc::clone(&config));
    presets_container.append(&presets_section);
    
    section.append(&presets_container);

    section
}

fn create_presets_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);

    // Use FlowBox instead of Grid for better responsiveness
    let flowbox = gtk4::FlowBox::new();
    flowbox.set_column_spacing(18);
    flowbox.set_row_spacing(18);
    flowbox.set_halign(gtk4::Align::Fill);
    flowbox.set_hexpand(true);
    flowbox.set_vexpand(true);
    // Responsive: adjust columns based on available width
    flowbox.set_max_children_per_line(4);
    flowbox.set_min_children_per_line(1);
    flowbox.set_selection_mode(gtk4::SelectionMode::None);
    flowbox.set_homogeneous(true); // Equal size buttons

    for preset in PRESETS.iter() {
        let (name, bg, primary, secondary, text, accent) = *preset;

        let preset_button = create_preset_button(
            name,
            bg,
            primary,
            secondary,
            text,
            accent,
            Arc::clone(&config),
        );
        flowbox.append(&preset_button);
    }

    section.append(&flowbox);
    section
}

fn create_preset_button(
    name: &str,
    bg: &str,
    primary: &str,
    secondary: &str,
    text: &str,
    accent: &str,
    config: Arc<Mutex<ColorConfig>>,
) -> Button {
    let button = Button::new();
    button.add_css_class("preset-button");
    // Responsive sizing - let CSS handle minimum sizes
    button.set_hexpand(true); // Allow horizontal expansion
    button.set_vexpand(false);
    // Ensure button can shrink if needed
    button.set_can_shrink(true);

    let content = GtkBox::new(Orientation::Vertical, 6);
    content.set_margin_start(16);
    content.set_margin_end(16);
    content.set_margin_top(16);
    content.set_margin_bottom(16);

    // Color preview bars - set background colors directly
    let bg_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    bg_bar.set_size_request(-1, 14);
    bg_bar.add_css_class("color-bar");
    set_box_background_color(&bg_bar, bg);
    content.append(&bg_bar);

    let primary_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    primary_bar.set_size_request(-1, 14);
    primary_bar.add_css_class("color-bar");
    set_box_background_color(&primary_bar, primary);
    content.append(&primary_bar);

    let secondary_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    secondary_bar.set_size_request(-1, 14);
    secondary_bar.add_css_class("color-bar");
    set_box_background_color(&secondary_bar, secondary);
    content.append(&secondary_bar);

    let accent_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    accent_bar.set_size_request(-1, 14);
    accent_bar.add_css_class("color-bar");
    set_box_background_color(&accent_bar, accent);
    content.append(&accent_bar);

    // Preset name
    let name_label = Label::new(Some(name));
    name_label.add_css_class("preset-name");
    name_label.set_margin_top(14);
    content.append(&name_label);

    button.set_child(Some(&content));

    let bg = bg.to_string();
    let primary = primary.to_string();
    let secondary = secondary.to_string();
    let text = text.to_string();
    let accent = accent.to_string();
    let name = name.to_string();
    button.connect_clicked(move |_| {
        // Reload config from disk to preserve existing settings (like sidebar position)
        let mut cfg = ColorConfig::load();
        cfg.update_colors(&bg, &primary, &secondary, &text, &accent);
        cfg.set_preset(&name);
        if let Err(e) = cfg.save() {
        } else {
            // Update the shared config
            *config.lock().unwrap() = cfg.clone();
            // Wait a bit for file to be written and synced to disk
            std::thread::sleep(std::time::Duration::from_millis(200));
            // Notify quickshell about color change
            if let Err(e) = quickshell::notify_color_change() {
            }
        }
    });

    button
}

fn create_sidebar_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");

    let section_title = Label::new(Some("Sidebar"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_margin_start(18);
    section_title.set_margin_end(18);
    section_title.set_margin_top(18);
    section_title.set_margin_bottom(8);
    section.append(&section_title);

    let desc = Label::new(Some("Configure sidebar visibility and position"));
    desc.add_css_class("section-description");
    desc.set_xalign(0.0);
    desc.set_margin_start(18);
    desc.set_margin_end(18);
    desc.set_margin_bottom(18);
    section.append(&desc);

    let content = GtkBox::new(Orientation::Vertical, 0);
    content.set_margin_start(0);
    content.set_margin_end(0);
    content.set_margin_bottom(12);

    // Sidebar Visibility toggle
    let current_visible = config.lock().unwrap().sidebar_visible.unwrap_or(true);
    let sidebar_visible_row = create_toggle_row(
        "Sidebar Visibility",
        "Show or hide the sidebar",
        {
            let config = Arc::clone(&config);
            move |enabled| {
                // Reload config from disk to preserve existing settings
                let mut cfg = ColorConfig::load();
                cfg.set_sidebar_visible(enabled);
                if let Err(e) = cfg.save() {
                } else {
                    // Update the shared config
                    *config.lock().unwrap() = cfg.clone();
                    // Wait a bit for file to be written
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    // Notify quickshell about change
                    if let Err(e) = quickshell::notify_color_change() {
                    }
                }
            }
        },
        current_visible,
    );
    content.append(&sidebar_visible_row);

    // Sidebar Position
    let position_section = create_sidebar_position_section(Arc::clone(&config));
    content.append(&position_section);

    section.append(&content);

    section
}

fn create_toggle_row(
    title: &str,
    description: &str,
    on_toggle: impl Fn(bool) + 'static,
    initial_value: bool,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(0);
    row.set_margin_end(0);
    row.set_margin_top(0);
    row.set_margin_bottom(0);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);
    row.set_valign(gtk4::Align::Center);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk4::Align::Fill);
    text_box.set_valign(gtk4::Align::Center);

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

fn create_sidebar_position_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-row");
    section.set_margin_start(0);
    section.set_margin_end(0);
    section.set_margin_top(0);
    section.set_margin_bottom(0);

    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_valign(gtk4::Align::Center);
    
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
    button_box.set_valign(gtk4::Align::Center);
    
    let current_pos = config.lock().unwrap().sidebar_position.clone().unwrap_or_else(|| "left".to_string());
    let is_left = current_pos == "left";
    let is_top = current_pos == "top";

    let left_button = Button::with_label("Left");
    if is_left {
        left_button.add_css_class("suggested-action");
    }
    let top_button = Button::with_label("Top");
    if is_top {
        top_button.add_css_class("suggested-action");
    }
    
    {
        let config = Arc::clone(&config);
        let top_btn = top_button.clone();
        left_button.connect_clicked(move |btn| {
            // Reload config from disk to preserve existing settings (like color preset)
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_position("left");
            if let Err(e) = cfg.save() {
            } else {
                // Update the shared config
                *config.lock().unwrap() = cfg.clone();
                // Update button styles
                btn.add_css_class("suggested-action");
                top_btn.remove_css_class("suggested-action");
                // Wait a bit for file to be written and synced to disk
                std::thread::sleep(std::time::Duration::from_millis(200));
                if let Err(e) = quickshell::notify_color_change() {
                }
            }
        });
    }
    button_box.append(&left_button);

    {
        let config = Arc::clone(&config);
        let left_btn = left_button.clone();
        top_button.connect_clicked(move |btn| {
            // Reload config from disk to preserve existing settings (like color preset)
            let mut cfg = ColorConfig::load();
            cfg.set_sidebar_position("top");
            if let Err(e) = cfg.save() {
            } else {
                // Update the shared config
                *config.lock().unwrap() = cfg.clone();
                // Update button styles
                btn.add_css_class("suggested-action");
                left_btn.remove_css_class("suggested-action");
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

fn create_misc_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");

    let section_title = Label::new(Some("Misc"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_margin_start(18);
    section_title.set_margin_end(18);
    section_title.set_margin_top(18);
    section_title.set_margin_bottom(8);
    section.append(&section_title);

    let desc = Label::new(Some("Additional appearance settings"));
    desc.add_css_class("section-description");
    desc.set_xalign(0.0);
    desc.set_margin_start(18);
    desc.set_margin_end(18);
    desc.set_margin_bottom(18);
    section.append(&desc);

    let content = GtkBox::new(Orientation::Vertical, 0);
    content.set_margin_start(0);
    content.set_margin_end(0);
    content.set_margin_bottom(18);

    // Rounding section
    let rounding_section = create_rounding_section(Arc::clone(&config));
    content.append(&rounding_section);

    section.append(&content);

    section
}

fn create_rounding_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-row");
    section.set_margin_start(0);
    section.set_margin_end(0);
    section.set_margin_top(0);
    section.set_margin_bottom(0);

    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_valign(gtk4::Align::Center);
    
    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let title = Label::new(Some("Global Rounding"));
    title.add_css_class("row-title");
    title.set_xalign(0.0);
    text_box.append(&title);

    let desc = Label::new(Some("Choose corner rounding style: Rounded or Sharp"));
    desc.add_css_class("row-description");
    desc.set_xalign(0.0);
    text_box.append(&desc);

    header.append(&text_box);

    let button_box = GtkBox::new(Orientation::Horizontal, 10);
    button_box.set_valign(gtk4::Align::Center);
    
    let current_rounding = config.lock().unwrap().rounding.clone().unwrap_or_else(|| "rounded".to_string());
    let is_rounded = current_rounding == "rounded";
    let is_sharp = current_rounding == "sharp";

    let rounded_button = Button::with_label("Rounded");
    if is_rounded {
        rounded_button.add_css_class("suggested-action");
    }
    let sharp_button = Button::with_label("Sharp");
    if is_sharp {
        sharp_button.add_css_class("suggested-action");
    }
    
    {
        let config = Arc::clone(&config);
        let sharp_btn = sharp_button.clone();
        rounded_button.connect_clicked(move |btn| {
            // Reload config from disk to preserve existing settings
            let mut cfg = ColorConfig::load();
            cfg.set_rounding("rounded");
            if let Err(e) = cfg.save() {
            } else {
                // Update the shared config
                *config.lock().unwrap() = cfg.clone();
                // Update button styles
                btn.add_css_class("suggested-action");
                sharp_btn.remove_css_class("suggested-action");
                // Wait a bit for file to be written and synced to disk
                std::thread::sleep(std::time::Duration::from_millis(200));
                if let Err(e) = quickshell::notify_color_change() {
                }
            }
        });
    }
    button_box.append(&rounded_button);

    {
        let config = Arc::clone(&config);
        let rounded_btn = rounded_button.clone();
        sharp_button.connect_clicked(move |btn| {
            // Reload config from disk to preserve existing settings
            let mut cfg = ColorConfig::load();
            cfg.set_rounding("sharp");
            if let Err(e) = cfg.save() {
            } else {
                // Update the shared config
                *config.lock().unwrap() = cfg.clone();
                // Update button styles
                btn.add_css_class("suggested-action");
                rounded_btn.remove_css_class("suggested-action");
                // Wait a bit for file to be written and synced to disk
                std::thread::sleep(std::time::Duration::from_millis(200));
                if let Err(e) = quickshell::notify_color_change() {
                }
            }
        });
    }
    button_box.append(&sharp_button);

    header.append(&button_box);
    section.append(&header);

    section
}
