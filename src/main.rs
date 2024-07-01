use eframe::egui;
use pages::{DefaultPage, Page};

pub mod builder;
pub mod factory;
pub mod pages;
pub mod world;

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
        Box::new(|_| Box::new(SolverApp::new())),
    )
}

struct SolverApp {
    page: Box<dyn Page>,
}

impl SolverApp {
    fn new() -> Self {
        SolverApp {
            page: Box::new(pages::world::LandingPage::new()),
        }
    }
}

impl eframe::App for SolverApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.page = std::mem::replace(&mut self.page, Box::new(DefaultPage)).show(ui);
        });
    }
}
