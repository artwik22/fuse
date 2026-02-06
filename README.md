# Fuse

**The Unified Configuration Hub for Quickshell.**

Fuse is a modern system settings application designed specifically for Alloy ecosystem. Built with **Rust**, **GTK4**, and **Libadwaita**, it provides a seamless and aesthetic interface for managing all aspects of your Alloy system.

---

## ✨ Features

Fuse serves as the central brain for your Quickshell environment, offering granular control over:

- **🎨 Appearance & Theme**: Synchronize system colors and manage the dynamic CSS engine.
- **🖼️ Wallpaper Management**: Integrated background selector with real-time previews.
- **📊 Bar Configuration**: Customize your system bars, widgets, and layout.
- **🌐 Connectivity**: Modern interfaces for Bluetooth and Network management.
- **🔊 Audio Control**: Fine-tuned volume and source management via a sleek GTK interface.
- **🔔 Notifications**: Unified center for managing system alerts and history.
- **⚙️ System Tweaks**: Access to core Quickshell settings, scripts, and look-and-feel adjustments.

## 🛠️ Tech Stack

- **Language**: [Rust](https://www.rust-lang.org/)
- **UI Framework**: [GTK4](https://www.gtk.org/) with [Libadwaita](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- **Core Libraries**:
  - `tokio`: Asynchronous runtime for system interactions.
  - `serde`: Robust configuration serialization.
  - `glib` & `gio`: Deep integration with the Linux system bus.

## 🚀 Installation

### Using the Install Script
The easiest way to install Fuse is using the provided `install.sh` script:

```bash
git clone https://github.com/alloy-team/fuse.git
cd fuse
chmod +x install.sh
./install.sh
```

This script will:
1. Ensure the Rust toolchain is configured.
2. Build the binary in release mode.
3. Install it to `/usr/local/bin` (or `~/.local/bin` if sudo is not available).

## 🧑‍💻 Development

To build and run Fuse locally for development:

```bash
cargo run
```

### Configuration Path
Fuse stores its configuration in standard environment paths, typically:
- `~/.config/alloy/fuse/`

---

Built with ❤️ by artwik22.
