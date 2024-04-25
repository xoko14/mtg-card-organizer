use app::App;
use iced::{Application, Settings};

mod app;
mod models;
mod mtg;

fn main() {
    let settings = Settings::with_flags(());

    App::run(settings).unwrap();
}
