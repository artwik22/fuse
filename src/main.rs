mod app;
mod core;
mod tabs;
mod widgets;
mod window;

use app::FuseApp;
use crate::core::config::ColorConfig;

fn main() {
    let config = ColorConfig::load();
    ColorConfig::apply_scale_env_from_config(&config);
    let app = FuseApp::new(config);
    app.run();
}
