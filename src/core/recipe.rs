use std::time::Duration;

use crate::persistance::recipe::{
    IngredientEntity, MutableIngredientEntity, MutableRecipeEntity, RecipeEntity, RecipeRepository,
    SearchRecipesArguments,
};
use thiserror::Error;

#[derive(Debug, Clone)]
pub(crate) struct RecipeService<RR: RecipeRepository> {
    repository: RR,
}

#[derive(Debug)]
pub(crate) struct Recipe {
    pub(crate) recipe_id: i32,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) ingredients: Vec<Ingredient>,
    pub(crate) cooking_time: Option<Duration>,
    pub(crate) meal_type: MealType,
}

impl From<RecipeEntity> for Recipe {
    fn from(value: RecipeEntity) -> Self {
        Self {
            recipe_id: value.recipe_id,
            name: value.name,
            description: value.description,
            ingredients: value
                .ingredients
                .into_iter()
                .map(Ingredient::from)
                .collect(),
            cooking_time: value.cooking_time,
            meal_type: value.meal_type.into(),
        }
    }
}

impl From<Recipe> for MutableRecipeEntity {
    fn from(value: Recipe) -> Self {
        Self {
            name: value.name,
            description: value.description,
            ingredients: value
                .ingredients
                .into_iter()
                .map(MutableIngredientEntity::from)
                .collect(),
            cooking_time: value.cooking_time,
            meal_type: value.meal_type.into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Ingredient {
    pub(crate) name: String,
    pub(crate) quantity_type: QuantityType,
    pub(crate) quantity: f32,
}

impl From<IngredientEntity> for Ingredient {
    fn from(value: IngredientEntity) -> Self {
        Self {
            name: value.name,
            quantity_type: value.quantity_type.into(),
            quantity: value.quantity,
        }
    }
}

impl From<Ingredient> for MutableIngredientEntity {
    fn from(value: Ingredient) -> Self {
        Self {
            name: value.name,
            quantity_type: value.quantity_type.into(),
            quantity: value.quantity,
        }
    }
}

#[derive(Debug)]
pub(crate) struct NewRecipe {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) ingredients: Vec<Ingredient>,
    pub(crate) cooking_time: Option<Duration>,
    pub(crate) meal_type: MealType,
}

#[derive(Debug)]
pub(crate) struct SearchCriteria {
    pub(crate) recipe_name: Option<String>,
    pub(crate) ingredient_name: Option<String>,
    pub(crate) meal_type: Option<MealType>,
}

impl From<NewRecipe> for MutableRecipeEntity {
    fn from(value: NewRecipe) -> Self {
        Self {
            name: value.name,
            description: value.description,
            ingredients: value
                .ingredients
                .into_iter()
                .map(MutableIngredientEntity::from)
                .collect(),
            cooking_time: value.cooking_time,
            meal_type: value.meal_type.into(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum QuantityType {
    Count,
    Kilo,
    Gram,
    Liter,
    Milliliter,
}

impl From<crate::persistance::recipe::QuantityType> for QuantityType {
    fn from(value: crate::persistance::recipe::QuantityType) -> Self {
        match value {
            crate::persistance::recipe::QuantityType::Count => Self::Count,
            crate::persistance::recipe::QuantityType::Kilo => Self::Kilo,
            crate::persistance::recipe::QuantityType::Gram => Self::Gram,
            crate::persistance::recipe::QuantityType::Liter => Self::Liter,
            crate::persistance::recipe::QuantityType::Milliliter => Self::Milliliter,
        }
    }
}

impl From<QuantityType> for crate::persistance::recipe::QuantityType {
    fn from(value: QuantityType) -> Self {
        match value {
            QuantityType::Count => Self::Count,
            QuantityType::Kilo => Self::Kilo,
            QuantityType::Gram => Self::Gram,
            QuantityType::Liter => Self::Liter,
            QuantityType::Milliliter => Self::Milliliter,
        }
    }
}

#[derive(Debug)]
pub(crate) enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

impl From<crate::persistance::recipe::MealType> for MealType {
    fn from(value: crate::persistance::recipe::MealType) -> Self {
        match value {
            crate::persistance::recipe::MealType::Breakfast => Self::Breakfast,
            crate::persistance::recipe::MealType::Lunch => Self::Lunch,
            crate::persistance::recipe::MealType::Dinner => Self::Dinner,
        }
    }
}

impl From<MealType> for crate::persistance::recipe::MealType {
    fn from(value: MealType) -> Self {
        match value {
            MealType::Breakfast => Self::Breakfast,
            MealType::Lunch => Self::Lunch,
            MealType::Dinner => Self::Dinner,
        }
    }
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
pub(crate) enum SearchRecipeError {
    #[error("An unknown error occured: {0:}")]
    Unknown(
        #[from]
        #[source]
        eyre::Report,
    ),
}

impl From<crate::persistance::recipe::SearchRecipeError> for SearchRecipeError {
    fn from(value: crate::persistance::recipe::SearchRecipeError) -> Self {
        match value {
            crate::persistance::recipe::SearchRecipeError::Unknown(report) => Self::Unknown(report),
        }
    }
}

impl From<crate::persistance::recipe::ListRecipeError> for ListRecipeError {
    fn from(value: crate::persistance::recipe::ListRecipeError) -> Self {
        match value {
            crate::persistance::recipe::ListRecipeError::Unknown(report) => Self::Unknown(report),
        }
    }
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

impl From<crate::persistance::recipe::CreateRecipeError> for CreateRecipeError {
    fn from(value: crate::persistance::recipe::CreateRecipeError) -> Self {
        match value {
            crate::persistance::recipe::CreateRecipeError::Unknown(report) => Self::Unknown(report),
        }
    }
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

impl From<crate::persistance::recipe::UpdateRecipeError> for UpdateRecipeError {
    fn from(value: crate::persistance::recipe::UpdateRecipeError) -> Self {
        match value {
            crate::persistance::recipe::UpdateRecipeError::Unknown(report) => Self::Unknown(report),
            crate::persistance::recipe::UpdateRecipeError::NotFound => Self::NotFound,
        }
    }
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

impl From<crate::persistance::recipe::DeleteRecipeError> for DeleteRecipeError {
    fn from(value: crate::persistance::recipe::DeleteRecipeError) -> Self {
        match value {
            crate::persistance::recipe::DeleteRecipeError::Unknown(report) => Self::Unknown(report),
            crate::persistance::recipe::DeleteRecipeError::NotFound => Self::NotFound,
        }
    }
}

impl<RR: RecipeRepository> RecipeService<RR> {
    pub(crate) fn new(repository: RR) -> Self {
        Self { repository }
    }

    pub(crate) async fn list_recipes(&self) -> Result<Vec<Recipe>, ListRecipeError> {
        let entity = self.repository.list_recipes().await?;
        Ok(entity.into_iter().map(Recipe::from).collect())
    }

    pub(crate) async fn create_recipe(&self, dto: NewRecipe) -> Result<Recipe, CreateRecipeError> {
        let entity = self.repository.create_recipe(dto.into()).await?;
        Ok(entity.into())
    }

    pub(crate) async fn update_recipe(&self, dto: Recipe) -> Result<Recipe, UpdateRecipeError> {
        let entity = self
            .repository
            .update_recipe(dto.recipe_id, dto.into())
            .await?;

        Ok(entity.into())
    }

    pub(crate) async fn delete_recipe(&self, recipe_id: i32) -> Result<(), DeleteRecipeError> {
        self.repository.delete_recipe(recipe_id).await?;
        Ok(())
    }

    pub(crate) async fn search_recipes(
        &self,
        criteria: SearchCriteria,
    ) -> Result<Vec<Recipe>, SearchRecipeError> {
        let args = SearchRecipesArguments {
            recipe_name: criteria.recipe_name,
            ingredient_name: criteria.ingredient_name,
            meal_type: criteria.meal_type.map(|mt| mt.into()),
        };

        let entities = self.repository.search_recipes(args).await?;
        Ok(entities.into_iter().map(Recipe::from).collect())
    }
}
