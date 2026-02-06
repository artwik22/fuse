use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Switch, Button};
use std::sync::{Arc, Mutex};
use std::process::Command;

use crate::core::config::ColorConfig;

#[derive(Debug, Clone)]
pub struct BluetoothDevice {
    pub mac_address: String,
    pub name: String,
    pub _paired: bool,
    pub connected: bool,
}

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
        
        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        let title = Label::new(Some("Bluetooth"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // Bluetooth Power toggle - in a nice card
        let power_section = create_power_section();
        content.append(&power_section);

        // Paired Devices section - in a card
        let paired_section = create_paired_devices_section();
        content.append(&paired_section);

        // Available Devices section - in a card
        let available_section = create_available_devices_section();
        content.append(&available_section);

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

fn create_power_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");
    
    let is_powered = is_bluetooth_powered();
    let toggle_switch = Switch::new();
    toggle_switch.set_active(is_powered);
    toggle_switch.set_halign(gtk4::Align::End);
    toggle_switch.set_hexpand(false);
    toggle_switch.set_valign(gtk4::Align::Center);
    toggle_switch.set_vexpand(false);
    toggle_switch.set_sensitive(true);
    toggle_switch.set_can_focus(true);
    toggle_switch.set_focusable(true);
    
    {
        let toggle_switch_clone = toggle_switch.clone();
        toggle_switch.connect_active_notify(move |toggle| {
            let enabled = toggle.is_active();
            set_bluetooth_powered(enabled);
            toggle_switch_clone.set_sensitive(false);
            let toggle_switch_for_timeout = toggle_switch_clone.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_millis(500), move || {
                toggle_switch_for_timeout.set_sensitive(true);
                gtk4::glib::ControlFlow::Break
            });
        });
    }
    
    let power_row = create_toggle_row_with_switch(
        "Enable Bluetooth",
        if is_powered { "Turn Bluetooth on or off" } else { "Turn Bluetooth on or off" },
        toggle_switch,
    );
    power_row.set_margin_start(18);
    power_row.set_margin_end(18);
    power_row.set_margin_top(18);
    power_row.set_margin_bottom(18);
    section.append(&power_row);
    
    section
}

fn create_paired_devices_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");
    
    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_margin_start(18);
    header.set_margin_end(18);
    header.set_margin_top(18);
    header.set_margin_bottom(12);
    header.set_valign(gtk4::Align::Center);
    
    let title_label = Label::new(Some("Paired Devices"));
    title_label.add_css_class("section-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    title_label.set_hexpand(true);
    header.append(&title_label);
    
    let refresh_button = Button::new();
    refresh_button.set_icon_name("view-refresh-symbolic");
    refresh_button.add_css_class("flat");
    refresh_button.set_halign(gtk4::Align::End);
    refresh_button.set_hexpand(false);
    refresh_button.set_tooltip_text(Some("Refresh device list"));
    
    // Device list container
    let devices_box = GtkBox::new(Orientation::Vertical, 0);
    devices_box.set_margin_start(0);
    devices_box.set_margin_end(0);
    devices_box.set_margin_bottom(18);
    
    {
        let devices_box_clone = devices_box.clone();
        refresh_button.connect_clicked(move |_| {
            refresh_paired_devices(&devices_box_clone);
        });
    }
    
    header.append(&refresh_button);
    section.append(&header);
    refresh_paired_devices(&devices_box);
    section.append(&devices_box);
    
    section
}

