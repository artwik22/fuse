use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Separator};
use std::sync::{Arc, Mutex};

use crate::core::config::ColorConfig;

pub struct AboutTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl AboutTab {
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
        let title = Label::new(Some("About Alloy"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // Logo/Icon section
        let logo_section = create_logo_section();
        content.append(&logo_section);

        // Separator
        let separator = Separator::new(Orientation::Horizontal);
        separator.set_margin_start(0);
        separator.set_margin_end(0);
        separator.set_margin_top(18);
        separator.set_margin_bottom(18);
        content.append(&separator);

        // Version and Info section
        let info_section = create_info_section();
        content.append(&info_section);

        // Separator
        let separator2 = Separator::new(Orientation::Horizontal);
        separator2.set_margin_start(0);
        separator2.set_margin_end(0);
        separator2.set_margin_top(18);
        separator2.set_margin_bottom(18);
        content.append(&separator2);

        // Components section
        let components_section = create_components_section();
        content.append(&components_section);

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

fn create_logo_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 16);
    section.add_css_class("content-section");
    section.set_halign(gtk4::Align::Center);

    // Logo/Icon
    let logo = Label::new(Some("⚙️"));
    logo.add_css_class("about-logo");
    logo.set_halign(gtk4::Align::Center);
    section.append(&logo);

    // Name
    let name = Label::new(Some("Alloy"));
    name.add_css_class("about-name");
    name.set_halign(gtk4::Align::Center);
    section.append(&name);

    // Tagline
    let tagline = Label::new(Some("A collection of modern desktop applications"));
    tagline.add_css_class("about-tagline");
    tagline.set_halign(gtk4::Align::Center);
    section.append(&tagline);

    section
}

fn create_info_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 16);
    section.add_css_class("content-section");

    let section_title = Label::new(Some("Information"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section.append(&section_title);

    // Version
    let version_row = create_info_row("Version", "0.8");
    section.append(&version_row);

    // Description
    let desc_text = "Alloy is a collection of modern desktop applications built with Rust and GTK4/libadwaita, \
                     including a customizable launcher, system settings, file manager, and system monitor.";
    let desc_row = create_description_row("Description", desc_text);
    section.append(&desc_row);

    section
}

fn create_components_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 16);
    section.add_css_class("content-section");

    let section_title = Label::new(Some("Components"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section.append(&section_title);

    let components = vec![
        ("Spark", "󰈙", "Application launcher and desktop shell"),
        ("Fuse", "⚙️", "System settings application"),
        ("Blink", "󰉋", "File manager"),
        ("Vitals", "󰍛", "System monitor"),
    ];

    for (name, icon, description) in components {
        let component_row = create_component_row(name, icon, description);
        section.append(&component_row);
    }

    section
}

fn create_component_row(name: &str, icon: &str, description: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(0);
    row.set_margin_end(0);
    row.set_margin_top(0);
    row.set_margin_bottom(0);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);

    // Icon
    let icon_label = Label::new(Some(icon));
    icon_label.set_margin_end(12);
    icon_label.set_size_request(32, -1);
    icon_label.add_css_class("component-icon");
    row.append(&icon_label);

    // Text box
    let text_box = GtkBox::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let title_label = Label::new(Some(name));
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

    row
}

fn create_info_row(
    title: &str,
    value: &str,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(0);
    row.set_margin_end(0);
    row.set_margin_top(0);
    row.set_margin_bottom(0);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk4::Align::Fill);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    text_box.append(&title_label);

    let value_label = Label::new(Some(value));
    value_label.add_css_class("row-description");
    value_label.set_xalign(0.0);
    value_label.set_halign(gtk4::Align::Start);
    text_box.append(&value_label);

    row.append(&text_box);

    row
}

fn create_description_row(
    title: &str,
    description: &str,
) -> GtkBox {
    let row = GtkBox::new(Orientation::Vertical, 8);
    row.add_css_class("settings-row");
    row.set_margin_start(0);
    row.set_margin_end(0);
    row.set_margin_top(0);
    row.set_margin_bottom(0);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);

    let title_label = Label::new(Some(title));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    row.append(&title_label);

    let desc_label = Label::new(Some(description));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    desc_label.set_halign(gtk4::Align::Start);
    desc_label.set_wrap(true);
    desc_label.set_max_width_chars(80);
    row.append(&desc_label);

    row
}
