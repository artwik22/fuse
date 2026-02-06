use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Button, Entry, Grid, Separator, gdk};
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
    
    // load_from_data takes &str and returns ()
    css_provider.load_from_data(&css);
    
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

const PRESETS: &[(&str, &str, &str, &str, &str, &str)] = &[
    ("Midnight (Mono)", "#000000", "#121212", "#080808", "#ffffff", "#c0c0c0"),
    ("Gruvbox", "#282828", "#32302f", "#1d2021", "#ebdbb2", "#d65d0e"),
    ("Catppuccin", "#24273a", "#363a4f", "#494d64", "#cad3f5", "#c6a0f6"),
    ("Nord", "#2e3440", "#3b4252", "#434c5e", "#eceff4", "#88c0d0"),
    ("Dracula", "#282a36", "#44475a", "#6272a4", "#f8f8f2", "#bd93f9"),
];

pub struct ColorsTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl ColorsTab {
    pub fn new(config: Arc<Mutex<ColorConfig>>) -> Self {
        let scrolled = ScrolledWindow::new();
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_overlay_scrolling(false); // stałe paski przewijania przy małym oknie
        scrolled.set_hexpand(true);
        scrolled.set_vexpand(true);
        
        let content = GtkBox::new(Orientation::Vertical, 18);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        let title = Label::new(Some("Color Presets"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // Presets grid
        let presets_section = create_presets_section(Arc::clone(&config));
        content.append(&presets_section);

        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        content.append(&separator);

        // Custom colors section
        let custom_section = create_custom_colors_section(Arc::clone(&config));
        content.append(&custom_section);

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

fn create_presets_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 12);

    let section_title = Label::new(Some("Color Presets"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section.append(&section_title);

    let desc = Label::new(Some("Choose from predefined color schemes"));
    desc.add_css_class("section-description");
    desc.set_xalign(0.0);
    section.append(&desc);

    // Use FlowBox instead of Grid for better responsiveness
    let flowbox = gtk4::FlowBox::new();
    flowbox.set_column_spacing(16);
    flowbox.set_row_spacing(16);
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

    let content = GtkBox::new(Orientation::Vertical, 4);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_margin_top(12);
    content.set_margin_bottom(12);

    // Color preview bars - set background colors directly
    let bg_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    bg_bar.set_size_request(-1, 12);
    bg_bar.add_css_class("color-bar");
    set_box_background_color(&bg_bar, bg);
    content.append(&bg_bar);

    let primary_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    primary_bar.set_size_request(-1, 12);
    primary_bar.add_css_class("color-bar");
    set_box_background_color(&primary_bar, primary);
    content.append(&primary_bar);

    let secondary_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    secondary_bar.set_size_request(-1, 12);
    secondary_bar.add_css_class("color-bar");
    set_box_background_color(&secondary_bar, secondary);
    content.append(&secondary_bar);

    let accent_bar = gtk4::Box::new(Orientation::Horizontal, 0);
    accent_bar.set_size_request(-1, 12);
    accent_bar.add_css_class("color-bar");
    set_box_background_color(&accent_bar, accent);
    content.append(&accent_bar);

    // Preset name
    let name_label = Label::new(Some(name));
    name_label.add_css_class("preset-name");
    name_label.set_margin_top(12);
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
            // Update the shared config - avoid full clone, update fields directly
            {
                let mut shared_cfg = config.lock().unwrap();
                shared_cfg.background = cfg.background.clone();
                shared_cfg.primary = cfg.primary.clone();
                shared_cfg.secondary = cfg.secondary.clone();
                shared_cfg.text = cfg.text.clone();
                shared_cfg.accent = cfg.accent.clone();
            }
            // Wait a bit for file to be written and synced to disk
            std::thread::sleep(std::time::Duration::from_millis(200));
            // Notify quickshell about color change
            if let Err(e) = quickshell::notify_color_change() {
            }
        }
    });

    button
}

fn create_custom_colors_section(config: Arc<Mutex<ColorConfig>>) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 12);

    let section_title = Label::new(Some("Custom Colors"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section.append(&section_title);

    let desc = Label::new(Some("Enter custom HEX color values"));
    desc.add_css_class("section-description");
    desc.set_xalign(0.0);
    section.append(&desc);

    let cfg = config.lock().unwrap();
    let bg_entry = create_color_input("Background", &cfg.background, Arc::clone(&config));
    let primary_entry = create_color_input("Primary", &cfg.primary, Arc::clone(&config));
    let secondary_entry = create_color_input("Secondary", &cfg.secondary, Arc::clone(&config));
    let text_entry = create_color_input("Text", &cfg.text, Arc::clone(&config));
    let accent_entry = create_color_input("Accent", &cfg.accent, Arc::clone(&config));
    drop(cfg);

    section.append(&bg_entry);
    section.append(&primary_entry);
    section.append(&secondary_entry);
    section.append(&text_entry);
    section.append(&accent_entry);

    // Apply button
    let apply_button = Button::with_label("Apply Custom Colors");
    apply_button.add_css_class("apply-button");
    apply_button.set_margin_top(24);
    {
        let config = Arc::clone(&config);
        apply_button.connect_clicked(move |_| {
            // Get current color values from config (updated by input fields)
            let current_cfg = config.lock().unwrap();
            let bg = current_cfg.background.clone();
            let primary = current_cfg.primary.clone();
            let secondary = current_cfg.secondary.clone();
            let text = current_cfg.text.clone();
            let accent = current_cfg.accent.clone();
            drop(current_cfg);
            
            // Reload config from disk to preserve existing settings (like sidebar position)
            let mut cfg = ColorConfig::load();
            // Apply the custom colors from input fields
            cfg.update_colors(&bg, &primary, &secondary, &text, &accent);
            cfg.set_preset(""); // Clear preset when using custom
            if let Err(e) = cfg.save() {
            } else {
                // Update the shared config - avoid full clone, update fields directly
                {
                    let mut shared_cfg = config.lock().unwrap();
                    shared_cfg.background = cfg.background.clone();
                    shared_cfg.primary = cfg.primary.clone();
                    shared_cfg.secondary = cfg.secondary.clone();
                    shared_cfg.text = cfg.text.clone();
                    shared_cfg.accent = cfg.accent.clone();
                }
                // Wait a bit for file to be written and synced to disk
                std::thread::sleep(std::time::Duration::from_millis(200));
                // Notify quickshell about color change
                if let Err(e) = quickshell::notify_color_change() {
                }
            }
        });
    }
    section.append(&apply_button);

    section
}

fn create_color_input(
    label: &str,
    initial_value: &str,
    config: Arc<Mutex<ColorConfig>>,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 16);

    // Color preview
    let preview = gtk4::Box::new(Orientation::Horizontal, 0);
    preview.set_size_request(40, 40);
    preview.add_css_class("color-preview");
    row.append(&preview);

    let text_box = GtkBox::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let label_widget = Label::new(Some(label));
    label_widget.add_css_class("color-label");
    label_widget.set_xalign(0.0);
    text_box.append(&label_widget);

    let entry = Entry::new();
    entry.set_text(initial_value);
    entry.add_css_class("color-entry");
    
    {
        let config = Arc::clone(&config);
        let field = match label {
            "Background" => "background",
            "Primary" => "primary",
            "Secondary" => "secondary",
            "Text" => "text",
            "Accent" => "accent",
            _ => return row,
        };
        entry.connect_changed(move |entry| {
            let text = entry.text();
            let mut cfg = config.lock().unwrap();
            match field {
                "background" => cfg.background = text.to_string(),
                "primary" => cfg.primary = text.to_string(),
                "secondary" => cfg.secondary = text.to_string(),
                "text" => cfg.text = text.to_string(),
                "accent" => cfg.accent = text.to_string(),
                _ => {}
            }
            // Auto-save on change (optional - could be debounced)
            // let _ = cfg.save();
        });
    }
    text_box.append(&entry);

    row.append(&text_box);
    row
}
