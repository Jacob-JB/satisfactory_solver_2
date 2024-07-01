use serde::{Deserialize, Serialize};

/// a resource id within a world
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ResourceId(pub usize);

/// a recipe id within a world
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecipeId(pub usize);

/// an id that is either a resource or a recipe
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VariableId {
    Resource(ResourceId),
    Recipe(RecipeId),
}

pub struct Resource {
    pub name: String,
}

pub struct Recipe {
    pub name: String,
    pub tags: Vec<String>,
    pub rates: Vec<(ResourceId, f64)>,
}

#[derive(Default)]
pub struct World {
    pub resources: Vec<Resource>,
    pub recipes: Vec<Recipe>,
}

impl ResourceId {
    pub fn variable_id(self) -> VariableId {
        VariableId::Resource(self)
    }
}

impl RecipeId {
    pub fn variable_id(self) -> VariableId {
        VariableId::Recipe(self)
    }
}

impl World {
    pub fn resource_id_of_name(&self, resource_name: &str) -> Option<ResourceId> {
        self.resources
            .iter()
            .position(|Resource { name }| *name == *resource_name)
            .map(|index| ResourceId(index))
    }

    pub fn name_of_resource(&self, resource: ResourceId) -> &str {
        &self
            .resources
            .get(resource.0)
            .expect("Invalid resource id was used")
            .name
    }

    pub fn recipe_id_of_name(&self, recipe_name: &str) -> Option<RecipeId> {
        self.recipes
            .iter()
            .position(|Recipe { name, .. }| *name == *recipe_name)
            .map(|index| RecipeId(index))
    }

    pub fn name_of_recipe(&self, recipe: RecipeId) -> &str {
        &self
            .recipes
            .get(recipe.0)
            .expect("Invalid recipe id was used")
            .name
    }

    pub fn name_of_variable(&self, variable: VariableId) -> String {
        match variable {
            VariableId::Resource(resource) => {
                format!("Resource {}", self.name_of_resource(resource))
            }
            VariableId::Recipe(recipe) => format!("Recipe {}", self.name_of_recipe(recipe)),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct WorldJson {
    resources: Vec<String>,
    recipes: Vec<RecipeJson>,
}

#[derive(Serialize, Deserialize)]
struct RecipeJson {
    name: String,
    tags: Vec<String>,
    per_minute: f64,
    rates: Vec<(String, f64)>,
}

#[derive(Debug)]
pub enum LoadWorldError {
    IoError(std::io::Error),
    JsonError(serde_json::Error),
    /// the name of a resource in a recipe did not appear in the list of resources
    BadRecipeResource {
        recipe_name: String,
        resource_name: String,
    },
}

pub fn load_world(path: impl AsRef<std::path::Path>) -> Result<World, LoadWorldError> {
    let file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(err) => return Err(LoadWorldError::IoError(err)),
    };

    let world_json: WorldJson = match serde_json::from_reader(std::io::BufReader::new(file)) {
        Ok(world) => world,
        Err(err) => return Err(LoadWorldError::JsonError(err)),
    };

    let mut world = World::default();

    // parse resources
    for resource_name in world_json.resources {
        world.resources.push(Resource {
            name: resource_name,
        });
    }

    // parse recipes
    for RecipeJson {
        name,
        tags,
        per_minute,
        rates,
    } in world_json.recipes
    {
        let mut recipe = Recipe {
            name: name.clone(),
            tags,
            rates: Vec::new(),
        };

        // convert from resource names to recipe ids
        for (resource_name, rate) in rates.iter() {
            let Some(resource_id) = world.resource_id_of_name(&resource_name) else {
                return Err(LoadWorldError::BadRecipeResource {
                    recipe_name: name,
                    resource_name: resource_name.clone(),
                });
            };

            let rate = rate * per_minute;

            recipe.rates.push((resource_id, rate));
        }

        world.recipes.push(recipe);
    }

    Ok(world)
}
