use eframe::egui::{Key, ScrollArea, Ui, Vec2};

use crate::{
    factory::load_factory,
    world::{load_world, LoadWorldError, World},
};

use super::{builder::BuildFactoryPage, factory::EditFactoryPage, Page};

pub struct LandingPage {
    input: String,
    valid_path: bool,
    feedback: std::borrow::Cow<'static, str>,
}

impl LandingPage {
    pub fn new() -> Self {
        LandingPage {
            input: String::new(),
            valid_path: false,
            feedback: "Input the path to the world json configuration.".into(),
        }
    }
}

impl Page for LandingPage {
    fn show(mut self: Box<Self>, ui: &mut Ui) -> Box<dyn Page> {
        let mut next_page: Option<Box<dyn Page>> = None;

        ui.heading("Load World");

        ui.horizontal(|ui| {
            let text_box = ui.text_edit_singleline(&mut self.input);

            if text_box.changed() {
                let path = std::path::Path::new(&self.input);

                if path.exists() {
                    self.feedback = "Valid path.".into();
                    self.valid_path = true;
                } else {
                    self.feedback = "Invalid path.".into();
                    self.valid_path = false;
                };
            }

            ui.set_enabled(self.valid_path);

            if ui.button("Load").clicked()
                || (self.valid_path
                    && text_box.lost_focus()
                    && text_box.ctx.input(|input| input.key_pressed(Key::Enter)))
            {
                let path = std::path::Path::new(&self.input);
                match load_world(path) {
                    Ok(world) => {
                        next_page = Some(Box::new(LoadedPage::new(world)));
                    }
                    Err(LoadWorldError::IoError(_)) => {
                        self.feedback = "Io Error".into();
                    }
                    Err(LoadWorldError::JsonError(_)) => {
                        self.feedback = "Invalid Json".into();
                    }
                    Err(LoadWorldError::BadRecipeResource {
                        recipe_name,
                        resource_name,
                    }) => {
                        self.feedback = format!(
                            "Bad resource name \"{}\" in recipe \"{}\"",
                            resource_name, recipe_name
                        )
                        .into();
                    }
                }
            }
        });

        ui.label(self.feedback.as_ref());

        next_page.unwrap_or(self)
    }
}

struct LoadedPage {
    world: World,
    tags: Vec<String>,
    included: Vec<bool>,
    open_field: String,
}

impl LoadedPage {
    fn new(world: World) -> Self {
        let mut tags = Vec::new();

        for recipe in world.recipes.iter() {
            for tag in recipe.tags.iter() {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }
        }

        let included = vec![true; world.recipes.len()];

        LoadedPage {
            world,
            tags,
            included,
            open_field: String::new(),
        }
    }

    fn filter_world(self) -> World {
        World {
            recipes: self
                .world
                .recipes
                .into_iter()
                .enumerate()
                .filter_map(|(index, recipe)| {
                    if self.included[index] {
                        Some(recipe)
                    } else {
                        None
                    }
                })
                .collect(),
            ..self.world
        }
    }
}

impl Page for LoadedPage {
    fn show(mut self: Box<Self>, ui: &mut Ui) -> Box<dyn Page> {
        ui.heading("Select Recipes");

        let (back, confirm, open) = ui
            .horizontal(|ui| {
                (
                    ui.button("Back").clicked(),
                    ui.button("Build").clicked(),
                    ui.button("Open Factory").clicked(),
                )
            })
            .inner;

        ui.text_edit_singleline(&mut self.open_field);

        let available_space = ui.available_rect_before_wrap();

        let collumn_width = available_space.width() / 3.;

        ui.push_id("Resources", |ui| {
            let mut collumn = available_space;
            collumn.set_width(collumn_width);

            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Resources");

                    ScrollArea::new([false, true]).show(ui, |ui| {
                        for resource in self.world.resources.iter() {
                            ui.label(&resource.name);
                        }
                    });
                });
            });
        });

        ui.push_id("Tags", |ui| {
            let mut collumn = available_space.translate(Vec2::new(collumn_width, 0.));
            collumn.set_width(collumn_width);

            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Tags");

                    ScrollArea::new([false, true]).show(ui, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("Add all").clicked() {
                                for included in self.included.iter_mut() {
                                    *included = true;
                                }
                            }

                            if ui.button("Remove all").clicked() {
                                for included in self.included.iter_mut() {
                                    *included = false;
                                }
                            }
                        });

                        for tag in self.tags.iter() {
                            ui.horizontal(|ui| {
                                if ui.button("Add").clicked() {
                                    for (index, recipe) in self.world.recipes.iter().enumerate() {
                                        if recipe.tags.contains(tag) {
                                            self.included[index] = true;
                                        }
                                    }
                                }

                                if ui.button("Remove").clicked() {
                                    for (index, recipe) in self.world.recipes.iter().enumerate() {
                                        if recipe.tags.contains(tag) {
                                            self.included[index] = false;
                                        }
                                    }
                                }

                                ui.label(tag);
                            });
                        }
                    });
                });
            });
        });

        ui.push_id("Recipes", |ui| {
            let mut collumn = available_space.translate(Vec2::new(collumn_width * 2., 0.));
            collumn.set_width(collumn_width);

            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.vertical(|ui| {
                    ui.heading("Recipes");

                    ScrollArea::new([false, true]).show(ui, |ui| {
                        for (index, recipe) in self.world.recipes.iter().enumerate() {
                            ui.checkbox(&mut self.included[index], &recipe.name);
                        }
                    });
                });
            });
        });

        if back {
            return Box::new(LandingPage::new());
        }

        if confirm {
            return Box::new(BuildFactoryPage::new(self.filter_world()));
        }

        if open {
            if let Ok(factory) = load_factory(&self.world, &self.open_field) {
                return Box::new(EditFactoryPage::new(self.world, factory));
            }
        }

        self
    }
}
