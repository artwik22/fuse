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
        
        // GNOME spacing: 24px section gap, 12px container margins
        let content = GtkBox::new(Orientation::Vertical, 24);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        // Title
        let title = Label::new(Some("System"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_halign(gtk4::Align::Start);
        content.append(&title);

        // System Information section
        let system_info_section = create_system_info_section();
        system_info_section.set_hexpand(true);
        content.append(&system_info_section);

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

fn create_system_info_section() -> GtkBox {
    let section = GtkBox::new(Orientation::Vertical, 0);
    section.add_css_class("settings-section");

    let header = GtkBox::new(Orientation::Horizontal, 12);
    header.set_margin_start(18);
    header.set_margin_end(18);
    header.set_margin_top(18);
    header.set_margin_bottom(12);
    header.set_valign(gtk4::Align::Center);

    let section_title = Label::new(Some("System Information"));
    section_title.add_css_class("section-title");
    section_title.set_xalign(0.0);
    section_title.set_halign(gtk4::Align::Start);
    section_title.set_hexpand(true);
    header.append(&section_title);

    section.append(&header);

    let placeholder = "…";
    let os_value = Label::new(Some(placeholder));
    let wm_value = Label::new(Some(placeholder));
    let cpu_value = Label::new(Some(placeholder));
    let gpu_value = Label::new(Some(placeholder));
    let memory_value = Label::new(Some(placeholder));
    let kernel_value = Label::new(Some(placeholder));

    let os_row = create_info_row_with_value_label("Operating System", &os_value);
    let wm_row = create_info_row_with_value_label("Window Manager / DE", &wm_value);
    let cpu_row = create_info_row_with_value_label("Processor", &cpu_value);
    let gpu_row = create_info_row_with_value_label("Graphics", &gpu_value);
    let memory_row = create_info_row_with_value_label("Memory", &memory_value);
    let kernel_row = create_info_row_with_value_label("Kernel", &kernel_value);

    section.append(&os_row);
    section.append(&wm_row);
    section.append(&cpu_row);
    section.append(&gpu_row);
    section.append(&memory_row);
    section.append(&kernel_row);

    let os_c = os_value.clone();
    let wm_c = wm_value.clone();
    let cpu_c = cpu_value.clone();
    let gpu_c = gpu_value.clone();
    let memory_c = memory_value.clone();
    let kernel_c = kernel_value.clone();
    glib::MainContext::default().spawn_local(async move {
        let data = gio::spawn_blocking(|| {
            (
                get_os_info(),
                get_wm_de(),
                get_cpu_info(),
                get_gpu_info(),
                get_memory_info(),
                get_kernel_info(),
            )
        })
        .await
        .expect("spawn_blocking");
        os_c.set_text(&data.0);
        wm_c.set_text(&data.1);
        cpu_c.set_text(&data.2);
        gpu_c.set_text(&data.3);
        memory_c.set_text(&data.4);
        kernel_c.set_text(&data.5);
    });

    section
}

fn create_info_row_with_value_label(label: &str, value_label: &Label) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(18);
    row.set_margin_end(18);
    row.set_margin_top(0);
    row.set_margin_bottom(0);

    let label_widget = Label::new(Some(label));
    label_widget.add_css_class("row-title");
    label_widget.set_xalign(0.0);
    label_widget.set_halign(gtk4::Align::Start);
    label_widget.set_hexpand(true);
    row.append(&label_widget);

    value_label.add_css_class("row-description");
    value_label.set_xalign(1.0);
    value_label.set_halign(gtk4::Align::End);
    value_label.set_hexpand(false);
    row.append(value_label);

    row
}

fn create_info_row(label: &str, value: &str) -> GtkBox {
    let row = GtkBox::new(Orientation::Horizontal, 12);
    row.add_css_class("settings-row");
    row.set_margin_start(18);
    row.set_margin_end(18);
    row.set_margin_top(0);
    row.set_margin_bottom(0);

    let label_widget = Label::new(Some(label));
    label_widget.add_css_class("row-title");
    label_widget.set_xalign(0.0);
    label_widget.set_halign(gtk4::Align::Start);
    label_widget.set_hexpand(true);
    row.append(&label_widget);

    let value_widget = Label::new(Some(value));
    value_widget.add_css_class("row-description");
    value_widget.set_xalign(1.0);
    value_widget.set_halign(gtk4::Align::End);
    value_widget.set_hexpand(false);
    row.append(&value_widget);

    row
}

fn get_os_info() -> String {
    // Try /etc/os-release first
    if let Ok(content) = fs::read_to_string("/etc/os-release") {
        let mut name = String::new();
        let mut version = String::new();
        
        for line in content.lines() {
            if line.starts_with("PRETTY_NAME=") {
                name = line.trim_start_matches("PRETTY_NAME=")
                    .trim_matches('"')
                    .to_string();
            } else if line.starts_with("VERSION=") && version.is_empty() {
                version = line.trim_start_matches("VERSION=")
                    .trim_matches('"')
                    .to_string();
            }
        }
        
        if !name.is_empty() {
            return name;
        }
    }
    
    // Fallback to uname
    if let Ok(output) = Command::new("uname").arg("-s").output() {
        if let Ok(os) = String::from_utf8(output.stdout) {
            return os.trim().to_string();
        }
    }
    
    "Unknown".to_string()
}

fn get_wm_de() -> String {
    // Try XDG_CURRENT_DESKTOP first
    if let Ok(de) = std::env::var("XDG_CURRENT_DESKTOP") {
        if !de.is_empty() {
            return de;
        }
    }
    
    // Try WAYLAND_DISPLAY or X11
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        // Try to detect window manager
        if let Ok(output) = Command::new("ps")
            .args(&["-e", "-o", "comm="])
            .output() {
            let processes = String::from_utf8_lossy(&output.stdout);
            for wm in &["hyprland", "sway", "river", "wayfire", "kwin_wayland"] {
                if processes.contains(wm) {
                    return wm.to_string();
                }
            }
        }
        return "Wayland".to_string();
    }
    
    // Try to detect X11 window manager using ps
    if let Ok(output) = Command::new("ps")
        .args(&["-e", "-o", "comm="])
        .output() {
        let processes = String::from_utf8_lossy(&output.stdout);
        for wm in &["hyprland", "sway", "river", "wayfire", "kwin_wayland", "gnome-shell", "kwin_x11", "xfwm4", "openbox", "i3", "dwm", "bspwm"] {
            if processes.contains(wm) {
                return wm.to_string();
            }
        }
    }
    
    "Unknown".to_string()
}

