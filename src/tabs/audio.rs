use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Scale};
use std::sync::{Arc, Mutex};
use std::process::Command;

use crate::core::config::ColorConfig;

pub struct AudioTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl AudioTab {
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

        // Title
        let title = Label::new(Some("Audio Settings"));
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

        // --- Output Group ---
        add_group_header(&content, "Output");
        let output_card = GtkBox::new(Orientation::Vertical, 0);
        output_card.add_css_class("card");
        output_card.append(&create_audio_row("Speaker", "󰓃", true));
        content.append(&output_card);

        // --- Input Group ---
        add_group_header(&content, "Input");
        let input_card = GtkBox::new(Orientation::Vertical, 0);
        input_card.add_css_class("card");
        input_card.append(&create_audio_row("Microphone", "󰍬", false));
        content.append(&input_card);

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

fn create_audio_row(label: &str, icon: &str, is_output: bool) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    row.set_valign(gtk4::Align::Center);

    let icon_label = Label::new(Some(icon));
    icon_label.add_css_class("device-status-icon");
    icon_label.set_margin_end(8);
    row.append(&icon_label);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let name_str = if is_output {
        get_default_sink().unwrap_or_else(|| "No Output Device".to_string())
    } else {
        get_default_source().unwrap_or_else(|| "No Input Device".to_string())
    };
    
    let title = Label::new(Some(label));
    title.add_css_class("row-title");
    title.set_halign(gtk4::Align::Start);
    text_box.append(&title);

    let desc = Label::new(Some(&name_str));
    desc.add_css_class("row-description");
    desc.set_halign(gtk4::Align::Start);
    desc.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    text_box.append(&desc);

    row.append(&text_box);

    // Controls
    let controls = GtkBox::new(Orientation::Horizontal, 12);
    
    let vol_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    vol_scale.set_width_request(120);
    vol_scale.set_value(50.0);
    vol_scale.add_css_class("volume-scale");

    let vol_label = Label::new(Some("50%"));
    vol_label.add_css_class("dim-label");
    vol_label.set_width_chars(4);

    let vol_label_clone = vol_label.clone();
    vol_scale.connect_value_changed(move |s| {
        let val = s.value() as u32;
        if is_output { set_default_sink_volume(val); }
        else { set_default_source_volume(val); }
        vol_label_clone.set_text(&format!("{}%", val));
    });

    controls.append(&vol_scale);
    controls.append(&vol_label);
    row.append(&controls);

    row
}

fn get_default_sink() -> Option<String> {
    Command::new("pactl").arg("get-default-sink").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s: String| s.trim().to_string())
}

fn get_default_source() -> Option<String> {
    Command::new("pactl").arg("get-default-source").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok()).map(|s: String| s.trim().to_string())
}

fn set_default_sink_volume(v: u32) {
    let pa = (v as f64 * 65536.0 / 100.0) as u32;
    let _ = Command::new("pactl").args(&["set-sink-volume", "@DEFAULT_SINK@", &pa.to_string()]).output();
}

fn set_default_source_volume(v: u32) {
    let pa = (v as f64 * 65536.0 / 100.0) as u32;
    let _ = Command::new("pactl").args(&["set-source-volume", "@DEFAULT_SOURCE@", &pa.to_string()]).output();
}
