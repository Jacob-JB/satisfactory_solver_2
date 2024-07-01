use eframe::egui::{RichText, ScrollArea, Vec2};

use crate::{
    factory::{save_factory, Factory, NetResources},
    world::{ResourceId, World},
};

use super::Page;

pub struct EditFactoryPage {
    world: World,

    factory: Factory,
    resources: NetResources,
    save_path: String,

    sub_factory: Factory,
    sub_resources: NetResources,
}

impl EditFactoryPage {
    pub fn new(world: World, factory: Factory) -> Self {
        let resources = factory.net_resources(&world);

        let sub_factory = Factory::default();
        let sub_resources = sub_factory.net_resources(&world);

        EditFactoryPage {
            world,

            factory,
            resources,
            save_path: String::new(),

            sub_factory,
            sub_resources,
        }
    }
}

impl Page for EditFactoryPage {
    fn show(mut self: Box<Self>, ui: &mut eframe::egui::Ui) -> Box<dyn Page> {
        ui.heading("Edit Factory");

        let save = ui.button("Save").clicked();
        ui.text_edit_singleline(&mut self.save_path);

        if save {
            save_factory(&self.world, &self.factory, &self.save_path);
        }

        let available_space = ui.available_rect_before_wrap();
        let collumn_width = available_space.width() / 2.;

        ui.push_id("Factory", |ui| {
            let mut collumn = available_space;
            collumn.set_width(collumn_width);

            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.heading("Factory");

                ScrollArea::new([false, true]).show(ui, |ui| {
                    ui.collapsing("Net Resources", |ui| {
                        for (resource_index, (rate, recipes)) in
                            self.resources.resources.iter().enumerate()
                        {
                            if recipes.len() == 0 {
                                continue;
                            }

                            let resource_name =
                                self.world.name_of_resource(ResourceId(resource_index));

                            ui.label(
                                RichText::new(format!(
                                    "{} net {:.0000001} /min",
                                    resource_name, rate
                                ))
                                .strong(),
                            );

                            for &(recipe, rate) in recipes.iter() {
                                let recipe_name = self.world.name_of_recipe(recipe);

                                ui.label(format!("  {} {:.0000001} /min", recipe_name, rate));
                            }
                        }
                    });

                    ui.collapsing("Recipes", |ui| {
                        for &(recipe, rate) in self.factory.recipes.iter() {
                            let recipe_name = self.world.name_of_recipe(recipe);

                            ui.horizontal(|ui| {
                                ui.label(RichText::new(recipe_name).strong());
                                ui.label(format!("{:.0000001} machines", rate));
                            });

                            for &(resource, resource_rate) in
                                self.world.recipes[recipe.0].rates.iter()
                            {
                                let resource_name = self.world.name_of_resource(resource);

                                ui.label(format!(
                                    "  {} {:.0000001} /min",
                                    resource_name,
                                    rate * resource_rate
                                ));
                            }
                        }
                    });
                });
            });
        });

        ui.push_id("Sub Factory", |ui| {
            let mut collumn = available_space.translate(Vec2::new(collumn_width, 0.));
            collumn.set_width(collumn_width);

            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.heading("Sub Factory");

                ui.label("In the future you'll be able to split out parts of your factory into a sub factory and save them separately.");
            });
        });

        self
    }
}
