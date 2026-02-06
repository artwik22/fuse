use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Button, FlowBox, Picture, Overlay, gdk};
use gtk4::gio;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;

use crate::core::config::ColorConfig;
use crate::core::quickshell;

fn schedule_notify_color_change_ms(ms: u32) {
    gtk4::glib::timeout_add_local(std::time::Duration::from_millis(ms as u64), move || {
        let _ = quickshell::notify_color_change();
        gtk4::glib::ControlFlow::Break
    });
}

fn color_class_for_preset(color: &str) -> String {
    format!("color-bar-c{}", color.replace("#", "").replace(" ", ""))
}

/// Add one CSS provider for all preset color bars. Call once before building preset cards.
fn add_preset_colors_provider_to_display() {
    let mut colors: std::collections::HashSet<String> = std::collections::HashSet::new();
    for preset in COLOR_PRESETS.iter() {
        // preset: (name, theme, bg, primary, secondary, text, accent)
        colors.insert(preset.2.to_string());
        colors.insert(preset.3.to_string());
        colors.insert(preset.4.to_string());
        colors.insert(preset.5.to_string());
        colors.insert(preset.6.to_string());
    }
    let rules: Vec<String> = colors
        .iter()
        .map(|c| {
            let class = color_class_for_preset(c);
            format!(".{} {{ background-color: {}; }}", class, c)
        })
        .collect();
    let css = rules.join("\n");
    let provider = gtk4::CssProvider::new();
    provider.load_from_string(&css);
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

/// Set background color on a Box by adding its preset color class. add_preset_colors_provider_to_display() must have been called first.
fn set_box_background_color(box_widget: &gtk4::Box, color: &str) {
    box_widget.add_css_class(&color_class_for_preset(color));
}

// 8 new color presets, each with light and dark variants
// Format: (name, theme, background, primary, secondary, text, accent)
const COLOR_PRESETS: &[(&str, &str, &str, &str, &str, &str, &str)] = &[
    // Preset 1: Midnight (Mono)
    ("Midnight (Mono)", "light", "#ffffff", "#f5f5f5", "#e5e5e5", "#000000", "#333333"),
    ("Midnight (Mono)", "dark", "#000000", "#121212", "#080808", "#ffffff", "#c0c0c0"),
    // Preset 2: Gruvbox
    ("Gruvbox", "light", "#fbf1c7", "#f2e5bc", "#ebdbb2", "#3c3836", "#af3a03"),
    ("Gruvbox", "dark", "#282828", "#32302f", "#1d2021", "#ebdbb2", "#d65d0e"),
    // Preset 3: Catppuccin
    ("Catppuccin", "light", "#eff1f5", "#e6e9ef", "#ccd0da", "#4c4f69", "#8839ef"),
    ("Catppuccin", "dark", "#24273a", "#363a4f", "#494d64", "#cad3f5", "#c6a0f6"),
    // Preset 4: Nord
    ("Nord", "light", "#eceff4", "#e5e9f0", "#d8dee9", "#2e3440", "#5e81ac"),
    ("Nord", "dark", "#2e3440", "#3b4252", "#434c5e", "#eceff4", "#88c0d0"),
    // Preset 5: Dracula
    ("Dracula", "light", "#f8f8f2", "#e2e2e2", "#dcdcdc", "#282a36", "#6272a4"),
    ("Dracula", "dark", "#282a36", "#44475a", "#6272a4", "#f8f8f2", "#bd93f9"),
];

pub struct AppearanceTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl AppearanceTab {
    pub fn new(config: Arc<Mutex<ColorConfig>>) -> Self {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_overlay_scrolling(false); // stałe paski przewijania przy małym oknie
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        // Same layout pattern as Network/Bluetooth: vertical, 12px margins, 24px spacing
        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        let title = Label::new(Some("Appearance"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // Single-column layout (like Network/Bluetooth) so it scrolls well at any width
        let style_row = create_style_row(Arc::clone(&config));
        style_row.set_hexpand(true);
        content.append(&style_row);



        let colors_section = create_colors_section(Arc::clone(&config));
        colors_section.set_hexpand(true);
        colors_section.set_margin_top(24);
        content.append(&colors_section);

        let background_section = create_background_section(Arc::clone(&config));
        background_section.set_hexpand(true);
        background_section.set_margin_top(24);
        content.append(&background_section);

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

fn create_style_row(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    // Single row container, with spacing
    let container = GtkBox::new(Orientation::Horizontal, 24);
    // Expand to fill width
    container.set_hexpand(true);
    
    // --- THEME COLUMN ---
    let theme_col = GtkBox::new(Orientation::Vertical, 0);
    theme_col.add_css_class("settings-section");
    theme_col.set_hexpand(true);
    
    // Theme Header
    let theme_header = GtkBox::new(Orientation::Vertical, 0);
    let theme_title = Label::new(Some("Theme"));
    theme_title.add_css_class("section-title");
    theme_title.set_xalign(0.0);
    theme_title.set_margin_start(20);
    theme_title.set_margin_end(20);
    theme_title.set_margin_top(20);
    theme_title.set_margin_bottom(6);
    theme_header.append(&theme_title);

    let theme_desc = Label::new(Some("Select interface mode"));
    theme_desc.add_css_class("section-description");
    theme_desc.set_xalign(0.0);
    theme_desc.set_margin_start(20);
    theme_desc.set_margin_end(20);
    theme_desc.set_margin_bottom(16);
    theme_header.append(&theme_desc);
    theme_col.append(&theme_header);

    // Theme Content (Cards) - Centered
    let cards_container = GtkBox::new(Orientation::Horizontal, 16);
    cards_container.set_margin_start(20);
    cards_container.set_margin_end(20);
    cards_container.set_margin_bottom(20);
    cards_container.set_halign(gtk4::Align::Center);
    cards_container.set_valign(gtk4::Align::Center);

    // Theme Logic
    let current_config = config.lock().unwrap();
    let current_bg = current_config.background.clone();
    drop(current_config);
    
    // Helper function to check if color is light
    let is_light = |color: &str| -> bool {
        let hex = color.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                let brightness = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;
                return brightness > 0.5;
            }
        }
        false
    };
    
    let is_current_light = is_light(&current_bg);
    
    let light_card = create_theme_card("Light", "light", is_current_light);
    let dark_card = create_theme_card("Dark", "dark", !is_current_light);
    
    let light_card_clone = light_card.clone();
    let dark_card_clone = dark_card.clone();
    
    // Connect Light card click
    {
        let dark_ref = dark_card_clone.clone();
        let config_clone = Arc::clone(&config);
        light_card.connect_clicked(move |btn| {
            btn.add_css_class("theme-card-selected");
            dark_ref.remove_css_class("theme-card-selected");
            
            let current_cfg = config_clone.lock().unwrap();
            let preset_name = current_cfg.color_preset.clone().unwrap_or_else(|| "Ocean Breeze".to_string());
            drop(current_cfg);
            
            let light_preset = COLOR_PRESETS.iter()
                .find(|p| p.0 == preset_name && p.1 == "light")
                .or_else(|| COLOR_PRESETS.iter().find(|p| p.1 == "light"));
            
            if let Some(light_preset) = light_preset {
                let mut cfg = ColorConfig::load();
                cfg.update_colors(light_preset.2, light_preset.3, light_preset.4, light_preset.5, light_preset.6);
                cfg.set_preset(light_preset.0);
                if let Err(_e) = cfg.save() {
                } else {
                    *config_clone.lock().unwrap() = cfg.clone();
                    schedule_notify_color_change_ms(300);
                }
            }
        });
    }
    
    // Connect Dark card click
    {
        let light_ref = light_card_clone.clone();
        let config_clone = Arc::clone(&config);
        dark_card.connect_clicked(move |btn| {
            btn.add_css_class("theme-card-selected");
            light_ref.remove_css_class("theme-card-selected");
            
            let current_cfg = config_clone.lock().unwrap();
            let preset_name = current_cfg.color_preset.clone().unwrap_or_else(|| "Ocean Breeze".to_string());
            drop(current_cfg);
            
            let dark_preset = COLOR_PRESETS.iter()
                .find(|p| p.0 == preset_name && p.1 == "dark")
                .or_else(|| COLOR_PRESETS.iter().find(|p| p.1 == "dark"));
            
            if let Some(dark_preset) = dark_preset {
                let mut cfg = ColorConfig::load();
                cfg.update_colors(dark_preset.2, dark_preset.3, dark_preset.4, dark_preset.5, dark_preset.6);
                cfg.set_preset(dark_preset.0);
                if let Err(_e) = cfg.save() {
                } else {
                    *config_clone.lock().unwrap() = cfg.clone();
                    schedule_notify_color_change_ms(300);
                }
            }
        });
    }
    
    cards_container.append(&light_card);
    cards_container.append(&dark_card);
    theme_col.append(&cards_container);
    
    // --- ROUNDING COLUMN ---
    let rounding_col = GtkBox::new(Orientation::Vertical, 0);
    rounding_col.add_css_class("settings-section");
    rounding_col.set_hexpand(true);

    // Rounding Header
    let rounding_header = GtkBox::new(Orientation::Vertical, 0);
    let rounding_title = Label::new(Some("Corner Rounding"));
    rounding_title.add_css_class("section-title");
    rounding_title.set_xalign(0.0);
    rounding_title.set_margin_start(20);
    rounding_title.set_margin_end(20);
    rounding_title.set_margin_top(20);
    rounding_title.set_margin_bottom(6);
    rounding_header.append(&rounding_title);

    let rounding_desc = Label::new(Some("Select corner style"));
    rounding_desc.add_css_class("section-description");
    rounding_desc.set_xalign(0.0);
    rounding_desc.set_margin_start(20);
    rounding_desc.set_margin_end(20);
    rounding_desc.set_margin_bottom(16);
    rounding_header.append(&rounding_desc);
    rounding_col.append(&rounding_header);

    // Rounding Buttons
    let buttons_move = GtkBox::new(Orientation::Horizontal, 12);
    buttons_move.set_margin_start(20);
    buttons_move.set_margin_end(20);
    buttons_move.set_margin_bottom(20);
    buttons_move.set_halign(gtk4::Align::Center);
    buttons_move.set_valign(gtk4::Align::Center);
    
    let current_rounding = config.lock().unwrap().rounding.clone().unwrap_or_else(|| "rounded".to_string());
    let is_rounded = current_rounding == "rounded";
    let is_sharp = current_rounding == "sharp";

    let rounded_button = Button::with_label("Rounded");
    rounded_button.add_css_class("rounding-button");
    if is_rounded {
        rounded_button.add_css_class("suggested-action");
    }
    let sharp_button = Button::with_label("Sharp");
    sharp_button.add_css_class("rounding-button");
    if is_sharp {
        sharp_button.add_css_class("suggested-action");
    }
    
    // Rounding Logic
    {
        let config = Arc::clone(&config);
        let sharp_btn = sharp_button.clone();
        rounded_button.connect_clicked(move |btn| {
            let mut cfg = ColorConfig::load();
            cfg.set_rounding("rounded");
            if let Err(_e) = cfg.save() {
            } else {
                *config.lock().unwrap() = cfg.clone();
                btn.add_css_class("suggested-action");
                sharp_btn.remove_css_class("suggested-action");
                schedule_notify_color_change_ms(200);
            }
        });
    }

    {
        let config = Arc::clone(&config);
        let rounded_btn = rounded_button.clone();
        sharp_button.connect_clicked(move |btn| {
            let mut cfg = ColorConfig::load();
            cfg.set_rounding("sharp");
            if let Err(_e) = cfg.save() {
            } else {
                *config.lock().unwrap() = cfg.clone();
                btn.add_css_class("suggested-action");
                rounded_btn.remove_css_class("suggested-action");
                schedule_notify_color_change_ms(200);
            }
        });
    }
    
    buttons_move.append(&rounded_button);
    buttons_move.append(&sharp_button);
    rounding_col.append(&buttons_move);

    // Append columns
    container.append(&theme_col);
    container.append(&rounding_col);

    container
}

fn create_theme_card(name: &str, theme: &str, is_selected: bool) -> Button {
    let button = Button::new();
    button.add_css_class("theme-card");
    
    if is_selected {
        button.add_css_class("theme-card-selected");
    }

    let card_content = GtkBox::new(Orientation::Vertical, 0);
    card_content.set_hexpand(true);
    card_content.set_vexpand(true);

    // Preview area
    let preview_container = GtkBox::new(Orientation::Vertical, 0);
    preview_container.add_css_class("theme-preview");
    preview_container.add_css_class(&format!("theme-preview-{}", theme));
    preview_container.set_hexpand(true);
    preview_container.set_vexpand(true);

    card_content.append(&preview_container);

    // Theme name label
    let name_label = Label::new(Some(name));
    name_label.add_css_class("theme-name");
    name_label.set_margin_top(12);
    name_label.set_margin_bottom(12);
    card_content.append(&name_label);

    button.set_child(Some(&card_content));

    button
}

fn create_colors_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    add_preset_colors_provider_to_display();

    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");
    section.set_hexpand(true);

    // Determine current theme to show only matching variant
    let current_config = config.lock().unwrap();
    let current_bg = current_config.background.clone();
    drop(current_config);
    
    // Helper function to check if color is light
    let is_light = |color: &str| -> bool {
        let hex = color.trim_start_matches('#');
        if hex.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex[0..2], 16),
                u8::from_str_radix(&hex[2..4], 16),
                u8::from_str_radix(&hex[4..6], 16),
            ) {
                let brightness = (r as f32 * 0.299 + g as f32 * 0.587 + b as f32 * 0.114) / 255.0;
                return brightness > 0.5;
            }
        }
        false
    };
    
    let is_current_light = is_light(&current_bg);
    let theme_label = if is_current_light { "Light" } else { "Dark" };

    // Section header
    let header = GtkBox::new(Orientation::Vertical, 0);
    let section_title = Label::new(Some("Color Presets"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_margin_start(20);
    section_title.set_margin_end(20);
    section_title.set_margin_top(20);
    section_title.set_margin_bottom(6);
    header.append(&section_title);

    let desc = Label::new(Some(&format!("Choose from predefined color schemes (showing {} theme)", theme_label)));
    desc.add_css_class("section-description");
    desc.set_xalign(0.0);
    desc.set_margin_start(20);
    desc.set_margin_end(20);
    desc.set_margin_bottom(16);
    header.append(&desc);

    // Presets container with FlowBox - responsive, better layout
    let flowbox = FlowBox::new();
    flowbox.set_column_spacing(16);
    flowbox.set_row_spacing(16);
    flowbox.set_halign(gtk4::Align::Fill);
    flowbox.set_hexpand(true);
    flowbox.set_vexpand(true);
    // Responsive: adjust columns based on available width (2-4 columns for better display)
    flowbox.set_max_children_per_line(4);
    flowbox.set_min_children_per_line(2);
    flowbox.set_selection_mode(gtk4::SelectionMode::None);
    flowbox.set_homogeneous(true);

    // Group presets by name and create cards with only current theme variant
    let mut preset_groups: HashMap<&str, Vec<(&str, &str, &str, &str, &str, &str, &str)>> = HashMap::new();
    for preset in COLOR_PRESETS.iter() {
        let (name, theme, bg, primary, secondary, text, accent) = *preset;
        preset_groups.entry(name).or_insert_with(Vec::new).push((name, theme, bg, primary, secondary, text, accent));
    }

    for (name, variants) in preset_groups.iter() {
        // Find variant matching current theme
        let matching_variant = if is_current_light {
            variants.iter().find(|v| v.1 == "light")
        } else {
            variants.iter().find(|v| v.1 == "dark")
        };
        
        if let Some(variant) = matching_variant {
            let preset_card = create_preset_card_single(
                name,
                variant.2, variant.3, variant.4, variant.5, variant.6, // colors
                variant.1, // theme name
                Arc::clone(&config),
            );
            flowbox.append(&preset_card);
        }
    }

    let presets_container = GtkBox::new(Orientation::Vertical, 0);
    presets_container.set_margin_start(20);
    presets_container.set_margin_end(20);
    presets_container.set_margin_bottom(20);
    presets_container.append(&flowbox);

    section.append(&header);
    section.append(&presets_container);

    section
}

fn create_preset_card_with_variants(
    name: &str,
    light_bg: &str, light_primary: &str, light_secondary: &str, light_text: &str, light_accent: &str,
    dark_bg: &str, dark_primary: &str, dark_secondary: &str, dark_text: &str, dark_accent: &str,
    config: Arc<Mutex<ColorConfig>>,
) -> Button {
    let button = Button::new();
    button.add_css_class("preset-button");
    button.set_hexpand(true);
    button.set_vexpand(false);
    button.set_can_shrink(true);
    // Make button responsive - allow it to shrink and expand
    button.set_size_request(-1, -1); // No fixed size - fully responsive

    let content = GtkBox::new(Orientation::Vertical, 14);
    content.set_margin_start(18);
    content.set_margin_end(18);
    content.set_margin_top(18);
    content.set_margin_bottom(18);

    // Preset name at the top
    let name_label = Label::new(Some(name));
    name_label.add_css_class("preset-name");
    name_label.set_margin_bottom(14);
    content.append(&name_label);

    // Light variant preview
    let light_label = Label::new(Some("Light"));
    light_label.add_css_class("preset-variant-label");
    light_label.set_margin_bottom(6);
    content.append(&light_label);

    let light_colors = GtkBox::new(Orientation::Vertical, 5);
    let bg_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    bg_bar.set_size_request(-1, 14);
    bg_bar.add_css_class("color-bar");
    bg_bar.add_css_class("color-bar-large");
    set_box_background_color(&bg_bar, light_bg);
    light_colors.append(&bg_bar);

    let primary_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    primary_bar.set_size_request(-1, 14);
    primary_bar.add_css_class("color-bar");
    primary_bar.add_css_class("color-bar-large");
    set_box_background_color(&primary_bar, light_primary);
    light_colors.append(&primary_bar);

    let accent_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    accent_bar.set_size_request(-1, 14);
    accent_bar.add_css_class("color-bar");
    accent_bar.add_css_class("color-bar-large");
    set_box_background_color(&accent_bar, light_accent);
    light_colors.append(&accent_bar);

    content.append(&light_colors);

    // Separator
    let separator = gtk4::Separator::new(Orientation::Horizontal);
    separator.set_margin_top(10);
    separator.set_margin_bottom(10);
    content.append(&separator);

    // Dark variant preview
    let dark_label = Label::new(Some("Dark"));
    dark_label.add_css_class("preset-variant-label");
    dark_label.set_margin_bottom(6);
    content.append(&dark_label);

    let dark_colors = GtkBox::new(Orientation::Vertical, 5);
    let bg_bar_dark = gtk4::Box::new(Orientation::Horizontal, 0);
    bg_bar_dark.set_size_request(-1, 14);
    bg_bar_dark.add_css_class("color-bar");
    bg_bar_dark.add_css_class("color-bar-large");
    set_box_background_color(&bg_bar_dark, dark_bg);
    dark_colors.append(&bg_bar_dark);

    let primary_bar_dark = gtk4::Box::new(Orientation::Horizontal, 0);
    primary_bar_dark.set_size_request(-1, 14);
    primary_bar_dark.add_css_class("color-bar");
    primary_bar_dark.add_css_class("color-bar-large");
    set_box_background_color(&primary_bar_dark, dark_primary);
    dark_colors.append(&primary_bar_dark);

    let accent_bar_dark = gtk4::Box::new(Orientation::Horizontal, 0);
    accent_bar_dark.set_size_request(-1, 14);
    accent_bar_dark.add_css_class("color-bar");
    accent_bar_dark.add_css_class("color-bar-large");
    set_box_background_color(&accent_bar_dark, dark_accent);
    dark_colors.append(&accent_bar_dark);

    content.append(&dark_colors);

    button.set_child(Some(&content));

    // Store dark variant for click handler (light variant stored for potential future use)
    let _light_bg = light_bg.to_string();
    let _light_primary = light_primary.to_string();
    let _light_secondary = light_secondary.to_string();
    let _light_text = light_text.to_string();
    let _light_accent = light_accent.to_string();
    let dark_bg = dark_bg.to_string();
    let dark_primary = dark_primary.to_string();
    let dark_secondary = dark_secondary.to_string();
    let dark_text = dark_text.to_string();
    let dark_accent = dark_accent.to_string();
    let name = name.to_string();

    button.connect_clicked(move |_| {
        // For now, apply dark variant (could add theme selection later)
        let mut cfg = ColorConfig::load();
        cfg.update_colors(&dark_bg, &dark_primary, &dark_secondary, &dark_text, &dark_accent);
        cfg.set_preset(&name);
        if let Err(e) = cfg.save() {
        } else {
            *config.lock().unwrap() = cfg.clone();
            schedule_notify_color_change_ms(200);
        }
    });

    button
}

fn create_preset_card_single(
    name: &str,
    bg: &str, primary: &str, secondary: &str, text: &str, accent: &str,
    theme: &str,
    config: Arc<Mutex<ColorConfig>>,
) -> Button {
    let button = Button::new();
    button.add_css_class("preset-button");
    button.set_hexpand(true);
    button.set_vexpand(false);
    button.set_can_shrink(true);
    button.set_size_request(-1, -1);

    let content = GtkBox::new(Orientation::Vertical, 14);
    content.set_margin_start(18);
    content.set_margin_end(18);
    content.set_margin_top(18);
    content.set_margin_bottom(18);

    // Preset name at the top
    let name_label = Label::new(Some(name));
    name_label.add_css_class("preset-name");
    name_label.set_margin_bottom(14);
    content.append(&name_label);

    // Theme label (Light/Dark)
    let theme_label = Label::new(Some(theme));
    theme_label.add_css_class("preset-variant-label");
    theme_label.set_margin_bottom(6);
    content.append(&theme_label);

    // Color bars preview
    let colors = GtkBox::new(Orientation::Vertical, 5);
    
    let bg_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    bg_bar.set_size_request(-1, 14);
    bg_bar.add_css_class("color-bar");
    bg_bar.add_css_class("color-bar-large");
    set_box_background_color(&bg_bar, bg);
    colors.append(&bg_bar);

    let primary_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    primary_bar.set_size_request(-1, 14);
    primary_bar.add_css_class("color-bar");
    primary_bar.add_css_class("color-bar-large");
    set_box_background_color(&primary_bar, primary);
    colors.append(&primary_bar);

    let accent_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    accent_bar.set_size_request(-1, 14);
    accent_bar.add_css_class("color-bar");
    accent_bar.add_css_class("color-bar-large");
    set_box_background_color(&accent_bar, accent);
    colors.append(&accent_bar);

    content.append(&colors);

    button.set_child(Some(&content));

    // Store colors for click handler
    let bg = bg.to_string();
    let primary = primary.to_string();
    let secondary = secondary.to_string();
    let text = text.to_string();
    let accent = accent.to_string();
    let name = name.to_string();
    let _theme = theme.to_string();

    button.connect_clicked(move |_| {
        let mut cfg = ColorConfig::load();
        cfg.update_colors(&bg, &primary, &secondary, &text, &accent);
        cfg.set_preset(&name);
        if let Err(e) = cfg.save() {
        } else {
            *config.lock().unwrap() = cfg.clone();
            schedule_notify_color_change_ms(200);
        }
    });

    button
}

fn create_background_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");
    section.set_hexpand(true);

    // Section header with title and Show more button
    let header = GtkBox::new(Orientation::Horizontal, 0);
    header.set_margin_start(20);
    header.set_margin_end(20);
    header.set_margin_top(20);
    header.set_margin_bottom(16);
    header.set_valign(gtk4::Align::Center);

    let section_title = Label::new(Some("Wallpapers"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_hexpand(true);
    section_title.set_halign(gtk4::Align::Start);
    header.append(&section_title);

    // Create responsive container and empty FlowBoxes; populate from background
    let grid_container = GtkBox::new(Orientation::Vertical, 12);
    grid_container.set_margin_start(20);
    grid_container.set_margin_end(20);
    grid_container.set_margin_bottom(20);
    grid_container.set_hexpand(true);
    grid_container.set_vexpand(true);

    let flowbox = FlowBox::new();
    flowbox.set_column_spacing(12);
    flowbox.set_row_spacing(12);
    flowbox.set_halign(gtk4::Align::Fill);
    flowbox.set_hexpand(true);
    flowbox.set_vexpand(true);
    flowbox.set_max_children_per_line(3);
    flowbox.set_min_children_per_line(3);
    flowbox.set_selection_mode(gtk4::SelectionMode::None);
    flowbox.set_homogeneous(true);

    let expanded_flowbox = FlowBox::new();
    expanded_flowbox.set_column_spacing(12);
    expanded_flowbox.set_row_spacing(12);
    expanded_flowbox.set_halign(gtk4::Align::Fill);
    expanded_flowbox.set_hexpand(true);
    expanded_flowbox.set_vexpand(true);
    expanded_flowbox.set_max_children_per_line(3);
    expanded_flowbox.set_min_children_per_line(3);
    expanded_flowbox.set_selection_mode(gtk4::SelectionMode::None);
    expanded_flowbox.set_homogeneous(true);
    expanded_flowbox.set_visible(false);

    let flowbox_c = flowbox.clone();
    let expanded_c = expanded_flowbox.clone();
    let config_c = Arc::clone(&config);
    gtk4::glib::MainContext::default().spawn_local(async move {
        let config_for_blocking = Arc::clone(&config_c);
        let (all_wallpapers, current_wallpaper) = gio::spawn_blocking(move || {
            let wallpapers_path = quickshell::get_wallpapers_path();
            let all_wallpapers = find_wallpapers(&wallpapers_path);
            let current_wallpaper = config_for_blocking.lock().unwrap().last_wallpaper.clone();
            (all_wallpapers, current_wallpaper)
        })
        .await
        .expect("spawn_blocking");
        for wallpaper_path in all_wallpapers.iter().take(15) {
            let is_selected = current_wallpaper
                .as_ref()
                .map(|w| w == wallpaper_path.to_string_lossy().as_ref())
                .unwrap_or(false);
            let tile = create_wallpaper_tile(wallpaper_path, is_selected, Arc::clone(&config_c));
            flowbox_c.append(&tile);
        }
        for wallpaper_path in all_wallpapers.iter() {
            let is_selected = current_wallpaper
                .as_ref()
                .map(|w| w == wallpaper_path.to_string_lossy().as_ref())
                .unwrap_or(false);
            let tile = create_wallpaper_tile(wallpaper_path, is_selected, Arc::clone(&config_c));
            expanded_c.append(&tile);
        }
    });

    grid_container.append(&flowbox);

    let show_more_button = Button::with_label("Show more");
    show_more_button.add_css_class("flat");
    show_more_button.add_css_class("expand-wallpapers-button");
    show_more_button.set_halign(gtk4::Align::End);

    let expanded_flowbox_clone = expanded_flowbox.clone();
    let flowbox_clone = flowbox.clone();
    show_more_button.connect_clicked(move |btn| {
        let is_visible = expanded_flowbox_clone.is_visible();
        expanded_flowbox_clone.set_visible(!is_visible);
        flowbox_clone.set_visible(is_visible);
        if is_visible {
            btn.set_label("Show more");
        } else {
            btn.set_label("Show less");
        }
    });

    header.append(&show_more_button);

    grid_container.append(&expanded_flowbox);

    section.append(&header);
    section.append(&grid_container);

    section
}

fn create_wallpaper_tile(path: &PathBuf, is_selected: bool, config: Arc<Mutex<ColorConfig>>) -> Button {
    let button = Button::new();
    button.add_css_class("wallpaper-tile-appearance");
    
    if is_selected {
        button.add_css_class("wallpaper-tile-selected");
    }

    // Use Overlay to add checkmark on selected tile
    let overlay = Overlay::new();
    
    // Picture widget for wallpaper - responsive
    let picture = Picture::new();
    let file = gtk4::gio::File::for_path(path.as_path());
    picture.set_file(Some(&file));
    picture.set_content_fit(gtk4::ContentFit::Cover);
    picture.set_hexpand(true);
    picture.set_vexpand(true);
    picture.set_can_shrink(true);
    overlay.set_child(Some(&picture));

    // Selected indicator (checkmark)
    if is_selected {
        let checkmark_container = GtkBox::new(Orientation::Horizontal, 0);
        checkmark_container.add_css_class("checkmark-container");
        checkmark_container.set_halign(gtk4::Align::End);
        checkmark_container.set_valign(gtk4::Align::End);
        checkmark_container.set_margin_end(8);
        checkmark_container.set_margin_bottom(8);

        let checkmark = Label::new(Some("✓"));
        checkmark.add_css_class("checkmark-icon");
        checkmark_container.append(&checkmark);

        overlay.add_overlay(&checkmark_container);
    }

    button.set_child(Some(&overlay));
    button.set_hexpand(true);
    button.set_vexpand(true);
    button.set_can_shrink(true);
    // Fully responsive - no fixed size
    button.set_size_request(-1, -1);

    let path_str = path.to_string_lossy().to_string();
    button.connect_clicked(move |_| {
        if let Err(_e) = quickshell::set_wallpaper(&path_str) {
        } else {
            // Update config
            let mut cfg = config.lock().unwrap();
            cfg.last_wallpaper = Some(path_str.clone());
            drop(cfg);
            // Note: Save would typically happen here
        }
    });

    button
}

fn find_wallpapers(path: &PathBuf) -> Vec<PathBuf> {
    let mut wallpapers = Vec::new();
    
    if !path.exists() {
        return wallpapers;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif") {
                        wallpapers.push(path);
                    }
                }
            }
        }
    }

    // Sort
    wallpapers.sort();
    // wallpapers.truncate(6); // Removed truncation for 3x5 grid support
    wallpapers
}