fn get_cpu_info() -> String {
    // Try /proc/cpuinfo
    if let Ok(content) = fs::read_to_string("/proc/cpuinfo") {
        for line in content.lines() {
            if line.starts_with("model name") {
                if let Some(name) = line.split(':').nth(1) {
                    return name.trim().to_string();
                }
            } else if line.starts_with("Model") && !line.contains("name") {
                if let Some(name) = line.split(':').nth(1) {
                    return name.trim().to_string();
                }
            }
        }
    }
    
    // Fallback to lscpu
    if let Ok(output) = Command::new("lscpu")
        .args(&["--format=cpu,Model name"])
        .output() {
        if let Ok(content) = String::from_utf8(output.stdout) {
            for line in content.lines() {
                if line.contains("Model name") {
                    if let Some(name) = line.split(':').nth(1) {
                        return name.trim().to_string();
                    }
                }
            }
        }
    }
    
    "Unknown".to_string()
}

fn get_gpu_info() -> String {
    // Try nvidia-smi first
    if let Ok(output) = Command::new("nvidia-smi")
        .args(&["--query-gpu=name", "--format=csv,noheader"])
        .output() {
        if let Ok(gpu) = String::from_utf8(output.stdout) {
            let gpu = gpu.trim();
            if !gpu.is_empty() {
                return gpu.to_string();
            }
        }
    }
    
    // Try lspci for GPU
    if let Ok(output) = Command::new("sh")
        .args(&["-c", "lspci | grep -i 'vga\\|3d\\|display' | head -1"])
        .output() {
        if let Ok(content) = String::from_utf8(output.stdout) {
            let content = content.trim();
            if !content.is_empty() {
                // Extract GPU name (everything after the last colon)
                if let Some(gpu) = content.split(':').nth(2) {
                    return gpu.trim().to_string();
                }
            }
        }
    }
    
    // Try /sys/class/drm (simpler approach)
    if let Ok(entries) = fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with("card") && !name_str.contains("-") {
                // Try to read device name from sysfs
                let device_path = entry.path().join("device/uevent");
                if let Ok(content) = fs::read_to_string(&device_path) {
                    // Look for PCI_ID or other identifiers
                    for line in content.lines() {
                        if line.starts_with("PCI_ID") {
                            // Found a GPU device, try lspci with the card number
                            if let Some(_card_num) = name_str.strip_prefix("card") {
                                if let Ok(lspci_output) = Command::new("sh")
                                    .args(&["-c", "lspci | grep -i 'vga\\|3d\\|display' | head -1"])
                                    .output() {
                                    if let Ok(lspci_content) = String::from_utf8(lspci_output.stdout) {
                                        if let Some(gpu) = lspci_content.split(':').nth(2) {
                                            return gpu.trim().to_string();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    "Unknown".to_string()
}

fn get_memory_info() -> String {
    // Try /proc/meminfo
    if let Ok(content) = fs::read_to_string("/proc/meminfo") {
        let mut total_kb = 0u64;
        for line in content.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb_str) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb_str.parse::<u64>() {
                        total_kb = kb;
                        break;
                    }
                }
            }
        }
        
        if total_kb > 0 {
            let total_gb = total_kb as f64 / 1024.0 / 1024.0;
            return format!("{:.1} GB", total_gb);
        }
    }
    
    "Unknown".to_string()
}

fn get_kernel_info() -> String {
    // Use uname
    if let Ok(output) = Command::new("uname")
        .args(&["-r", "-m"])
        .output() {
        if let Ok(info) = String::from_utf8(output.stdout) {
            return info.trim().to_string();
        }
    }
    
    "Unknown".to_string()
}
