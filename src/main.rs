//#![windows_subsystem = "windows"]

use iced::{Application, Settings};
use relm4::RelmApp;
use crate::app::AppModel;

mod app2;
mod models;
mod mtg;
mod app;
mod components;

fn main() {
    let app = RelmApp::new("com.shoudev.MtgCardOrganizer");
    relm4_icons::initialize_icons();
    app.run::<AppModel>(());
}
