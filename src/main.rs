#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod entry;
mod fonts;
mod fs;
mod icons;
mod state;
mod theme;
mod ui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("filox")
            .with_inner_size([1200.0, 760.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    eframe::run_native(
        "filox",
        options,
        Box::new(|cc| Ok(Box::new(app::FerroApp::new(cc)))),
    )
}