fn create_available_devices_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");
    
    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_margin_start(18);
    header.set_margin_end(18);
    header.set_margin_top(18);
    header.set_margin_bottom(12);
    header.set_valign(gtk4::Align::Center);
    
    let title_label = Label::new(Some("Available Devices"));
    title_label.add_css_class("section-title");
    title_label.set_xalign(0.0);
    title_label.set_halign(gtk4::Align::Start);
    title_label.set_hexpand(true);
    header.append(&title_label);
    
    let scan_button = Button::with_label("Scan for Devices");
    scan_button.add_css_class("suggested-action");
    scan_button.set_halign(gtk4::Align::End);
    scan_button.set_hexpand(false);
    
    // Device list container
    let devices_box = GtkBox::new(Orientation::Vertical, 0);
    devices_box.set_margin_start(0);
    devices_box.set_margin_end(0);
    devices_box.set_margin_bottom(18);
    
    {
        let devices_box_clone = devices_box.clone();
        scan_button.connect_clicked(move |btn| {
            btn.set_sensitive(false);
            btn.set_label("Scanning...");
            start_scan(&devices_box_clone);
            let btn_clone = btn.clone();
            gtk4::glib::timeout_add_local(std::time::Duration::from_secs(10), move || {
                btn_clone.set_sensitive(true);
                btn_clone.set_label("Scan for Devices");
                gtk4::glib::ControlFlow::Break
            });
        });
    }
    
    header.append(&scan_button);
    section.append(&header);
    section.append(&devices_box);
    
    section
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
    row.set_can_focus(false);

    let text_box = GtkBox::new(Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk4::Align::Fill);
    text_box.set_can_focus(false);

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

fn create_device_row(device: &BluetoothDevice, is_paired: bool) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(18);
    row.set_margin_end(18);
    row.set_margin_top(0);
    row.set_margin_bottom(0);
    row.set_hexpand(true);
    row.set_halign(gtk4::Align::Fill);
    row.set_valign(gtk4::Align::Center);

    // Device icon/status indicator
    let status_icon = Label::new(Some(if device.connected { "󰤨" } else { "󰤩" }));
    status_icon.add_css_class("device-status-icon");
    status_icon.set_margin_end(12);
    row.append(&status_icon);

    let text_box = GtkBox::new(Orientation::Vertical, 4);
    text_box.set_hexpand(true);
    text_box.set_halign(gtk4::Align::Fill);

    let name_label = Label::new(Some(&device.name));
    name_label.add_css_class("row-title");
    name_label.set_xalign(0.0);
    name_label.set_halign(gtk4::Align::Start);
    text_box.append(&name_label);

    let status_text = if device.connected { "Connected" } else if is_paired { "Paired" } else { "Available" };
    let mac_label = Label::new(Some(&format!("{} • {}", status_text, device.mac_address)));
    mac_label.add_css_class("row-description");
    mac_label.set_xalign(0.0);
    mac_label.set_halign(gtk4::Align::Start);
    text_box.append(&mac_label);

    row.append(&text_box);

    let buttons_box = GtkBox::new(Orientation::Horizontal, 8);
    buttons_box.set_valign(gtk4::Align::Center);
    
    if is_paired {
        if device.connected {
            let disconnect_btn = Button::with_label("Disconnect");
            disconnect_btn.add_css_class("flat");
            disconnect_btn.add_css_class("destructive-action");
            let mac = device.mac_address.clone();
            disconnect_btn.connect_clicked(move |_| {
                disconnect_device(&mac);
            });
            buttons_box.append(&disconnect_btn);
        } else {
            let connect_btn = Button::with_label("Connect");
            connect_btn.add_css_class("flat");
            connect_btn.add_css_class("suggested-action");
            let mac = device.mac_address.clone();
            connect_btn.connect_clicked(move |_| {
                connect_device(&mac);
            });
            buttons_box.append(&connect_btn);
        }
        
        let remove_btn = Button::new();
        remove_btn.set_icon_name("user-trash-symbolic");
        remove_btn.add_css_class("flat");
        remove_btn.add_css_class("destructive-action");
        remove_btn.set_tooltip_text(Some("Remove device"));
        let mac = device.mac_address.clone();
        remove_btn.connect_clicked(move |_| {
            remove_device(&mac);
        });
        buttons_box.append(&remove_btn);
    } else {
        let pair_btn = Button::with_label("Pair");
        pair_btn.add_css_class("flat");
        pair_btn.add_css_class("suggested-action");
        let mac = device.mac_address.clone();
        pair_btn.connect_clicked(move |_| {
            pair_device(&mac);
        });
        buttons_box.append(&pair_btn);
    }
    
    row.append(&buttons_box);
    row
}

// Bluetooth management functions

fn is_bluetooth_powered() -> bool {
    Command::new("bluetoothctl")
        .arg("show")
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.contains("Powered: yes"))
        .unwrap_or(false)
}

fn set_bluetooth_powered(enabled: bool) {
    let state = if enabled { "on" } else { "off" };
    let _ = Command::new("bluetoothctl")
        .arg("power")
        .arg(state)
        .output();
}

