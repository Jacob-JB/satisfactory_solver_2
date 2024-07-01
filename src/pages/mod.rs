use eframe::egui::Ui;

pub mod builder;
pub mod factory;
pub mod world;

pub trait Page {
    fn show(self: Box<Self>, ui: &mut Ui) -> Box<dyn Page>;
}

pub struct DefaultPage;

impl Page for DefaultPage {
    fn show(self: Box<Self>, _ui: &mut Ui) -> Box<dyn Page> {
        panic!("Default page was reached");
    }
}
