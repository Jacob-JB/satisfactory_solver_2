use std::io::Write;

use log::warn;
use serde::{Deserialize, Serialize};

use crate::{
    factory::Factory,
    world::{RecipeId, ResourceId, VariableId, World},
};

#[derive(Clone, Copy)]
pub struct Rule {
    pub variable: VariableId,
    pub constraint: Constraint,
}

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    Less(f64),
    Equal(f64),
    Greater(f64),
    Unconstrained,
}

#[derive(Default)]
pub struct RuleList {
    pub rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize)]
pub enum RuleJson {
    Resource {
        resource: String,
        constraint: Constraint,
    },
    Recipe {
        recipe: String,
        constraint: Constraint,
    },
}

#[derive(Default, Serialize, Deserialize)]
pub struct RuleListJson {
    pub rules: Vec<RuleJson>,
}

#[derive(Debug)]
pub enum LoadRuleListError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    BadRecipeName { recipe_name: String },
    BadResourceName { resource_name: String },
}

pub fn load_rule_list(
    world: &World,
    path: impl AsRef<std::path::Path>,
) -> Result<RuleList, LoadRuleListError> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(LoadRuleListError::IoError(err)),
    };

    let rule_list_json: RuleListJson = match serde_json::from_reader(std::io::BufReader::new(file))
    {
        Ok(rule_list) => rule_list,
        Err(err) => return Err(LoadRuleListError::JsonError(err)),
    };

    let mut rule_list = RuleList::default();

    for rule in rule_list_json.rules {
        match rule {
            RuleJson::Resource {
                resource,
                constraint,
            } => {
                let Some(resource) = world.resource_id_of_name(&resource) else {
                    return Err(LoadRuleListError::BadResourceName {
                        resource_name: resource,
                    });
                };

                rule_list.rules.push(Rule {
                    variable: resource.variable_id(),
                    constraint,
                });
            }
            RuleJson::Recipe { recipe, constraint } => {
                let Some(recipe) = world.recipe_id_of_name(&recipe) else {
                    return Err(LoadRuleListError::BadRecipeName {
                        recipe_name: recipe,
                    });
                };

                rule_list.rules.push(Rule {
                    variable: recipe.variable_id(),
                    constraint,
                });
            }
        }
    }

    Ok(rule_list)
}

pub fn save_rule_list(world: &World, rule_list: &RuleList, path: impl AsRef<std::path::Path>) {
    let mut rule_list_json = RuleListJson::default();

    for rule in rule_list.rules.iter() {
        rule_list_json.rules.push(match rule.variable {
            VariableId::Resource(resource) => RuleJson::Resource {
                resource: world.name_of_resource(resource).into(),
                constraint: rule.constraint,
            },
            VariableId::Recipe(recipe) => RuleJson::Recipe {
                recipe: world.name_of_recipe(recipe).into(),
                constraint: rule.constraint,
            },
        });
    }

    let mut file = match std::fs::File::create(path) {
        Ok(file) => file,
        Err(err) => {
            warn!("failed to open rule list file: {:?}", err);
            return;
        }
    };

    if let Err(err) = file.write_all(
        serde_json::to_string(&rule_list_json)
            .expect("Failed to convert to json")
            .as_bytes(),
    ) {
        warn!("failed to write to rule list file: {:?}", err);
    }
}

#[derive(Default)]
pub struct Problem {
    pub rules: Vec<Rule>,
    pub optimizations: Vec<(VariableId, f64)>,
}

pub const SOLUTION_ROUND_PRECISION: f64 = 1_000_000.;

impl Problem {
    pub fn solve(&self, world: &World) -> Result<Factory, String> {
        let mut problem = minilp::Problem::new(minilp::OptimizationDirection::Maximize);

        // add all the problem variables

        let mut resource_coefficients = vec![0.; world.resources.len()];
        let mut recipe_coefficients = vec![0.; world.recipes.len()];

        for &(variable, coefficient) in self.optimizations.iter() {
            match variable {
                VariableId::Resource(ResourceId(index)) => {
                    resource_coefficients[index] = coefficient
                }
                VariableId::Recipe(RecipeId(index)) => recipe_coefficients[index] = coefficient,
            }
        }

        let resource_variables: Vec<_> = resource_coefficients
            .into_iter()
            .map(|coefficient| problem.add_var(coefficient, (f64::NEG_INFINITY, f64::INFINITY)))
            .collect();

        let recipe_variables: Vec<_> = recipe_coefficients
            .into_iter()
            .map(|coefficient| problem.add_var(coefficient, (f64::NEG_INFINITY, f64::INFINITY)))
            .collect();

        // add all the recipe constraints
        //
        // the net usage of a resource should be the sum
        // of all the recipes that produce or consume it
        // times the rate for each recipe

        let mut resource_recipe_coefficients = vec![Vec::new(); world.resources.len()];

        for (recipe_index, recipe) in world.recipes.iter().enumerate() {
            let recipe_variable = recipe_variables[recipe_index];

            for &(ResourceId(resource_index), rate) in recipe.rates.iter() {
                resource_recipe_coefficients[resource_index].push((recipe_variable, rate));
            }
        }

        for (resource_index, mut recipe_coefficients) in
            resource_recipe_coefficients.into_iter().enumerate()
        {
            recipe_coefficients.push((resource_variables[resource_index], -1.));

            problem.add_constraint(recipe_coefficients, minilp::ComparisonOp::Eq, 0.)
        }

        // limit all recipes to be positive

        for &recipe_variable in recipe_variables.iter() {
            problem.add_constraint(&[(recipe_variable, 1.)], minilp::ComparisonOp::Ge, 0.);
        }

        // add user constraints

        // whether to constrain a resource net value to 0 by default
        let mut resource_default = vec![true; world.resources.len()];

        for rule in self.rules.iter() {
            // if there is any rule specified for a resource, don't apply the default rule
            if let VariableId::Resource(ResourceId(index)) = rule.variable {
                resource_default[index] = false;
            }

            let (operator, rhs) = match rule.constraint {
                Constraint::Less(rate) => (minilp::ComparisonOp::Le, rate),
                Constraint::Equal(rate) => (minilp::ComparisonOp::Eq, rate),
                Constraint::Greater(rate) => (minilp::ComparisonOp::Ge, rate),
                Constraint::Unconstrained => continue,
            };

            let variable = match rule.variable {
                VariableId::Resource(ResourceId(index)) => resource_variables[index],
                VariableId::Recipe(RecipeId(index)) => recipe_variables[index],
            };

            problem.add_constraint(&[(variable, 1.)], operator, rhs);
        }

        // add default resource constraints

        for (index, constrain) in resource_default.into_iter().enumerate() {
            if constrain {
                let resource_variable = resource_variables[index];
                problem.add_constraint(&[(resource_variable, 1.)], minilp::ComparisonOp::Eq, 0.);
            }
        }

        // solve

        let solution = match problem.solve() {
            Ok(solution) => solution,
            Err(error) => {
                return Err(match error {
                    minilp::Error::Infeasible => "Infeasible".into(),
                    minilp::Error::Unbounded => "Unbounded".into(),
                });
            }
        };

        let mut factory = Factory::default();

        for (index, recipe_variable) in recipe_variables.into_iter().enumerate() {
            let rate = *solution.var_value(recipe_variable);

            let rate = (rate * SOLUTION_ROUND_PRECISION).round() / SOLUTION_ROUND_PRECISION;

            if rate.abs() < f64::EPSILON {
                continue;
            }

            factory.recipes.push((RecipeId(index), rate));
        }

        Ok(factory)
    }
}
