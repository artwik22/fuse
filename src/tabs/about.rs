use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow};
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
        
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(48);

        // --- Logo Section (Not a card) ---
        let logo_box = GtkBox::new(Orientation::Vertical, 12);
        logo_box.set_margin_bottom(32);
        logo_box.set_halign(gtk4::Align::Center);

        let icon = Label::new(Some("⚙️"));
        icon.set_css_classes(&["about-logo"]);
        logo_box.append(&icon);

        let name = Label::new(Some("Alloy"));
        name.set_css_classes(&["title"]);
        logo_box.append(&name);

        let tagline = Label::new(Some("Modern Desktop Experience"));
        tagline.add_css_class("dim-label");
        logo_box.append(&tagline);

        content.append(&logo_box);

        let add_group_header = |box_: &GtkBox, label: &str| {
            let l = Label::new(Some(label));
            l.add_css_class("group-header");
            l.set_halign(gtk4::Align::Start);
            box_.append(&l);
        };

        // --- Info Group ---
        add_group_header(&content, "System Info");
        let info_card = GtkBox::new(Orientation::Vertical, 0);
        info_card.add_css_class("card");
        info_card.append(&create_info_row("Version", "1.0.0-beta"));
        info_card.append(&create_info_row("Platform", "Linux x86_64"));
        info_card.append(&create_info_row("Engine", "Rust / GTK4 / QML"));
        content.append(&info_card);

        // --- Components Group ---
        add_group_header(&content, "Suite Components");
        let components_card = GtkBox::new(Orientation::Vertical, 0);
        components_card.add_css_class("card");

        let items = vec![
            ("Spark", "󰈙", "Launcher & Shell"),
            ("Fuse", "⚙️", "System Control"),
            ("Blink", "󰉋", "File Browser"),
            ("Vitals", "󰍛", "Resource Monitor"),
        ];

        for (n, i, d) in items {
            components_card.append(&create_component_row(n, i, d));
        }
        content.append(&components_card);

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

fn create_info_row(label: &str, value: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    
    let l = Label::new(Some(label));
    l.add_css_class("row-title");
    l.set_hexpand(true);
    l.set_halign(gtk4::Align::Start);
    row.append(&l);

    let v = Label::new(Some(value));
    v.add_css_class("row-description");
    row.append(&v);

    row
}

fn create_component_row(name: &str, icon: &str, desc: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");

    let i = Label::new(Some(icon));
    i.add_css_class("device-status-icon");
    i.set_margin_end(8);
    row.append(&i);

    let txt = GtkBox::new(Orientation::Vertical, 2);
    txt.set_hexpand(true);

    let n = Label::new(Some(name));
    n.add_css_class("row-title");
    n.set_halign(gtk4::Align::Start);
    txt.append(&n);

    let d = Label::new(Some(desc));
    d.add_css_class("row-description");
    d.set_halign(gtk4::Align::Start);
    txt.append(&d);

    row.append(&txt);
    row
}
