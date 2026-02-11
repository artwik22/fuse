use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Switch, Button};
use gtk4::gio;
use std::sync::{Arc, Mutex};
use std::process::Command;

use crate::core::config::ColorConfig;

pub struct BluetoothTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl BluetoothTab {
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
        let title = Label::new(Some("Bluetooth Settings"));
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

        // --- Power Group ---
        add_group_header(&content, "Status");
        let power_card = GtkBox::new(Orientation::Vertical, 0);
        power_card.add_css_class("card");
        
        power_card.append(&create_power_toggle_row());
        content.append(&power_card);

        // --- Paired Group ---
        add_group_header(&content, "Paired Devices");
        let paired_card = GtkBox::new(Orientation::Vertical, 0);
        paired_card.add_css_class("card");
        
        let paired_container = GtkBox::new(Orientation::Vertical, 0);
        refresh_paired_devices_async(&paired_container);
        paired_card.append(&paired_container);
        content.append(&paired_card);

        // --- Discover Group ---
        add_group_header(&content, "Discover");
        let discover_card = GtkBox::new(Orientation::Vertical, 0);
        discover_card.add_css_class("card");

        let discover_container = GtkBox::new(Orientation::Vertical, 0);
        discover_card.append(&create_scan_button_row(&discover_container));
        discover_card.append(&discover_container);
        content.append(&discover_card);

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

fn create_power_toggle_row() -> GtkBox {
    let sw = Switch::new();
    sw.set_valign(gtk4::Align::Center);

    {
        sw.connect_active_notify(move |s| {
            set_bluetooth_powered(s.is_active());
        });
    }

    let sw_init = sw.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        let powered = gio::spawn_blocking(is_bluetooth_powered).await.unwrap_or(false);
        sw_init.set_active(powered);
    });

    create_card_row("Bluetooth Power", sw)
}

fn create_scan_button_row(container: &GtkBox) -> GtkBox {
    let btn = Button::with_label("Scan for Devices");
    btn.add_css_class("flat");
    btn.add_css_class("suggested-action");
    btn.set_halign(gtk4::Align::End);
    
    let container_clone = container.clone();
    btn.connect_clicked(move |b| {
        b.set_sensitive(false);
        b.set_label("Scanning...");
        start_scan(&container_clone);
        let b_clone = b.clone();
        gtk4::glib::timeout_add_local(std::time::Duration::from_secs(10), move || {
            b_clone.set_sensitive(true);
            b_clone.set_label("Scan for Devices");
            gtk4::glib::ControlFlow::Break
        });
    });

    create_card_row("Discoverable", btn)
}

fn create_device_row(device: &BluetoothDevice, is_paired: bool) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    
    let icon = Label::new(Some(if device.connected { "󰤨" } else { "󰤩" }));
    icon.add_css_class("device-status-icon");
    icon.set_margin_end(8);
    row.append(&icon);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);

    let name = Label::new(Some(&device.name));
    name.add_css_class("row-title");
    name.set_halign(gtk4::Align::Start);
    text_box.append(&name);

    let status = format!("{} • {}", if device.connected { "Connected" } else if is_paired { "Paired" } else { "Available" }, device.mac_address);
    let desc = Label::new(Some(&status));
    desc.add_css_class("row-description");
    desc.set_halign(gtk4::Align::Start);
    text_box.append(&desc);

    row.append(&text_box);

    let btn_box = GtkBox::new(Orientation::Horizontal, 8);
    if is_paired {
        let connect_btn = Button::with_label(if device.connected { "Disconnect" } else { "Connect" });
        connect_btn.add_css_class("flat");
        let mac = device.mac_address.clone();
        let connected = device.connected;
        connect_btn.connect_clicked(move |_| {
            if connected { disconnect_device(&mac); }
            else { connect_device(&mac); }
        });
        btn_box.append(&connect_btn);
    } else {
        let pair_btn = Button::with_label("Pair");
        pair_btn.add_css_class("flat");
        let mac = device.mac_address.clone();
        pair_btn.connect_clicked(move |_| pair_device(&mac));
        btn_box.append(&pair_btn);
    }

    row.append(&btn_box);
    row
}

#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub mac_address: String,
    pub name: String,
    pub _paired: bool,
    pub connected: bool,
}

fn is_bluetooth_powered() -> bool {
    Command::new("bluetoothctl").arg("show").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("Powered: yes")).unwrap_or(false)
}

fn set_bluetooth_powered(enabled: bool) {
    let _ = Command::new("bluetoothctl").arg("power").arg(if enabled { "on" } else { "off" }).output();
}

fn get_paired_devices() -> Vec<BluetoothDevice> {
    let mut devices = Vec::new();
    if let Ok(output) = Command::new("bluetoothctl").arg("devices").output() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                if line.starts_with("Device ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let mac = parts[1].to_string();
                        let name = parts[2..].join(" ");
                        let connected = is_device_connected(&mac);
                        devices.push(BluetoothDevice { mac_address: mac, name, _paired: true, connected });
                    }
                }
            }
        }
    }
    devices
}

fn is_device_connected(mac: &str) -> bool {
    Command::new("bluetoothctl").arg("info").arg(mac).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.contains("Connected: yes")).unwrap_or(false)
}

fn refresh_paired_devices_async(container: &GtkBox) {
    let loading = Label::new(Some("Ładowanie…"));
    loading.add_css_class("dim-label");
    loading.set_margin_top(12);
    loading.set_margin_bottom(12);
    container.append(&loading);

    let container_clone = container.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        let devices = gio::spawn_blocking(get_paired_devices).await.unwrap_or_default();
        while let Some(child) = container_clone.first_child() { container_clone.remove(&child); }
        if devices.is_empty() {
            let p = Label::new(Some("No paired devices"));
            p.add_css_class("dim-label");
            p.set_margin_top(12);
            p.set_margin_bottom(12);
            container_clone.append(&p);
        } else {
            for d in devices { container_clone.append(&create_device_row(&d, true)); }
        }
    });
}

fn start_scan(container: &GtkBox) {
    let _ = Command::new("bluetoothctl").args(&["scan", "on"]).spawn();
    let container_clone = container.clone();
    gtk4::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        let devices = get_available_devices();
        while let Some(child) = container_clone.first_child() { container_clone.remove(&child); }
        for d in devices { container_clone.append(&create_device_row(&d, false)); }
        gtk4::glib::ControlFlow::Break
    });
}

fn get_available_devices() -> Vec<BluetoothDevice> {
    let paired = get_paired_devices();
    let macs: std::collections::HashSet<_> = paired.iter().map(|d| d.mac_address.clone()).collect();
    let mut devices = Vec::new();
    if let Ok(output) = Command::new("bluetoothctl").arg("devices").output() {
        if let Ok(output_str) = String::from_utf8(output.stdout) {
            for line in output_str.lines() {
                if line.starts_with("Device ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    let mac = parts[1].to_string();
                    if !macs.contains(&mac) {
                        let name = parts[2..].join(" ");
                        devices.push(BluetoothDevice { mac_address: mac, name, _paired: false, connected: false });
                    }
                }
            }
        }
    }
    devices
}

fn connect_device(mac: &str) { let _ = Command::new("bluetoothctl").arg("connect").arg(mac).output(); }
fn disconnect_device(mac: &str) { let _ = Command::new("bluetoothctl").arg("disconnect").arg(mac).output(); }
fn pair_device(mac: &str) { let _ = Command::new("bluetoothctl").arg("pair").arg(mac).output(); }