fn get_paired_devices() -> Vec<BluetoothDevice> {
    let output = Command::new("bluetoothctl")
        .arg("devices")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());
    
    let mut devices = Vec::new();
    
    if let Some(output) = output {
        for line in output.lines() {
            if line.starts_with("Device ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let mac = parts[1].to_string();
                    let name = parts[2..].join(" ");
                    
                    // Check if connected
                    let connected = is_device_connected(&mac);
                    
                    devices.push(BluetoothDevice {
                        mac_address: mac,
                        name,
                        _paired: true,
                        connected,
                    });
                }
            }
        }
    }
    
    devices
}

fn is_device_connected(mac: &str) -> bool {
    Command::new("bluetoothctl")
        .arg("info")
        .arg(mac)
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.contains("Connected: yes"))
        .unwrap_or(false)
}

fn refresh_paired_devices(container: &GtkBox) {
    // Clear existing devices
    while let Some(child) = container.last_child() {
        container.remove(&child);
    }
    
    let devices = get_paired_devices();
    
    if devices.is_empty() {
        let placeholder = Label::new(Some("No paired devices"));
        placeholder.add_css_class("dim-label");
        placeholder.set_xalign(0.0);
        placeholder.set_margin_start(18);
        placeholder.set_margin_end(18);
        placeholder.set_margin_top(12);
        placeholder.set_margin_bottom(12);
        container.append(&placeholder);
    } else {
        for device in devices {
            let row = create_device_row(&device, true);
            container.append(&row);
        }
    }
}

fn start_scan(container: &GtkBox) {
    // Clear existing devices
    while let Some(child) = container.last_child() {
        container.remove(&child);
    }
    
    // Start scan in background
    let _ = Command::new("bluetoothctl")
        .arg("scan")
        .arg("on")
        .spawn();
    
    // Clone container for use in closure
    let container_clone = container.clone();
    
    // Wait a bit then get devices
    gtk4::glib::timeout_add_local(std::time::Duration::from_secs(2), move || {
        let devices = get_available_devices();
        
        // Clear container
        while let Some(child) = container_clone.last_child() {
            container_clone.remove(&child);
        }
        
        if devices.is_empty() {
            let placeholder = Label::new(Some("No devices found. Make sure Bluetooth is enabled and try scanning again."));
            placeholder.add_css_class("dim-label");
            placeholder.set_xalign(0.0);
            placeholder.set_margin_start(18);
            placeholder.set_margin_end(18);
            placeholder.set_margin_top(12);
            placeholder.set_margin_bottom(12);
            container_clone.append(&placeholder);
        } else {
            for device in devices {
                let row = create_device_row(&device, false);
                container_clone.append(&row);
            }
        }
        
        gtk4::glib::ControlFlow::Break
    });
}

fn get_available_devices() -> Vec<BluetoothDevice> {
    // Get devices that are not paired
    let paired = get_paired_devices();
    let paired_macs: std::collections::HashSet<String> = paired.iter()
        .map(|d| d.mac_address.clone())
        .collect();
    
    let output = Command::new("bluetoothctl")
        .arg("devices")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok());
    
    let mut devices = Vec::new();
    
    if let Some(output) = output {
        for line in output.lines() {
            if line.starts_with("Device ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let mac = parts[1].to_string();
                    if !paired_macs.contains(&mac) {
                        let name = parts[2..].join(" ");
                        devices.push(BluetoothDevice {
                            mac_address: mac,
                            name,
                            _paired: false,
                            connected: false,
                        });
                    }
                }
            }
        }
    }
    
    devices
}

fn connect_device(mac: &str) {
    let _ = Command::new("bluetoothctl")
        .arg("connect")
        .arg(mac)
        .output();
}

fn disconnect_device(mac: &str) {
    let _ = Command::new("bluetoothctl")
        .arg("disconnect")
        .arg(mac)
        .output();
}

fn pair_device(mac: &str) {
    let _ = Command::new("bluetoothctl")
        .arg("pair")
        .arg(mac)
        .output();
}

fn remove_device(mac: &str) {
    let _ = Command::new("bluetoothctl")
        .arg("remove")
        .arg(mac)
        .output();
}
