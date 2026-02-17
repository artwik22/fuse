use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Switch};
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
        
        let content = GtkBox::new(Orientation::Vertical, 0);
        content.set_margin_start(24);
        content.set_margin_end(24);
        content.set_margin_top(24);
        content.set_margin_bottom(48);
        content.set_hexpand(true);
        content.set_vexpand(true);

        // Title
        let title = Label::new(Some("Network Settings"));
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

        // --- Wi-Fi Group ---
        add_group_header(&content, "Wi-Fi");
        let wifi_card = GtkBox::new(Orientation::Vertical, 0);
        wifi_card.add_css_class("card");
        
        let (wifi_toggle_row, wifi_toggle) = create_wifi_toggle_row();
        wifi_card.append(&wifi_toggle_row);

        let wifi_info_row = create_wifi_info_row(wifi_toggle.clone());
        wifi_card.append(&wifi_info_row);

        content.append(&wifi_card);

        // --- Interfaces Group ---
        add_group_header(&content, "Network Interfaces");
        let interfaces_card = GtkBox::new(Orientation::Vertical, 0);
        interfaces_card.add_css_class("card");

        let interfaces_container = GtkBox::new(Orientation::Vertical, 0);
        populate_interfaces(&interfaces_container);
        interfaces_card.append(&interfaces_container);

        content.append(&interfaces_card);

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

fn create_card_row(label: &str, widget: impl IsA<gtk4::Widget>) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    row.set_valign(gtk4::Align::Center);

    let l = Label::new(Some(label));
    l.add_css_class("row-title");
    l.set_hexpand(true);
    l.set_halign(gtk4::Align::Start);
    
    row.append(&l);
    row.append(&widget);
    row
}

fn create_wifi_toggle_row() -> (GtkBox, Switch) {
    let wifi_toggle = Switch::new();
    wifi_toggle.set_valign(gtk4::Align::Center);

    {
        let toggle = wifi_toggle.clone();
        toggle.connect_active_notify(move |t| {
            if t.is_active() { enable_wifi(); }
            else { disable_wifi(); }
        });
    }

    // Initial state
    let toggle_clone = wifi_toggle.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        let enabled = gio::spawn_blocking(is_wifi_enabled).await.unwrap_or(false);
        toggle_clone.set_active(enabled);
    });

    (create_card_row("Enable Wi-Fi", wifi_toggle.clone()), wifi_toggle)
}

fn create_wifi_info_row(_toggle: Switch) -> GtkBox {
    let info_label = Label::new(Some("…"));
    info_label.add_css_class("row-description");
    
    let info_label_clone = info_label.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        let current = gio::spawn_blocking(get_current_wifi).await.unwrap_or_else(|_| "Not connected".to_string());
        info_label_clone.set_text(&current);
    });

    create_card_row("Connected to", info_label)
}


fn populate_interfaces(container: &GtkBox) {
    let loading = Label::new(Some("Ładowanie…"));
    loading.add_css_class("dim-label");
    loading.set_margin_top(12);
    loading.set_margin_bottom(12);
    container.append(&loading);

    let container_clone = container.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        let interfaces = gio::spawn_blocking(get_network_interfaces).await.unwrap_or_default();
        
        while let Some(child) = container_clone.first_child() {
            container_clone.remove(&child);
        }

        if interfaces.is_empty() {
            let p = Label::new(Some("No interfaces found"));
            p.add_css_class("dim-label");
            p.set_margin_top(12);
            p.set_margin_bottom(12);
            container_clone.append(&p);
        } else {
            for interface in interfaces {
                let row = create_interface_row(&interface);
                container_clone.append(&row);
            }
        }
    });
}

fn create_interface_row(interface: &NetworkInterface) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    
    let icon = Label::new(Some(if interface.interface_type == "wifi" { "󰤨" } else { "󰈀" }));
    icon.add_css_class("device-status-icon");
    icon.set_margin_end(8);
    row.append(&icon);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let name = Label::new(Some(&interface.name));
    name.add_css_class("row-title");
    name.set_halign(gtk4::Align::Start);
    text_box.append(&name);

    let status = if interface.is_up {
        format!("Connected • {}", interface.ip_address.as_deref().unwrap_or("No IP"))
    } else {
        "Disconnected".to_string()
    };
    let desc = Label::new(Some(&status));
    desc.add_css_class("row-description");
    desc.set_halign(gtk4::Align::Start);
    text_box.append(&desc);

    row.append(&text_box);
    row
}

#[derive(Debug, Clone)]
struct NetworkInterface {
    name: String,
    interface_type: String,
    is_up: bool,
    ip_address: Option<String>,
}

fn get_network_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();
    if let Ok(output) = Command::new("ip").args(&["-o", "addr", "show"]).output() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            let mut seen = HashSet::new();
            for line in output_str.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let name = parts[1].to_string();
                    if !seen.contains(&name) && name != "lo" {
                        seen.insert(name.clone());
                        let interface_type = if name.starts_with('w') { "wifi" } else { "ethernet" };
                        let ip_address = parts.get(5).map(|s| s.to_string());
                        let is_up = is_interface_up(&name);
                        interfaces.push(NetworkInterface { name, interface_type: interface_type.to_string(), is_up, ip_address });
                    }
                }
            }
        }
    }
    interfaces
}

fn is_interface_up(interface: &str) -> bool {
    Command::new("ip").args(&["link", "show", interface]).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("state UP")).unwrap_or(false)
}

fn is_wifi_enabled() -> bool {
    Command::new("nmcli").args(&["radio", "wifi"]).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().contains("enabled")).unwrap_or(false)
}

fn enable_wifi() { let _ = Command::new("nmcli").args(&["radio", "wifi", "on"]).output(); }
fn disable_wifi() { let _ = Command::new("nmcli").args(&["radio", "wifi", "off"]).output(); }

fn get_current_wifi() -> String {
    Command::new("nmcli").args(&["-t", "-f", "active,ssid", "dev", "wifi"]).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().find(|l| l.starts_with("yes:")).map(|l| l.split(':').nth(1).unwrap_or("Unknown").to_string()))
        .unwrap_or_else(|| "Not connected".to_string())
}
