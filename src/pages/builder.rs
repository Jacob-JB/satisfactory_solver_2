use eframe::egui::{ComboBox, RichText, ScrollArea, Ui, Vec2};
use log::debug;

use crate::{
    builder::{load_rule_list, save_rule_list, Constraint, Problem, Rule, RuleList},
    factory::{Factory, NetResources},
    world::{RecipeId, ResourceId, VariableId, World},
};

use super::{factory::EditFactoryPage, Page};

pub struct BuildFactoryPage {
    world: World,
    path_field: String,
    /// each element is a rule list, a uniqe ui id and a rule builder if building a rule
    rule_lists: Vec<(u64, RuleList, Option<RuleBuilder>)>,
    rule_list_id_incrementor: u64,

    optimizations: Vec<(u64, Option<VariableId>, String)>,
    optimization_id_incrementor: u64,

    solution: Result<(Factory, NetResources), String>,
}

impl BuildFactoryPage {
    pub fn new(world: World) -> Self {
        BuildFactoryPage {
            world,
            path_field: String::new(),
            rule_lists: Vec::new(),
            rule_list_id_incrementor: 0,

            optimizations: Vec::new(),
            optimization_id_incrementor: 0,

            solution: Err("".into()),
        }
    }
}

impl Page for BuildFactoryPage {
    fn show(mut self: Box<Self>, ui: &mut Ui) -> Box<dyn Page> {
        ui.heading("Factory Builder");

        let mut edit_factory = None;

        let available_space = ui.available_rect_before_wrap();
        let collumn_width = available_space.width() / 3.;

        ui.push_id("Rules", |ui| {
            let mut collumn = available_space;
            collumn.set_width(collumn_width);
            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.heading("Rules");

                let (new_list, load_list) = ui
                    .horizontal(|ui| {
                        (
                            ui.button("New List").clicked(),
                            ui.button("Load List").clicked(),
                        )
                    })
                    .inner;

                ui.text_edit_singleline(&mut self.path_field);

                if new_list {
                    self.rule_lists.push((
                        self.rule_list_id_incrementor,
                        RuleList::default(),
                        None,
                    ));
                    self.rule_list_id_incrementor += 1;
                }

                if load_list {
                    if let Ok(rule_list) = load_rule_list(&self.world, &self.path_field) {
                        self.rule_lists
                            .push((self.rule_list_id_incrementor, rule_list, None));
                        self.rule_list_id_incrementor += 1;
                    }
                }

                ScrollArea::new([false, true]).show(ui, |ui| {
                    let mut up = None;
                    let mut down = None;
                    let mut delete = None;

                    for (index, (ui_id, rule_list, rule_builder)) in
                        self.rule_lists.iter_mut().enumerate()
                    {
                        ui.push_id(ui_id, |ui| {
                            ui.collapsing("Rule List", |ui| {
                                let (
                                    up_clicked,
                                    down_clicked,
                                    delete_clicked,
                                    save_clicked,
                                    add_rule,
                                ) = ui
                                    .horizontal(|ui| {
                                        (
                                            ui.button("Up").clicked(),
                                            ui.button("Down").clicked(),
                                            ui.button("Delete").clicked(),
                                            ui.button("Save").clicked(),
                                            {
                                                if rule_builder.is_none() {
                                                    ui.button("Add Rule").clicked()
                                                } else {
                                                    false
                                                }
                                            },
                                        )
                                    })
                                    .inner;

                                if up_clicked {
                                    up = Some(index);
                                }

                                if down_clicked {
                                    down = Some(index);
                                }

                                if delete_clicked {
                                    delete = Some(index);
                                }

                                if save_clicked {
                                    save_rule_list(&self.world, &rule_list, &self.path_field);
                                }

                                if add_rule {
                                    *rule_builder = Some(RuleBuilder::new());
                                }

                                if if let Some(rule_builder) = rule_builder.as_mut() {
                                    rule_builder.show(&self.world, ui);

                                    let (cancel, add) = ui
                                        .horizontal(|ui| {
                                            (
                                                ui.button("Cancel").clicked(),
                                                ui.button("Add").clicked(),
                                            )
                                        })
                                        .inner;

                                    'b: {
                                        if cancel {
                                            break 'b true;
                                        }

                                        if add {
                                            if let Some(rule) = rule_builder.build() {
                                                rule_list.rules.push(rule);
                                                break 'b true;
                                            }
                                        }

                                        false
                                    }
                                } else {
                                    false
                                } {
                                    *rule_builder = None;
                                }

                                rule_list.rules.retain(|rule| {
                                    ui.horizontal(|ui| {
                                        let remove = ui.button("Edit").clicked();

                                        let mut rule_builder_rate = None;

                                        ui.label(format!(
                                            "{} {}",
                                            self.world.name_of_variable(rule.variable),
                                            match rule.constraint {
                                                Constraint::Less(rate) => {
                                                    rule_builder_rate = Some(rate);
                                                    format!("less than {}", rate)
                                                }
                                                Constraint::Equal(rate) => {
                                                    rule_builder_rate = Some(rate);
                                                    format!("equal to {}", rate)
                                                }
                                                Constraint::Greater(rate) => {
                                                    rule_builder_rate = Some(rate);
                                                    format!("greater than {}", rate)
                                                }
                                                Constraint::Unconstrained =>
                                                    format!("unconstrained"),
                                            }
                                        ));

                                        if remove {
                                            *rule_builder = Some(RuleBuilder {
                                                selected_variable: Some(rule.variable),
                                                constraint: rule.constraint,
                                                rate: format!(
                                                    "{}",
                                                    rule_builder_rate.unwrap_or(0.)
                                                ),
                                            })
                                        }

                                        !remove
                                    })
                                    .inner
                                });
                            });
                        });
                    }

                    if let Some(up) = up {
                        if up == 0 {
                            return;
                        }

                        self.rule_lists.swap(up, up - 1);
                    }

                    if let Some(down) = down {
                        if down + 1 >= self.rule_lists.len() {
                            return;
                        }

                        self.rule_lists.swap(down, down + 1);
                    }

                    if let Some(delete) = delete {
                        self.rule_lists.remove(delete);
                    }
                });
            });
        });

        ui.push_id("Optimization", |ui| {
            let mut collumn = available_space.translate(Vec2::new(collumn_width, 0.));
            collumn.set_width(collumn_width);
            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.heading("Optimization");

                if ui.button("New").clicked() {
                    self.optimizations
                        .push((self.optimization_id_incrementor, None, "0".into()));
                    self.optimization_id_incrementor += 1;
                }

                ScrollArea::new([false, true]).show(ui, |ui| {
                    let mut remove = None;

                    for (index, (ui_id, selected_variable, bias)) in
                        self.optimizations.iter_mut().enumerate()
                    {
                        ui.push_id(ui_id, |ui| {
                            ui.horizontal(|ui| {
                                let selected_text = match selected_variable {
                                    Some(variable) => self.world.name_of_variable(*variable),
                                    None => "...".into(),
                                };

                                ComboBox::from_label("")
                                    .selected_text(selected_text)
                                    .show_ui(ui, |ui| {
                                        if ui.selectable_label(false, "Delete").clicked() {
                                            remove = Some(index);
                                        };

                                        ui.label(RichText::new("Resources").strong());
                                        for (resource_id, resource) in
                                            self.world.resources.iter().enumerate()
                                        {
                                            ui.selectable_value(
                                                selected_variable,
                                                Some(VariableId::Resource(ResourceId(resource_id))),
                                                &resource.name,
                                            );
                                        }

                                        ui.label(RichText::new("Recipes").strong());
                                        for (recipe_id, recipe) in
                                            self.world.recipes.iter().enumerate()
                                        {
                                            ui.selectable_value(
                                                selected_variable,
                                                Some(VariableId::Recipe(RecipeId(recipe_id))),
                                                &recipe.name,
                                            );
                                        }
                                    });

                                ui.text_edit_singleline(bias);

                                if bias.parse::<f64>().is_err() {
                                    ui.label("Invalid number");
                                }
                            });
                        });
                    }

                    if let Some(index) = remove {
                        self.optimizations.remove(index);
                    }
                });
            });
        });

        ui.push_id("Output", |ui| {
            let mut collumn = available_space.translate(Vec2::new(collumn_width * 2., 0.));
            collumn.set_width(collumn_width);
            ui.allocate_ui_at_rect(collumn, |ui| {
                ui.heading("Output");

                let solve = ui.button("Solve").clicked();

                'cancel: {
                    if solve {
                        let mut problem = Problem::default();

                        for (_, rule_list, _) in self.rule_lists.iter() {
                            for rule in rule_list.rules.iter() {
                                problem.rules.push(*rule);
                            }
                        }

                        for (_, variable, rate) in self.optimizations.iter() {
                            let Some(variable) = variable else {
                                continue;
                            };

                            let Ok(rate) = rate.parse() else {
                                self.solution =
                                    Err(format!("Invalid number \"{}\" in optimization", rate));
                                break 'cancel;
                            };

                            problem.optimizations.push((*variable, rate));
                        }

                        self.solution = match problem.solve(&self.world) {
                            Err(response) => Err(response),
                            Ok(factory) => {
                                let resources = factory.net_resources(&self.world);
                                Ok((factory, resources))
                            }
                        };
                    }
                }

                ScrollArea::new([false, true]).show(ui, |ui| match &self.solution {
                    Ok((factory, net_resources)) => {
                        if ui.button("Edit").clicked() {
                            edit_factory = Some(factory.clone());
                        }

                        ui.collapsing("Net Resources", |ui| {
                            for (resource_index, (rate, recipes)) in
                                net_resources.resources.iter().enumerate()
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
                            for &(recipe, rate) in factory.recipes.iter() {
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
                    }
                    Err(response) => {
                        ui.label(response);
                    }
                });
            });
        });

        if let Some(factory) = edit_factory {
            return Box::new(EditFactoryPage::new(self.world, factory));
        }

        self
    }
}

