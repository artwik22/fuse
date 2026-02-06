use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Switch, Button};
use gtk4::gio;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::collections::HashSet;

use crate::core::config::ColorConfig;

pub struct NetworkTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl NetworkTab {
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
        let title = Label::new(Some("Network"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // Wi-Fi section
        let wifi_section = create_wifi_section();
        wifi_section.set_hexpand(true);
        content.append(&wifi_section);

        // Network Interfaces section
        let interfaces_section = create_interfaces_section();
        interfaces_section.set_hexpand(true);
        content.append(&interfaces_section);

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

fn create_wifi_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");

    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_margin_start(18);
    header.set_margin_end(18);
    header.set_margin_top(18);
    header.set_margin_bottom(12);
    header.set_valign(gtk4::Align::Center);

    let section_title = Label::new(Some("Wi-Fi"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_halign(gtk4::Align::Start);
    section_title.set_hexpand(true);
    header.append(&section_title);

    let wifi_toggle = Switch::new();
    wifi_toggle.set_halign(gtk4::Align::End);
    wifi_toggle.set_valign(gtk4::Align::Center);
    wifi_toggle.set_hexpand(false);
    wifi_toggle.set_vexpand(false);

    {
        let wifi_toggle_clone = wifi_toggle.clone();
        wifi_toggle.connect_active_notify(move |toggle| {
            let enabled = toggle.is_active();
            if enabled {
                enable_wifi();
            } else {
                disable_wifi();
            }
            let toggle_clone = wifi_toggle_clone.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                toggle_clone.set_active(is_wifi_enabled());
                gtk4::glib::ControlFlow::Break
            });
        });
    }

    header.append(&wifi_toggle);
    section.append(&header);

    let wifi_info_label = Label::new(Some("…"));
    let wifi_info_row = create_info_row_with_label("Connected to", &wifi_info_label);
    wifi_info_row.set_margin_start(18);
    wifi_info_row.set_margin_end(18);
    wifi_info_row.set_margin_bottom(12);
    section.append(&wifi_info_row);

    let wifi_toggle_c = wifi_toggle.clone();
    let wifi_info_c = wifi_info_label.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        let (enabled, current) = gio::spawn_blocking(|| (is_wifi_enabled(), get_current_wifi()))
            .await
            .expect("spawn_blocking");
        wifi_toggle_c.set_active(enabled);
        wifi_info_c.set_text(&current);
    });

    // Scan for networks button
    let scan_button = Button::with_label("Scan for Networks");
    scan_button.add_css_class("flat");
    scan_button.add_css_class("suggested-action");
    scan_button.set_halign(gtk4::Align::Start);
    scan_button.set_margin_start(18);
    scan_button.set_margin_end(18);
    scan_button.set_margin_bottom(18);
    
    scan_button.connect_clicked(move |_| {
        let _ = Command::new("nm-connection-editor")
            .spawn();
    });
    
    section.append(&scan_button);

    section
}

fn create_interfaces_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");

    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_margin_start(18);
    header.set_margin_end(18);
    header.set_margin_top(18);
    header.set_margin_bottom(12);
    header.set_valign(gtk4::Align::Center);

    let section_title = Label::new(Some("Network Interfaces"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_halign(gtk4::Align::Start);
    section_title.set_hexpand(true);
    header.append(&section_title);

    section.append(&header);

    let interfaces_container = GtkBox::new(Orientation::Vertical, 0);
    let loading_label = Label::new(Some("Ładowanie…"));
    loading_label.add_css_class("dim-label");
    loading_label.set_margin_start(18);
    loading_label.set_margin_end(18);
    loading_label.set_margin_top(12);
    loading_label.set_margin_bottom(18);
    interfaces_container.append(&loading_label);
    section.append(&interfaces_container);

    let container_clone = interfaces_container.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        let interfaces = gio::spawn_blocking(get_network_interfaces)
            .await
            .expect("spawn_blocking");
        if let Some(loading) = container_clone.first_child() {
            container_clone.remove(&loading);
        }
        if interfaces.is_empty() {
            let placeholder = Label::new(Some("No network interfaces found"));
            placeholder.add_css_class("dim-label");
            placeholder.set_xalign(0.0);
            placeholder.set_margin_start(18);
            placeholder.set_margin_end(18);
            placeholder.set_margin_top(12);
            placeholder.set_margin_bottom(18);
            container_clone.append(&placeholder);
        } else {
            for (i, interface) in interfaces.into_iter().enumerate() {
                let interface_row = create_interface_row(&interface);
                interface_row.set_margin_start(18);
                interface_row.set_margin_end(18);
                interface_row.set_margin_bottom(0);
                if i == 0 {
                    interface_row.set_margin_top(0);
                }
                container_clone.append(&interface_row);
            }
            if let Some(last_child) = container_clone.last_child() {
                if let Some(row) = last_child.downcast_ref::<GtkBox>() {
                    row.set_margin_bottom(18);
                }
            }
        }
    });

    section
}

