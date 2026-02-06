use libadwaita::prelude::*;
use libadwaita::ApplicationWindow;
use gtk4::{
    Box as GtkBox, Orientation, Label, Stack, ListBox, ListBoxRow, Separator, ScrolledWindow,
    Spinner,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use gtk4::glib;

use crate::core::config::ColorConfig;
use crate::tabs::{appearance::AppearanceTab,
                  system::SystemTab, audio::AudioTab, index::IndexTab, bluetooth::BluetoothTab, network::NetworkTab, notifications::NotificationsTab, about::AboutTab, quickshell::QuickshellTab, scripts::ScriptsTab};

const LAZY_TAB_NAMES: &[&str] = &["network", "appearance", "system"];

pub struct FuseWindow {
    window: ApplicationWindow,
    _config: Arc<Mutex<ColorConfig>>,
    _stack: Stack,
    _lazy_built: Rc<RefCell<HashSet<String>>>,
    _lazy_placeholders: Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
    _lazy_network: Rc<RefCell<Option<NetworkTab>>>,
    _lazy_appearance: Rc<RefCell<Option<AppearanceTab>>>,
    _lazy_system: Rc<RefCell<Option<SystemTab>>>,
}

impl FuseWindow {
    pub fn new(app: &libadwaita::Application, config: &Arc<Mutex<ColorConfig>>) -> Self {
        let config = Arc::clone(config);

        let window = ApplicationWindow::builder()
            .application(app)
            .title("⚙️ Fuse Settings")
            .default_width(1100)
            .default_height(750)
            .resizable(true)
            .build();

        window.set_default_size(1100, 750);
        window.set_size_request(380, 300);

        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_transition_duration(150);
        let stack_clone = stack.clone();

        let main_box = GtkBox::new(Orientation::Horizontal, 0);
        main_box.set_margin_start(0);
        main_box.set_margin_end(0);
        main_box.set_margin_top(0);
        main_box.set_margin_bottom(0);
        main_box.set_hexpand(true);
        main_box.set_vexpand(true);
        main_box.set_homogeneous(false);

        let sidebar = create_custom_sidebar(&stack_clone);
        main_box.append(&sidebar);

        // Placeholder: show "Loading..." so window appears immediately
        let loading_box = GtkBox::new(Orientation::Vertical, 18);
        loading_box.set_halign(gtk4::Align::Center);
        loading_box.set_valign(gtk4::Align::Center);
        loading_box.set_hexpand(true);
        loading_box.set_vexpand(true);
        let spinner = Spinner::new();
        spinner.set_spinning(true);
        spinner.set_size_request(48, 48);
        loading_box.append(&spinner);
        let loading_label = Label::new(Some("Ładowanie…"));
        loading_label.add_css_class("title");
        loading_box.append(&loading_label);
        stack.add_titled(&loading_box, Some("loading"), "Loading");
        stack.set_visible_child_name("loading");

        stack.set_hexpand(true);
        stack.set_vexpand(true);
        stack.set_margin_start(0);
        stack.set_margin_end(12);
        stack.set_margin_top(12);
        stack.set_margin_bottom(12);
        main_box.append(&stack);

        window.set_title(Some("⚙️ Fuse Settings"));
        window.set_content(Some(&main_box));

        let lazy_built = Rc::new(RefCell::new(HashSet::new()));
        let lazy_placeholders = Rc::new(RefCell::new((
            Some(create_lazy_placeholder()),
            Some(create_lazy_placeholder()),
            Some(create_lazy_placeholder()),
        )));
        let lazy_network = Rc::new(RefCell::new(None));
        let lazy_appearance = Rc::new(RefCell::new(None));
        let lazy_system = Rc::new(RefCell::new(None));

        let config_for_notify = Arc::clone(&config);
        let built = Rc::clone(&lazy_built);
        let placeholders_notify = Rc::clone(&lazy_placeholders);
        let ln = Rc::clone(&lazy_network);
        let la = Rc::clone(&lazy_appearance);
        let ls = Rc::clone(&lazy_system);
        stack.connect_closure(
            "notify::visible-child-name",
            false,
            gtk4::glib::closure_local!(move |stack: gtk4::Stack, _pspec: gtk4::glib::ParamSpec| {
                on_visible_child_maybe_build_lazy(
                    &stack,
                    &config_for_notify,
                    &built,
                    &placeholders_notify,
                    &ln,
                    &la,
                    &ls,
                );
            }),
        );

        // Build one tab per idle so the main loop can process events between tabs
        schedule_build_tab(stack.clone(), Arc::clone(&config), 0, Rc::clone(&lazy_placeholders));

        Self {
            window,
            _config: config,
            _stack: stack,
            _lazy_built: lazy_built,
            _lazy_placeholders: lazy_placeholders,
            _lazy_network: lazy_network,
            _lazy_appearance: lazy_appearance,
            _lazy_system: lazy_system,
        }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

fn create_lazy_placeholder() -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 18);
    box_.set_halign(gtk4::Align::Center);
    box_.set_valign(gtk4::Align::Center);
    box_.set_hexpand(true);
    box_.set_vexpand(true);
    let spinner = Spinner::new();
    spinner.set_spinning(true);
    spinner.set_size_request(48, 48);
    box_.append(&spinner);
    let label = Label::new(Some("Ładowanie…"));
    label.add_css_class("title");
    box_.append(&label);
    box_
}

fn on_visible_child_maybe_build_lazy(
    stack: &Stack,
    config: &Arc<Mutex<ColorConfig>>,
    built: &Rc<RefCell<HashSet<String>>>,
    placeholders: &Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
    lazy_network: &Rc<RefCell<Option<NetworkTab>>>,
    lazy_appearance: &Rc<RefCell<Option<AppearanceTab>>>,
    lazy_system: &Rc<RefCell<Option<SystemTab>>>,
) {
    let Some(name) = stack.visible_child_name() else { return };
    let name = name.as_str();
    if !LAZY_TAB_NAMES.contains(&name) {
        return;
    }
    if built.borrow().contains(name) {
        return;
    }
    let c = Arc::clone(config);
    match name {
        "network" => {
            if let Some(old) = stack.child_by_name("network") {
                stack.remove(&old);
            }
            placeholders.borrow_mut().0 = None;
            let t = NetworkTab::new(c);
            stack.add_titled(t.widget(), Some("network"), "󰤨 Network");
            lazy_network.borrow_mut().replace(t);
            built.borrow_mut().insert("network".into());
        }
        "appearance" => {
            if let Some(old) = stack.child_by_name("appearance") {
                stack.remove(&old);
            }
            placeholders.borrow_mut().1 = None;
            let t = AppearanceTab::new(c);
            stack.add_titled(t.widget(), Some("appearance"), "󰋺 Appearance");
            lazy_appearance.borrow_mut().replace(t);
            built.borrow_mut().insert("appearance".into());
        }
        "system" => {
            if let Some(old) = stack.child_by_name("system") {
                stack.remove(&old);
            }
            placeholders.borrow_mut().2 = None;
            let t = SystemTab::new(c);
            stack.add_titled(t.widget(), Some("system"), "󰍛 System");
            lazy_system.borrow_mut().replace(t);
            built.borrow_mut().insert("system".into());
        }
        "scripts" => {
            // Not strictly lazy in same way but good to keep pattern if we want
            // For now, let's just do nothing here as we don't treat it as lazy heavy tab yet?
            // Actually, `schedule_build_tab` handles the initial build loop.
            // If we want it to be built in that loop, we need to update `build_one_tab`.
            // But if we want it lazy, we add it here.
            // Let's assume consistent pattern: update build_one_tab instead.
        }
        _ => {}
    }
}

/// Build one tab per idle callback so the main loop can process events between tabs.
fn schedule_build_tab(
    stack: Stack,
    config: Arc<Mutex<ColorConfig>>,
    index: usize,
    placeholders: Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
) {
    let stack_clone = stack.clone();
    let config_clone = Arc::clone(&config);
    let placeholders_clone = Rc::clone(&placeholders);
    glib::source::idle_add_local_once(move || {
        build_one_tab(&stack_clone, &config_clone, index, &placeholders_clone);
        if index + 1 < 10 {
            schedule_build_tab(stack_clone, config_clone, index + 1, placeholders_clone);
        } else {
            if let Some(loading) = stack_clone.child_by_name("loading") {
                stack_clone.remove(&loading);
            }
            stack_clone.set_visible_child_name("network");
        }
    });
}

fn build_one_tab(
    stack: &Stack,
    config: &Arc<Mutex<ColorConfig>>,
    index: usize,
    placeholders: &Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
) {
    let c = Arc::clone(config);
    match index {
        0 => {
            if let Some(ref ph) = placeholders.borrow().0 {
                stack.add_titled(ph, Some("network"), "󰤨 Network");
            }
        }
        1 => {
            let t = BluetoothTab::new(c);
            stack.add_titled(t.widget(), Some("bluetooth"), "󰂯 Bluetooth");
        }
        2 => {
            if let Some(ref ph) = placeholders.borrow().1 {
                stack.add_titled(ph, Some("appearance"), "󰋺 Appearance");
            }
        }
        3 => {
            let t = AudioTab::new(c);
            stack.add_titled(t.widget(), Some("audio"), "󰕧 Audio");
        }
        4 => {
            let t = IndexTab::new(c);
            stack.add_titled(t.widget(), Some("index"), "󰉋 Index");
        }
        5 => {
            let t = NotificationsTab::new(c);
            stack.add_titled(t.widget(), Some("notifications"), "󰂚 Notifications");
        }
        6 => {
            let t = QuickshellTab::new(c);
            stack.add_titled(t.widget(), Some("quickshell"), "󰍜 QuickShell");
        }
        7 => {
            let t = ScriptsTab::new(c);
            stack.add_titled(t.widget(), Some("scripts"), "󰒓 Scripts");
        }
        8 => {
            if let Some(ref ph) = placeholders.borrow().2 {
                stack.add_titled(ph, Some("system"), "󰍛 System");
            }
        }
        9 => {
            let t = AboutTab::new(c);
            stack.add_titled(t.widget(), Some("about"), "󰋼 About");
        }
        _ => {}
    }
}

fn create_custom_sidebar(stack: &Stack) -> GtkBox {
    // Sidebar: narrower min on small windows so content area gets more space
    let sidebar = GtkBox::new(Orientation::Vertical, 0);
    sidebar.set_size_request(140, -1);
    sidebar.set_hexpand(false);
    sidebar.set_vexpand(true);
    sidebar.add_css_class("sidebar");
    sidebar.set_margin_start(0);
    sidebar.set_margin_end(0);
    sidebar.set_margin_top(0);
    sidebar.set_margin_bottom(0);

    // Create ScrolledWindow for sidebar to enable scrolling in small windows
    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.set_overlay_scrolling(false);
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    scrolled.set_min_content_width(-1);
    scrolled.set_min_content_height(-1);
    scrolled.set_propagate_natural_width(true);
    scrolled.set_propagate_natural_height(true);
    
    // Create ListBox for custom sidebar
    let list_box = ListBox::new();
    list_box.add_css_class("sidebar-listbox");
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    
    // Helper function to create a sidebar row
    let create_row = |name: &str, icon: &str, page_name: &str| -> (ListBoxRow, String) {
        let row = ListBoxRow::new();
        row.add_css_class("sidebar-row");
        
        let hbox = GtkBox::new(Orientation::Horizontal, 0);
        hbox.set_margin_start(12);
        hbox.set_margin_end(12);
        hbox.set_margin_top(6);
        hbox.set_margin_bottom(6);
        hbox.set_halign(gtk4::Align::Fill);
        
        // Icon label (using emoji/icon font)
        let icon_label = Label::new(Some(icon));
        icon_label.add_css_class("sidebar-icon");
        icon_label.set_margin_end(8); // Reduced spacing to 8px
        hbox.append(&icon_label);
        
        // Text label
        let label = Label::new(Some(name));
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);
        hbox.append(&label);
        
        row.set_child(Some(&hbox));
        
        // Connect click to switch stack page
        let stack_clone = stack.clone();
        let page_name_str = page_name.to_string();
        row.connect_activate(move |_| {
            stack_clone.set_visible_child_name(&page_name_str);
        });
        
        (row, page_name.to_string())
    };
    
    // Store page names in order (excluding separators)
    let page_names = vec!["network", "bluetooth", "appearance", "audio", "index", "notifications", "quickshell", "scripts", "system", "about"];
    
    // Add Network and Bluetooth at the top
    let network_row = create_row("Network", "󰤨", "network").0;
    list_box.append(&network_row);
    
    let bluetooth_row = create_row("Bluetooth", "󰂯", "bluetooth").0;
    list_box.append(&bluetooth_row);
    
    // Add separator
    let separator = Separator::new(Orientation::Horizontal);
    separator.set_margin_top(8);
    separator.set_margin_bottom(8);
    separator.set_margin_start(12);
    separator.set_margin_end(12);
    let separator_row = ListBoxRow::new();
    separator_row.add_css_class("sidebar-separator");
    separator_row.set_selectable(false);
    separator_row.set_activatable(false);
    separator_row.set_child(Some(&separator));
    list_box.append(&separator_row);
    
    // Add main tabs
    let appearance_row = create_row("Appearance", "󰋺", "appearance").0;
    list_box.append(&appearance_row);
    
    let audio_row = create_row("Audio", "󰕧", "audio").0;
    list_box.append(&audio_row);
    
    let index_row = create_row("Index", "󰉋", "index").0;
    list_box.append(&index_row);
    
    let notifications_row = create_row("Notifications", "󰂚", "notifications").0;
    list_box.append(&notifications_row);
    
    let quickshell_row = create_row("QuickShell", "󰍜", "quickshell").0;
    list_box.append(&quickshell_row);

    let scripts_row = create_row("Scripts", "󰒓", "scripts").0;
    list_box.append(&scripts_row);
    
    // Add separator before System and About
    let separator2 = Separator::new(Orientation::Horizontal);
    separator2.set_margin_top(8);
    separator2.set_margin_bottom(8);
    separator2.set_margin_start(12);
    separator2.set_margin_end(12);
    let separator_row2 = ListBoxRow::new();
    separator_row2.add_css_class("sidebar-separator");
    separator_row2.set_selectable(false);
    separator_row2.set_activatable(false);
    separator_row2.set_child(Some(&separator2));
    list_box.append(&separator_row2);
    
    // Add System and About at the bottom
    let system_row = create_row("System", "󰍛", "system").0;
    list_box.append(&system_row);
    
    let about_row = create_row("About", "󰋼", "about").0;
    list_box.append(&about_row);
    
    // Connect list box selection to stack using row index
    let stack_clone = stack.clone();
    let page_names_clone = page_names.clone();
    list_box.connect_row_selected(move |_list_box, row| {
        if let Some(row) = row {
            // Get the row index directly (returns i32, not Option)
            let row_index = row.index();
            // Map index to page (skip separators at index 2 and index 8)
            // Sidebar: 0=Network, 1=Bluetooth, 2=Separator, 3=Appearance, 4=Audio, 5=Index, 6=Notifications, 7=QuickShell, 8=Separator, 9=System, 10=About
            let page_idx = if row_index < 2 { 
                row_index as usize
            } else if row_index < 9 {
                (row_index - 1) as usize
            } else {
                (row_index - 2) as usize
            };
            if page_idx < page_names_clone.len() {
                stack_clone.set_visible_child_name(page_names_clone[page_idx]);
            }
        }
    });
    
    // Add ListBox to ScrolledWindow
    scrolled.set_child(Some(&list_box));
    
    // Add ScrolledWindow to sidebar
    sidebar.append(&scrolled);
    
    sidebar
}
