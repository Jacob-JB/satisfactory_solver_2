use log::info;

use eframe::egui;

fn main() -> eframe::Result<()> {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().into(),
        ..Default::default()
    };

    eframe::run_native(
        "Satisfactory Solver",
        options,
        Box::new(|_| Box::<SolverApp>::default()),
    )
}

#[derive(Default)]
struct SolverApp {}

impl eframe::App for SolverApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
        });
    }
}
