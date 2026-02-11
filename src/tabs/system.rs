use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow};
use gtk4::glib;
use gtk4::gio;
use std::sync::{Arc, Mutex};
use std::process::Command;
use std::fs;

use crate::core::config::ColorConfig;

pub struct SystemTab {
    widget: ScrolledWindow,
    _config: Arc<Mutex<ColorConfig>>,
}

impl SystemTab {
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
        let title = Label::new(Some("System Information"));
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

        // --- Hardware Group ---
        add_group_header(&content, "Hardware");
        let hardware_card = GtkBox::new(Orientation::Vertical, 0);
        hardware_card.add_css_class("card");
        
        let cpu_val = Label::new(Some("…"));
        hardware_card.append(&create_info_row("Processor", &cpu_val));
        
        let gpu_val = Label::new(Some("…"));
        hardware_card.append(&create_info_row("Graphics", &gpu_val));
        
        let mem_val = Label::new(Some("…"));
        hardware_card.append(&create_info_row("Memory", &mem_val));
        
        content.append(&hardware_card);

        // --- Software Group ---
        add_group_header(&content, "Software");
        let software_card = GtkBox::new(Orientation::Vertical, 0);
        software_card.add_css_class("card");

        let os_val = Label::new(Some("…"));
        software_card.append(&create_info_row("Operating System", &os_val));

        let wm_val = Label::new(Some("…"));
        software_card.append(&create_info_row("Window Manager", &wm_val));

        let kernel_val = Label::new(Some("…"));
        software_card.append(&create_info_row("Kernel", &kernel_val));

        content.append(&software_card);

        // Async Data Load
        let cpu_c = cpu_val.clone();
        let gpu_c = gpu_val.clone();
        let mem_c = mem_val.clone();
        let os_c = os_val.clone();
        let wm_c = wm_val.clone();
        let kernel_c = kernel_val.clone();
        glib::MainContext::default().spawn_local(async move {
            let data = gio::spawn_blocking(|| {
                (get_cpu_info(), get_gpu_info(), get_memory_info(), get_os_info(), get_wm_de(), get_kernel_info())
            }).await.unwrap_or_default();
            cpu_c.set_text(&data.0);
            gpu_c.set_text(&data.1);
            mem_c.set_text(&data.2);
            os_c.set_text(&data.3);
            wm_c.set_text(&data.4);
            kernel_c.set_text(&data.5);
        });

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

fn create_info_row(label: &str, value_label: &Label) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("card-row");
    row.set_valign(gtk4::Align::Center);

    let l = Label::new(Some(label));
    l.add_css_class("row-title");
    l.set_hexpand(true);
    l.set_halign(gtk4::Align::Start);
    row.append(&l);

    value_label.add_css_class("row-description");
    value_label.set_halign(gtk4::Align::End);
    row.append(value_label);

    row
}

fn get_os_info() -> String {
    if let Ok(c) = fs::read_to_string("/etc/os-release") {
        for l in c.lines() {
            if l.starts_with("PRETTY_NAME=") { return l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string(); }
        }
    }
    Command::new("uname").arg("-s").output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_else(|| "Unknown".to_string())
}

fn get_wm_de() -> String {
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") { if !de.is_empty() { return de; } }
    "Unknown".to_string()
}

fn get_cpu_info() -> String {
    if let Ok(c) = fs::read_to_string("/proc/cpuinfo") {
        for l in c.lines() { if l.starts_with("model name") { return l.split(':').nth(1).unwrap_or("").trim().to_string(); } }
    }
    "Unknown".to_string()
}

fn get_gpu_info() -> String {
    Command::new("sh").arg("-c").arg("lspci | grep -i 'vga\\|3d\\|display' | head -1").output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.split(':').nth(2).map(|v| v.trim().to_string()))
        .unwrap_or_else(|| "Unknown".to_string())
}

fn get_memory_info() -> String {
    if let Ok(c) = fs::read_to_string("/proc/meminfo") {
        for l in c.lines() {
            if l.starts_with("MemTotal:") {
                let kb = l.split_whitespace().nth(1).and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
                return format!("{:.1} GB", kb as f64 / 1024.0 / 1024.0);
            }
        }
    }
    "Unknown".to_string()
}

fn get_kernel_info() -> String {
    Command::new("uname").args(&["-r", "-m"]).output().ok()
        .and_then(|o| String::from_utf8(o.stdout).ok()).unwrap_or_else(|| "Unknown".to_string())
}
