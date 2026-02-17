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
                  system::SystemTab, audio::AudioTab, blink::BlinkTab, bluetooth::BluetoothTab, network::NetworkTab, notifications::NotificationsTab, about::AboutTab, 
                  quickshell::QuickshellTab, quickshell_sidebar::QuickshellSidebarTab, quickshell_dashboard::QuickshellDashboardTab,
                  scripts::ScriptsTab, lockscreen::LockScreenTab};

const LAZY_TAB_NAMES: &[&str] = &["network", "appearance", "system", "lockscreen"];

pub struct FuseWindow {
    window: ApplicationWindow,
    _config: Arc<Mutex<ColorConfig>>,
    _stack: Stack,
    _lazy_built: Rc<RefCell<HashSet<String>>>,
    _lazy_placeholders: Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
    _lazy_network: Rc<RefCell<Option<NetworkTab>>>,
    _lazy_appearance: Rc<RefCell<Option<AppearanceTab>>>,

    _lazy_system: Rc<RefCell<Option<SystemTab>>>,
    _lazy_lockscreen: Rc<RefCell<Option<LockScreenTab>>>,
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
            Some(create_lazy_placeholder()),
        )));
        let lazy_network = Rc::new(RefCell::new(None));
        let lazy_appearance = Rc::new(RefCell::new(None));
        let lazy_system = Rc::new(RefCell::new(None));
        let lazy_lockscreen = Rc::new(RefCell::new(None));

        let config_for_notify = Arc::clone(&config);
        let built = Rc::clone(&lazy_built);
        let placeholders_notify = Rc::clone(&lazy_placeholders);
        let ln = Rc::clone(&lazy_network);
        let la = Rc::clone(&lazy_appearance);

        let ls = Rc::clone(&lazy_system);
        let ll = Rc::clone(&lazy_lockscreen);
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
                    &ll,
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
            _lazy_lockscreen: lazy_lockscreen,
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
    placeholders: &Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
    lazy_network: &Rc<RefCell<Option<NetworkTab>>>,
    lazy_appearance: &Rc<RefCell<Option<AppearanceTab>>>,

    lazy_system: &Rc<RefCell<Option<SystemTab>>>,
    lazy_lockscreen: &Rc<RefCell<Option<LockScreenTab>>>,
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
            built.borrow_mut().insert("system".into());
        }
        "lockscreen" => {
            if let Some(old) = stack.child_by_name("lockscreen") {
                stack.remove(&old);
            }
            placeholders.borrow_mut().3 = None;
            let t = LockScreenTab::new(c);
            stack.add_titled(t.widget(), Some("lockscreen"), "🔒 Lock Screen");
            lazy_lockscreen.borrow_mut().replace(t);
            built.borrow_mut().insert("lockscreen".into());
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
    placeholders: Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
) {
    let stack_clone = stack.clone();
    let config_clone = Arc::clone(&config);
    let placeholders_clone = Rc::clone(&placeholders);
    glib::source::idle_add_local_once(move || {
        build_one_tab(&stack_clone, &config_clone, index, &placeholders_clone);
        if index + 1 < 13 {
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
    placeholders: &Rc<RefCell<(Option<GtkBox>, Option<GtkBox>, Option<GtkBox>, Option<GtkBox>)>>,
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
            let t = BlinkTab::new(c);
            stack.add_titled(t.widget(), Some("blink"), "󰉋 Blink");
        }
        5 => {
            let t = NotificationsTab::new(c);
            stack.add_titled(t.widget(), Some("notifications"), "󰂚 Notifications");
        }
        6 => {
            let t = QuickshellTab::new(c);
            stack.add_titled(t.widget(), Some("quickshell"), "󰍜 General");
        }
        7 => {
            let t = QuickshellSidebarTab::new(c);
            stack.add_titled(t.widget(), Some("quickshell_sidebar"), "󰍜 Sidebar");
        }
        8 => {
            let t = QuickshellDashboardTab::new(c);
            stack.add_titled(t.widget(), Some("quickshell_dashboard"), "󰍜 Dashboard");
        }
        9 => {
            let t = ScriptsTab::new(c);
            stack.add_titled(t.widget(), Some("scripts"), "󰒓 Scripts");
        }
        10 => {
            if let Some(ref ph) = placeholders.borrow().3 {
                stack.add_titled(ph, Some("lockscreen"), "🔒 Lock Screen");
            }
        }
        11 => {
            if let Some(ref ph) = placeholders.borrow().2 {
                stack.add_titled(ph, Some("system"), "󰍛 System");
            }
        }
        12 => {
            let t = AboutTab::new(c);
            stack.add_titled(t.widget(), Some("about"), "󰋼 About");
        }
        _ => {}
    }
}

fn create_custom_sidebar(stack: &Stack) -> GtkBox {
    // Sidebar: narrower min on small windows so content area gets more space
    let sidebar = GtkBox::new(Orientation::Vertical, 0);
    sidebar.set_size_request(160, -1); // Slightly wider for Polish text and icons
    sidebar.set_hexpand(false);
    sidebar.set_vexpand(true);
    sidebar.add_css_class("sidebar");
    
    let scrolled = ScrolledWindow::new();
    scrolled.set_policy(gtk4::PolicyType::Never, gtk4::PolicyType::Automatic);
    scrolled.set_overlay_scrolling(false);
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    
    let list_box = ListBox::new();
    list_box.add_css_class("sidebar-listbox");
    list_box.set_selection_mode(gtk4::SelectionMode::Single);
    list_box.set_activate_on_single_click(true);
    
    // Helper to create a row
    let create_row = |name: &str, icon: &str, page_name: &str, is_sub: bool| -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("sidebar-row");
        if is_sub {
            row.add_css_class("sidebar-sub-row");
        }
        
        let hbox = GtkBox::new(Orientation::Horizontal, 0);
        hbox.set_margin_start(if is_sub { 24 } else { 12 });
        hbox.set_margin_end(12);
        hbox.set_margin_top(6);
        hbox.set_margin_bottom(6);
        
        let icon_label = Label::new(Some(icon));
        icon_label.add_css_class("sidebar-icon");
        icon_label.set_margin_end(8);
        hbox.append(&icon_label);
        
        let label = Label::new(Some(name));
        label.set_halign(gtk4::Align::Start);
        label.set_hexpand(true);
        hbox.append(&label);

        if !is_sub && page_name == "quickshell_group" {
            let arrow = Label::new(Some("󰅂")); // Down arrow icon
            arrow.add_css_class("sidebar-arrow");
            hbox.append(&arrow);
        }
        
        row.set_child(Some(&hbox));
        row
    };

    // Add items
    list_box.append(&create_row("Network", "󰤨", "network", false));
    list_box.append(&create_row("Bluetooth", "󰂯", "bluetooth", false));

    let sep1 = Separator::new(Orientation::Horizontal);
    sep1.set_margin_top(8); sep1.set_margin_bottom(8); sep1.set_margin_start(12); sep1.set_margin_end(12);
    let sep_row1 = ListBoxRow::new();
    sep_row1.set_selectable(false); sep_row1.set_activatable(false);
    sep_row1.set_child(Some(&sep1));
    list_box.append(&sep_row1);

    list_box.append(&create_row("Appearance", "󰋺", "appearance", false));
    list_box.append(&create_row("Audio", "󰕧", "audio", false));
    list_box.append(&create_row("Blink", "󰉋", "blink", false));
    list_box.append(&create_row("Notifications", "󰂚", "notifications", false));

    // Quickshell Group
    let qs_head = create_row("QuickShell", "󰍜", "quickshell_group", false);
    list_box.append(&qs_head);

    let qs_gen = create_row("General", "󰇄", "quickshell", true);
    let qs_side = create_row("Sidebar", "󰕮", "quickshell_sidebar", true);
    let qs_dash = create_row("Dashboard", "󰕰", "quickshell_dashboard", true);
    let qs_lock = create_row("Lock Screen", "🔒", "lockscreen", true);

    list_box.append(&qs_gen);
    list_box.append(&qs_side);
    list_box.append(&qs_dash);
    list_box.append(&qs_lock);

    // Initial state: collapsed
    let sub_rows = vec![qs_gen.clone(), qs_side.clone(), qs_dash.clone(), qs_lock.clone()];
    for sub in &sub_rows {
        sub.set_visible(false);
    }
    
    unsafe { qs_head.set_data("expanded", false); } // Start collapsed

    list_box.append(&create_row("Scripts", "󰒓", "scripts", false));

    let sep2 = Separator::new(Orientation::Horizontal);
    sep2.set_margin_top(8); sep2.set_margin_bottom(8); sep2.set_margin_start(12); sep2.set_margin_end(12);
    let sep_row2 = ListBoxRow::new();
    sep_row2.set_selectable(false); sep_row2.set_activatable(false);
    sep_row2.set_child(Some(&sep2));
    list_box.append(&sep_row2);

    list_box.append(&create_row("System", "󰍛", "system", false));
    list_box.append(&create_row("About", "󰋼", "about", false));

    // Mapping indices to page names
    let page_map = vec![
        Some("network"),            // 0
        Some("bluetooth"),          // 1
        None,                       // 2 (Separator)
        Some("appearance"),         // 3
        Some("audio"),              // 4
        Some("blink"),              // 5
        Some("notifications"),      // 6
        None,                       // 7 (QS Head)
        Some("quickshell"),         // 8
        Some("quickshell_sidebar"), // 9
        Some("quickshell_dashboard"),// 10
        Some("lockscreen"),         // 11
        Some("scripts"),            // 12
        None,                       // 13 (Separator)
        Some("system"),             // 14
        Some("about"),              // 15
    ];

    let stack_clone = stack.clone();
    let sub_rows_for_sel = sub_rows.clone();
    let qs_head_for_sel = qs_head.clone();
    
    list_box.connect_row_selected(move |_, row| {
        if let Some(row) = row {
            let idx = row.index() as usize;
            
            // Auto-collapse logic
            let is_sub = idx >= 8 && idx <= 11;
            let is_header = idx == 7;
            let is_sep = idx == 2 || idx == 13;
            
            if is_header {
                // Toggle expansion when header is clicked
                let is_expanded = unsafe { qs_head_for_sel.data::<bool>("expanded").map(|b| *b.as_ref()).unwrap_or(false) };
                let new_state = !is_expanded;
                for sub in &sub_rows_for_sel {
                    sub.set_visible(new_state);
                }
                unsafe { qs_head_for_sel.set_data("expanded", new_state); }
                if let Some(hbox) = qs_head_for_sel.child().and_then(|c| c.downcast::<GtkBox>().ok()) {
                    if let Some(arrow) = hbox.last_child().and_then(|c| c.downcast::<Label>().ok()) {
                        arrow.set_text(if new_state { "󰅂" } else { "󰅃" });
                    }
                }
            } else if !is_sub && !is_sep {
                // Collapse QS group if other top-level items are selected
                for sub in &sub_rows_for_sel {
                    sub.set_visible(false);
                }
                unsafe { qs_head_for_sel.set_data("expanded", false); }
                if let Some(hbox) = qs_head_for_sel.child().and_then(|c| c.downcast::<GtkBox>().ok()) {
                    if let Some(arrow) = hbox.last_child().and_then(|c| c.downcast::<Label>().ok()) {
                        arrow.set_text("󰅃");
                    }
                }
            }

            if idx < page_map.len() {
                if let Some(page) = page_map[idx] {
                    stack_clone.set_visible_child_name(page);
                }
            }
        }
    });

    // We can remove the old qs_head.connect_activate as we handled it in row_selected

    scrolled.set_child(Some(&list_box));
    sidebar.append(&scrolled);
    sidebar
}
