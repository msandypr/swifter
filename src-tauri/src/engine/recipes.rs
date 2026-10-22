use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const DEFAULT_RECIPES_JSON: &str = include_str!("../../../recipes/default_recipes.json");

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BackendConfig {
    pub name: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EngineRecipe {
    pub name: String,
    pub recommended_backend: String,
    #[serde(default)]
    pub env: HashMap<String, String>,
    #[serde(default)]
    pub enable_msync: bool,
    #[serde(default)]
    pub enable_esync: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecipeDatabase {
    pub engines: HashMap<String, EngineRecipe>,
    pub backends: HashMap<String, BackendConfig>,
}

impl RecipeDatabase {
    pub fn load_default() -> Self {
        serde_json::from_str(DEFAULT_RECIPES_JSON).unwrap_or_else(|_| Self {
            engines: HashMap::new(),
            backends: HashMap::new(),
        })
    }

    pub fn get_recipe_for_engine(&self, engine: &str) -> Option<&EngineRecipe> {
        self.engines.get(engine)
    }

    pub fn get_backend_config(&self, backend: &str) -> Option<&BackendConfig> {
        self.backends.get(backend)
    }
}
