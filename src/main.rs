//#![windows_subsystem = "windows"]

use iced::{Application, Settings};
use relm4::RelmApp;
use crate::app::AppModel;

mod app2;
mod models;
mod mtg;
mod app;
mod components;

mod icon_names {
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}

fn main() {
    let app = RelmApp::new("com.shoudev.MtgCardOrganizer");
    relm4_icons::initialize_icons(icon_names::GRESOURCE_BYTES, icon_names::RESOURCE_PREFIX);
    app.run::<AppModel>(());
}