fn create_interface_row(interface: &NetworkInterface) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);
    row.set_valign(gtk4::Align::Center);

    // Icon
    let icon = Label::new(Some(if interface.interface_type == "wifi" { "󰤨" } else { "󰈀" }));
    icon.add_css_class("device-status-icon");
    icon.set_margin_end(12);
    row.append(&icon);

    // Text box
    let text_box = GtkBox::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);

    let title_label = Label::new(Some(&interface.name));
    title_label.add_css_class("row-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    text_box.append(&title_label);

    let status_text = if interface.is_up {
        format!("Up - {}", interface.ip_address.as_deref().unwrap_or("No IP"))
    } else {
        "Down".to_string()
    };
    let desc_label = Label::new(Some(&status_text));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    desc_label.set_halign(gtk4::Align::Start);
    text_box.append(&desc_label);

    row.append(&text_box);

    // Status indicator
    let status_text = if interface.is_up { "Connected" } else { "Disconnected" };
    let status_label = Label::new(Some(status_text));
    status_label.add_css_class("row-description");
    status_label.set_markup(&format!(
        "<span foreground='{}'>{}</span>",
        if interface.is_up { "#4ade80" } else { "#ef4444" },
        status_text
    ));
    status_label.set_halign(gtk4::Align::End);
    status_label.set_margin_start(12);
    row.append(&status_label);

    row
}

#[derive(Debug, Clone)]
struct NetworkInterface {
    name: String,
    interface_type: String, // "wifi" or "ethernet"
    is_up: bool,
    ip_address: Option<String>,
}

fn get_network_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();
    
    // Get interfaces using ip command
    if let Ok(output) = Command::new("ip")
        .args(&["-o", "addr", "show"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            let mut seen_interfaces = HashSet::new();
            
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[1].to_string();
                    if !seen_interfaces.contains(&name) && name != "lo" {
                        seen_interfaces.insert(name.clone());
                        
                        let interface_type = if name.starts_with("wlan") || name.starts_with("wlp") || name.starts_with("wifi") {
                            "wifi".to_string()
                        } else {
                            "ethernet".to_string()
                        };
                        
                        let ip_address = if parts.len() >= 6 {
                            Some(parts[5].to_string())
                        } else {
                            None
                        };
                        
                        let is_up = is_interface_up(&name);
                        
                        interfaces.push(NetworkInterface {
                            name,
                            interface_type,
                            is_up,
                            ip_address,
                        });
                    }
                }
            }
        }
    }
    
    interfaces
}

fn is_interface_up(interface: &str) -> bool {
    if let Ok(output) = Command::new("ip")
        .args(&["link", "show", interface])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            return output_str.contains("state UP");
        }
    }
    false
}

fn is_wifi_enabled() -> bool {
    if let Ok(output) = Command::new("nmcli")
        .args(&["radio", "wifi"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            return output_str.trim().contains("enabled");
        }
    }
    false
}

fn enable_wifi() {
    let _ = Command::new("nmcli")
        .args(&["radio", "wifi", "on"])
        .output();
}

fn disable_wifi() {
    let _ = Command::new("nmcli")
        .args(&["radio", "wifi", "off"])
        .output();
}

fn get_current_wifi() -> String {
    if let Ok(output) = Command::new("nmcli")
        .args(&["-t", "-f", "active,ssid", "dev", "wifi"])
        .output()
    {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                if line.starts_with("yes:") {
                    return line.split(':').nth(1).unwrap_or("Not connected").to_string();
                }
            }
        }
    }
    "Not connected".to_string()
}

fn create_toggle_row_with_switch(
    title: &str,
    description: &str,
    toggle: Switch,
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

    let desc_label = Label::new(Some(description));
    desc_label.add_css_class("row-description");
    desc_label.set_xalign(0.0);
    desc_label.set_halign(gtk4::Align::Start);
    text_box.append(&desc_label);

    row.append(&text_box);
    row.append(&toggle);

    row
}

fn create_info_row(title: &str, value: &str) -> GtkBox {
    let value_label = Label::new(Some(value));
    create_info_row_with_label(title, &value_label)
}

fn create_info_row_with_label(title: &str, value_label: &Label) -> GtkBox {
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

    value_label.add_css_class("row-description");
    value_label.set_xalign(0.0);
    value_label.set_halign(gtk4::Align::Start);
    text_box.append(value_label);

    row.append(&text_box);

    row
}
