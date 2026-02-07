# Fuse (Alloy Hub)

**The Premium Configuration Hub for Quickshell.**

[![Rust](https://img.shields.io/badge/Language-Rust-orange?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![GTK4](https://img.shields.io/badge/UI-GTK4-blue?style=for-the-badge&logo=gtk)](https://www.gtk.org/)
[![Libadwaita](https://img.shields.io/badge/Design-Libadwaita-62a0ea?style=for-the-badge&logo=gnome)](https://gnome.pages.gitlab.gnome.org/libadwaita/)

---

## 🖼️ Showcase

<p align="center">
  <!-- <img src="showcase.png" width="800" alt="Fuse Dashboard"/> -->
</p>

---

## 🌟 Overview

**Fuse** is a high-end, modern system settings application designed as the central nervous system for the **Alloy ecosystem**. Built with the performance of **Rust** and the sleek aesthetics of **GTK4/Libadwaita**, Fuse provides a unified and intuitive interface to control every aspect of your Quickshell environment.

## 🚀 Key Features

| Category | Description |
| :--- | :--- |
| **Appearance** | Synchronize system-wide colors, manage dynamic themes, and fine-tune your look. |
| **Wallpapers** | Integrated wallpaper manager with ultra-fast previews and categorization. |
| **System Bar** | Granular control over layout, widgets, and behavior of your desktop bars. |
| **Connectivity** | Modern, responsive interfaces for managing Bluetooth and Network stacks. |
| **Audio Hub** | Professional-grade volume controls and source management. |
| **Notifications** | A unified center for managing alerts, history, and "Do Not Disturb" modes. |
| **Deep Tweaks** | Access core Quickshell engine settings and custom script hooks. |

---

## 📦 Installation

```bash
mkdir ~/.config/alloy
git clone https://github.com/alloy-team/fuse.git ~/.config/alloy/fuse
cd ~/.config/alloy/fuse
./install.sh
```

---

## 🛠️ Performance & Architecture

Fuse is engineered for speed and reliability. By utilizing:
- **`tokio`**: For non-blocking, asynchronous system interactions.
- **`libadwaita`**: To provide a native, responsive, and adaptive UI.
- **`serde`**: For lightning-fast configuration parsing and serialization.

## 🤝 Contribution

We welcome contributions! Whether it's adding new features, improving documentation, or reporting bugs, feel free to open a PR or Issue.

---

<p align="center">
  Built by <b>artwik22</b> & the Alloy Team.
</p>
