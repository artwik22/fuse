use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Orientation, Label, ScrolledWindow, Button, FlowBox};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::fs;

use crate::core::config::ColorConfig;
use crate::core::quickshell;

pub struct WallpapersTab {
    widget: ScrolledWindow,
    config: Arc<Mutex<ColorConfig>>,
    flowbox: FlowBox,
}

impl WallpapersTab {
    pub fn new(config: Arc<Mutex<ColorConfig>>) -> Self {
        let scrolled = ScrolledWindow::new();
        // GNOME spacing: 24px section gap, 18px container margins
        let content = GtkBox::new(Orientation::Vertical, 18);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_hexpand(true);
        content.set_vexpand(true);

        // GNOME: 12px gap in header
        let header = GtkBox::new(Orientation::Horizontal, 12);
        
        let title = Label::new(Some("Select Wallpaper"));
        title.add_css_class("title");
        title.set_xalign(0.0);
        title.set_hexpand(true);
        header.append(&title);

        let refresh_button = Button::with_label("Refresh");
        refresh_button.add_css_class("refresh-button");
        header.append(&refresh_button);

        content.append(&header);

        // GNOME: 12px spacing in flowbox
        let flowbox = FlowBox::new();
        flowbox.set_column_spacing(12);
        flowbox.set_row_spacing(12);
        flowbox.set_halign(gtk4::Align::Fill);
        flowbox.set_hexpand(true);
        flowbox.set_vexpand(true);
        // Responsive: adjust columns based on available width
        // Will show 1-4 columns depending on window size
        flowbox.set_max_children_per_line(4);
        flowbox.set_min_children_per_line(1);
        flowbox.set_selection_mode(gtk4::SelectionMode::None);
        flowbox.set_homogeneous(true); // Make tiles equal size for better grid
        content.append(&flowbox);

        // Only vertical scrolling, no horizontal scrolling
        scrolled.set_policy(gtk4::PolicyType::Automatic, gtk4::PolicyType::Automatic);
        scrolled.set_overlay_scrolling(false);
        scrolled.set_child(Some(&content));

        let tab = Self {
            widget: scrolled,
            config: Arc::clone(&config),
            flowbox,
        };

        // Load wallpapers
        tab.load_wallpapers();

        // Connect refresh button
        {
            let flowbox_clone = tab.flowbox.clone();
            let config_clone = Arc::clone(&tab.config);
            refresh_button.connect_clicked(move |_| {
                load_wallpapers_into_flowbox(&flowbox_clone, &config_clone);
            });
        }

        tab
    }

    pub fn load_wallpapers(&self) {
        let wallpapers_path = quickshell::get_wallpapers_path();
        let wallpapers = find_wallpapers(&wallpapers_path);
        
        // Update title - store count for later use
        let _count = wallpapers.len();
        
        load_wallpapers_into_flowbox(&self.flowbox, &self.config);
        
        // Title update will be handled by the caller if needed
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.widget
    }
}

fn load_wallpapers_into_flowbox(flowbox: &FlowBox, config: &Arc<Mutex<ColorConfig>>) {
    let wallpapers_path = quickshell::get_wallpapers_path();
    let wallpapers = find_wallpapers(&wallpapers_path);


    // Clear existing - remove all children
    let mut child = flowbox.first_child();
    while let Some(c) = child {
        let next = c.next_sibling();
        flowbox.remove(&c);
        child = next;
    }

    // Add wallpapers - FlowBox will automatically wrap based on available width
    for (idx, wallpaper_path) in wallpapers.iter().enumerate() {
        if !wallpaper_path.exists() {
            continue;
        }
        let tile = create_wallpaper_tile(wallpaper_path, Arc::clone(config));
        flowbox.append(&tile);
    }
    
}

fn find_wallpapers(path: &PathBuf) -> Vec<PathBuf> {
    let mut wallpapers = Vec::new();
    
    if !path.exists() {
        return wallpapers;
    }

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif") {
                        wallpapers.push(path);
                    }
                }
            } else if path.is_dir() {
                // Check subdirectories (maxdepth 2)
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_file() {
                            if let Some(ext) = sub_path.extension() {
                                let ext_lower = ext.to_string_lossy().to_lowercase();
                                if matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "webp" | "gif") {
                                    wallpapers.push(sub_path);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    wallpapers.sort();
    wallpapers
}

fn create_wallpaper_tile(path: &PathBuf, _config: Arc<Mutex<ColorConfig>>) -> Button {
    let button = Button::new();
    button.add_css_class("wallpaper-tile");
    
    // Clone path for the closure
    let path_str = path.to_string_lossy().to_string();
    
    // Use Picture widget for better image loading and display
    let picture = gtk4::Picture::new();
    let file = gtk4::gio::File::for_path(path.as_path());
    picture.set_file(Some(&file));
    picture.set_content_fit(gtk4::ContentFit::Cover);
    picture.set_can_shrink(true); // Allow shrinking for responsiveness
    // Responsive scaling - expand to fill available space
    picture.set_halign(gtk4::Align::Fill);
    picture.set_valign(gtk4::Align::Fill);
    picture.set_vexpand(true);
    picture.set_hexpand(true);

    button.set_child(Some(&picture));
    
    // Make button expand to fill available space in FlowBox
    // FlowBox will automatically distribute space based on available width
    button.set_hexpand(true);
    button.set_vexpand(true);
    // Don't set size_request here - let CSS handle minimum sizes for better responsiveness

    button.connect_clicked(move |_| {
        if let Err(e) = quickshell::set_wallpaper(&path_str) {
        }
    });

    button
}
