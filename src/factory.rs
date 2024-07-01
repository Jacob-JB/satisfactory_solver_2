use std::io::Write;

use log::warn;
use serde::{Deserialize, Serialize};

use crate::world::{RecipeId, ResourceId, World};

#[derive(Default, Clone)]
pub struct Factory {
    pub recipes: Vec<(RecipeId, f64)>,
}

/// computed net resources from a [Factory]
pub struct NetResources {
    pub resources: Vec<(f64, Vec<(RecipeId, f64)>)>,
}

impl Factory {
    pub fn net_resources(&self, world: &World) -> NetResources {
        let mut resources = vec![(0., Vec::new()); world.resources.len()];

        for &(RecipeId(recipe_index), recipe_rate) in self.recipes.iter() {
            for &(ResourceId(resource_index), resource_rate) in
                world.recipes[recipe_index].rates.iter()
            {
                let rate = recipe_rate * resource_rate;

                let entry = &mut resources[resource_index];
                entry.0 += rate;
                entry.1.push((RecipeId(recipe_index), rate));
            }
        }

        NetResources { resources }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct FactoryJson {
    recipes: Vec<(String, f64)>,
}

pub fn save_factory(world: &World, factory: &Factory, path: impl AsRef<std::path::Path>) {
    let mut factory_json = FactoryJson::default();

    for &(recipe, rate) in factory.recipes.iter() {
        let recipe_name = world.name_of_recipe(recipe).into();

        factory_json.recipes.push((recipe_name, rate));
    }

    let mut file = match std::fs::File::create(path) {
        Ok(file) => file,
        Err(err) => {
            warn!("failed to open factory file: {:?}", err);
            return;
        }
    };

    if let Err(err) = file.write_all(
        serde_json::to_string(&factory_json)
            .expect("Failed to convert to json")
            .as_bytes(),
    ) {
        warn!("failed to write to factory file: {:?}", err);
    }
}

#[derive(Debug)]
pub enum LoadFactoryError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    BadRecipeName { recipe_name: String },
}

pub fn load_factory(
    world: &World,
    path: impl AsRef<std::path::Path>,
) -> Result<Factory, LoadFactoryError> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(LoadFactoryError::IoError(err)),
    };

    let factory_json: FactoryJson = match serde_json::from_reader(std::io::BufReader::new(file)) {
        Ok(factory) => factory,
        Err(err) => return Err(LoadFactoryError::JsonError(err)),
    };

    let mut factory = Factory::default();

    for (recipe_name, rate) in factory_json.recipes {
        let Some(recipe) = world.recipe_id_of_name(&recipe_name) else {
            return Err(LoadFactoryError::BadRecipeName { recipe_name });
        };

        factory.recipes.push((recipe, rate));
    }

    Ok(factory)
}
