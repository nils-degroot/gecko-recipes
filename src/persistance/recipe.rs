use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use thiserror::Error;

#[derive(Debug)]
pub(crate) struct RecipeEntity {
    pub(crate) recipe_id: i32,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) ingredients: Vec<IngredientEntity>,
    pub(crate) cooking_time: Option<Duration>,
    pub(crate) meal_type: MealType,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(crate) struct IngredientEntity {
    pub(crate) ingredient_id: i32,
    pub(crate) recipe_id: i32,
    pub(crate) ingredient_order: i32,
    pub(crate) name: String,
    pub(crate) quantity_type: QuantityType,
    pub(crate) quantity: f32,
}

#[derive(Debug)]
pub(crate) struct MutableRecipeEntity {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) ingredients: Vec<MutableIngredientEntity>,
    pub(crate) cooking_time: Option<Duration>,
    pub(crate) meal_type: MealType,
}

#[derive(Debug, FromRow, Serialize, Deserialize)]
pub(crate) struct MutableIngredientEntity {
    pub(crate) name: String,
    pub(crate) quantity_type: QuantityType,
    pub(crate) quantity: f32,
}

#[derive(Debug, Type, Serialize, Deserialize)]
#[sqlx(type_name = "quantity_type")]
pub(crate) enum QuantityType {
    Count,
    Kilo,
    Gram,
    Liter,
    Milliliter,
}

#[derive(Debug, Type, Serialize, Deserialize)]
#[sqlx(type_name = "meal_type")]
pub(crate) enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

#[derive(Debug, Error)]
pub(crate) enum ListRecipeError {
    #[error("An unknown error occured: {0:}")]
    Unknown(
        #[from]
        #[source]
        eyre::Report,
    ),
}

#[derive(Debug, Error)]
pub(crate) enum CreateRecipeError {
    #[error("An unknown error occured: {0:}")]
    Unknown(
        #[from]
        #[source]
        eyre::Report,
    ),
}

#[derive(Debug, Error)]
pub(crate) enum UpdateRecipeError {
    #[error("An unknown error occured: {0:}")]
    Unknown(
        #[from]
        #[source]
        eyre::Report,
    ),
    #[error("The recipe could not be found")]
    NotFound,
}

#[derive(Debug, Error)]
pub(crate) enum DeleteRecipeError {
    #[error("An unknown error occured: {0:}")]
    Unknown(
        #[from]
        #[source]
        eyre::Report,
    ),
    #[error("The recipe could not be found")]
    NotFound,
}

pub(crate) trait RecipeRepository: std::fmt::Debug + Clone + Send + Sync + 'static {
    async fn list_recipes(&self) -> Result<Vec<RecipeEntity>, ListRecipeError>;

    async fn create_recipe(
        &self,
        entity: MutableRecipeEntity,
    ) -> Result<RecipeEntity, CreateRecipeError>;

    async fn update_recipe(
        &self,
        recipe_id: i32,
        entity: MutableRecipeEntity,
    ) -> Result<RecipeEntity, UpdateRecipeError>;

    async fn delete_recipe(&self, recipe_id: i32) -> Result<(), DeleteRecipeError>;
}