struct RuleBuilder {
    selected_variable: Option<VariableId>,
    constraint: Constraint,
    rate: String,
}

impl RuleBuilder {
    fn new() -> Self {
        RuleBuilder {
            selected_variable: None,
            constraint: Constraint::Equal(0.),
            rate: "0".into(),
        }
    }

    fn show(&mut self, world: &World, ui: &mut Ui) {
        ui.label("New Rule:");

        let selected_text = match self.selected_variable {
            Some(variable) => world.name_of_variable(variable),
            None => "...".into(),
        };

        ui.push_id("Variable", |ui| {
            ComboBox::from_label("")
                .selected_text(selected_text)
                .show_ui(ui, |ui| {
                    ui.label(RichText::new("Resources").strong());
                    for (resource_id, resource) in world.resources.iter().enumerate() {
                        ui.selectable_value(
                            &mut self.selected_variable,
                            Some(VariableId::Resource(ResourceId(resource_id))),
                            &resource.name,
                        );
                    }

                    ui.label(RichText::new("Recipes").strong());
                    for (recipe_id, recipe) in world.recipes.iter().enumerate() {
                        ui.selectable_value(
                            &mut self.selected_variable,
                            Some(VariableId::Recipe(RecipeId(recipe_id))),
                            &recipe.name,
                        );
                    }
                });
        });

        ui.push_id("Constraint", |ui| {
            ComboBox::from_label("")
                .selected_text(match self.constraint {
                    Constraint::Less(_) => "Less",
                    Constraint::Equal(_) => "Equal",
                    Constraint::Greater(_) => "Greater",
                    Constraint::Unconstrained => "Unconstrained",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.constraint, Constraint::Less(0.), "Less");
                    ui.selectable_value(&mut self.constraint, Constraint::Equal(0.), "Equal");
                    ui.selectable_value(&mut self.constraint, Constraint::Greater(0.), "Greater");
                    ui.selectable_value(
                        &mut self.constraint,
                        Constraint::Unconstrained,
                        "Unconstrained",
                    );
                });
        });

        if matches!(
            self.constraint,
            Constraint::Less(_) | Constraint::Equal(_) | Constraint::Greater(_)
        ) {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.rate);

                if self.rate.parse::<f64>().is_err() {
                    ui.label("Invalid number");
                }
            });
        }
    }

    fn build(&self) -> Option<Rule> {
        let parse_rate = || self.rate.parse().ok();

        let constraint = match self.constraint {
            Constraint::Less(_) => Constraint::Less(parse_rate()?),
            Constraint::Equal(_) => Constraint::Equal(parse_rate()?),
            Constraint::Greater(_) => Constraint::Greater(parse_rate()?),
            Constraint::Unconstrained => Constraint::Unconstrained,
        };

        let variable = self.selected_variable?;

        Some(Rule {
            variable,
            constraint,
        })
    }
}
