use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Scale, Button};
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

        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        let title = Label::new(Some("Audio"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // Output Section
        let output_section = create_audio_device_section("Output", "󰓃", true);
        content.append(&output_section);

        // Input Section
        let input_section = create_audio_device_section("Input", "󰍬", false);
        content.append(&input_section);

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

fn create_audio_device_section(title: &str, icon: &str, is_output: bool) -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");
    section.set_hexpand(true);

    // Section Header (Title + Icon in header? Or just Title?)
    // Network tab has header with title 
    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_margin_start(18);
    header.set_margin_end(18);
    header.set_margin_top(18);
    header.set_margin_bottom(12);
    header.set_valign(gtk4::Align::Center);

    let section_title = Label::new(Some(title));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_halign(gtk4::Align::Start);
    section_title.set_hexpand(true);
    header.append(&section_title);
    section.append(&header);

    // Main Row
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(12);
    row.set_margin_end(12);
    row.set_margin_bottom(12);
    // Remove individual margins if settings-row handles padding, but here we add margin to the box itself inside the section
    // Actually, looking at network.ts:
    // create_interface_row creates a row with settings-row class
    // and then adds margin-start/end/bottom to it when appending to container
    
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);
    row.set_valign(gtk4::Align::Center);

    // Icon
    let icon_label = Label::new(Some(icon));
    icon_label.add_css_class("dim-label"); 
    icon_label.set_margin_end(8);
    // icon_label.set_margin_start(12); // Add some padding inside row
    row.append(&icon_label);

    // Device Name Container (Vertical stack for Name + Description if needed)
    let text_box = GtkBox::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);
    text_box.set_valign(gtk4::Align::Center);

    let device_name_str = if is_output {
        get_default_sink().unwrap_or_else(|| "No device".to_string())
    } else {
        get_default_source().unwrap_or_else(|| "No device".to_string())
    };
    
    let name_label = Label::new(Some(&device_name_str));
    name_label.add_css_class("row-title");
    name_label.set_xalign(0.0);
    name_label.set_halign(gtk4::Align::Start);
    // Truncate if too long?
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    
    text_box.append(&name_label);
    row.append(&text_box);

    // Controls Container
    let controls = GtkBox::new(Orientation::Horizontal, 12);
    controls.set_halign(gtk4::Align::End);
    controls.set_valign(gtk4::Align::Center);

    // Mute Button
    let mute_icon = if is_output { "audio-volume-high-symbolic" } else { "audio-input-microphone-symbolic" };
    let mute_button = Button::from_icon_name(mute_icon);
    mute_button.add_css_class("flat");
    mute_button.add_css_class("circular");
    mute_button.set_tooltip_text(Some("Mute"));
    controls.append(&mute_button);

    // Volume Slider
    let volume_scale = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    volume_scale.set_width_request(140);
    volume_scale.set_value(50.0);
    volume_scale.add_css_class("volume-scale");
    
    // Percentage Label
    let volume_label = Label::new(Some("50%"));
    volume_label.add_css_class("dim-label");
    volume_label.set_width_chars(4);
    volume_label.set_xalign(1.0); // Right align text

    let volume_label_clone = volume_label.clone();
    
    if is_output {
        volume_scale.connect_value_changed(move |scale| {
            let value = scale.value() as u32;
            set_default_sink_volume(value);
            volume_label_clone.set_text(&format!("{}%", value));
        });
    } else {
        volume_scale.connect_value_changed(move |scale| {
            let value = scale.value() as u32;
            set_default_source_volume(value);
            volume_label_clone.set_text(&format!("{}%", value));
        });
    }

    controls.append(&volume_scale);
    controls.append(&volume_label);

    row.append(&controls);
    section.append(&row);

    section
}

fn get_default_sink() -> Option<String> {
    Command::new("pactl")
        .arg("get-default-sink")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn get_default_source() -> Option<String> {
    Command::new("pactl")
        .arg("get-default-source")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn set_default_sink_volume(volume: u32) {
    let volume_pa = (volume as f64 * 65536.0 / 100.0) as u32;
    let _ = Command::new("pactl")
        .arg("set-sink-volume")
        .arg("@DEFAULT_SINK@")
        .arg(&volume_pa.to_string())
        .output();
}

fn set_default_source_volume(volume: u32) {
    let volume_pa = (volume as f64 * 65536.0 / 100.0) as u32;
    let _ = Command::new("pactl")
        .arg("set-source-volume")
        .arg("@DEFAULT_SOURCE@")
        .arg(&volume_pa.to_string())
        .output();
}
