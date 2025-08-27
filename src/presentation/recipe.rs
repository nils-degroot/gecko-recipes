use std::time::Duration;

use actix_web::{
    HttpResponse, ResponseError, delete, get,
    http::header::ContentType,
    post, put,
    web::{Data, Json, Path},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    RecipeService,
    core::recipe::{Ingredient, NewRecipe, Recipe},
};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct RecipeDto {
    pub(crate) recipe_id: i32,
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) ingredients: Vec<IngredientDto>,
    pub(crate) cooking_time: Option<Duration>,
    pub(crate) meal_type: MealType,
}

impl From<Recipe> for RecipeDto {
    fn from(value: Recipe) -> Self {
        Self {
            recipe_id: value.recipe_id,
            name: value.name,
            description: value.description,
            ingredients: value
                .ingredients
                .into_iter()
                .map(IngredientDto::from)
                .collect(),
            cooking_time: value.cooking_time,
            meal_type: value.meal_type.into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct IngredientDto {
    pub(crate) name: String,
    pub(crate) quantity_type: QuantityType,
    pub(crate) quantity: f32,
}

impl From<Ingredient> for IngredientDto {
    fn from(value: Ingredient) -> Self {
        Self {
            name: value.name,
            quantity_type: value.quantity_type.into(),
            quantity: value.quantity,
        }
    }
}

impl From<IngredientDto> for Ingredient {
    fn from(value: IngredientDto) -> Self {
        Self {
            name: value.name,
            quantity_type: value.quantity_type.into(),
            quantity: value.quantity,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct NewRecipeDto {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
    pub(crate) ingredients: Vec<IngredientDto>,
    pub(crate) cooking_time: Option<Duration>,
    pub(crate) meal_type: MealType,
}

impl From<NewRecipeDto> for NewRecipe {
    fn from(value: NewRecipeDto) -> Self {
        Self {
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum QuantityType {
    Count,
    Kilo,
    Gram,
    Liter,
    Milliliter,
}

impl From<crate::core::recipe::QuantityType> for QuantityType {
    fn from(value: crate::core::recipe::QuantityType) -> Self {
        match value {
            crate::core::recipe::QuantityType::Count => Self::Count,
            crate::core::recipe::QuantityType::Kilo => Self::Kilo,
            crate::core::recipe::QuantityType::Gram => Self::Gram,
            crate::core::recipe::QuantityType::Liter => Self::Liter,
            crate::core::recipe::QuantityType::Milliliter => Self::Milliliter,
        }
    }
}

impl From<QuantityType> for crate::core::recipe::QuantityType {
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum MealType {
    Breakfast,
    Lunch,
    Dinner,
}

impl From<crate::core::recipe::MealType> for MealType {
    fn from(value: crate::core::recipe::MealType) -> Self {
        match value {
            crate::core::recipe::MealType::Breakfast => Self::Breakfast,
            crate::core::recipe::MealType::Lunch => Self::Lunch,
            crate::core::recipe::MealType::Dinner => Self::Dinner,
        }
    }
}

impl From<MealType> for crate::core::recipe::MealType {
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

impl From<crate::core::recipe::ListRecipeError> for ListRecipeError {
    fn from(value: crate::core::recipe::ListRecipeError) -> Self {
        match value {
            crate::core::recipe::ListRecipeError::Unknown(report) => Self::Unknown(report),
        }
    }
}

impl ResponseError for ListRecipeError {}

#[derive(Debug, Error)]
pub(crate) enum CreateRecipeError {
    #[error("An unknown error occured: {0:}")]
    Unknown(
        #[from]
        #[source]
        eyre::Report,
    ),
}

impl From<crate::core::recipe::CreateRecipeError> for CreateRecipeError {
    fn from(value: crate::core::recipe::CreateRecipeError) -> Self {
        match value {
            crate::core::recipe::CreateRecipeError::Unknown(report) => Self::Unknown(report),
        }
    }
}

impl ResponseError for CreateRecipeError {}

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

impl From<crate::core::recipe::UpdateRecipeError> for UpdateRecipeError {
    fn from(value: crate::core::recipe::UpdateRecipeError) -> Self {
        match value {
            crate::core::recipe::UpdateRecipeError::Unknown(report) => Self::Unknown(report),
            crate::core::recipe::UpdateRecipeError::NotFound => Self::NotFound,
        }
    }
}

impl ResponseError for UpdateRecipeError {}

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

impl From<crate::core::recipe::DeleteRecipeError> for DeleteRecipeError {
    fn from(value: crate::core::recipe::DeleteRecipeError) -> Self {
        match value {
            crate::core::recipe::DeleteRecipeError::Unknown(report) => Self::Unknown(report),
            crate::core::recipe::DeleteRecipeError::NotFound => Self::NotFound,
        }
    }
}

impl ResponseError for DeleteRecipeError {}

#[get("/recipes")]
pub(crate) async fn list_recipes(
    svc: Data<RecipeService>,
) -> Result<Json<Vec<RecipeDto>>, ListRecipeError> {
    let recipes = svc.list_recipes().await?;
    Ok(Json(recipes.into_iter().map(RecipeDto::from).collect()))
}

#[post("/recipes")]
pub(crate) async fn create_recipe(
    svc: Data<RecipeService>,
    Json(data): Json<NewRecipeDto>,
) -> Result<HttpResponse, CreateRecipeError> {
    let recipe = svc.create_recipe(data.into()).await?;

    Ok(HttpResponse::Created()
        .content_type(ContentType::json())
        .json(RecipeDto::from(recipe)))
}

#[put("/recipes/{recipe_id}")]
pub(crate) async fn update_recipe(
    svc: Data<RecipeService>,
    path: Path<i32>,
    Json(data): Json<RecipeDto>,
) -> Result<Json<RecipeDto>, UpdateRecipeError> {
    let recipe = svc
        .update_recipe(Recipe {
            recipe_id: path.into_inner(),
            name: data.name,
            description: data.description,
            ingredients: data.ingredients.into_iter().map(Ingredient::from).collect(),
            cooking_time: data.cooking_time,
            meal_type: data.meal_type.into(),
        })
        .await?;

    Ok(Json(recipe.into()))
}

#[delete("/recipes/{recipe_id}")]
pub(crate) async fn delete_recipe(
    svc: Data<RecipeService>,
    path: Path<i32>,
) -> Result<(), DeleteRecipeError> {
    svc.delete_recipe(path.into_inner()).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use actix_web::test;

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    mod list_recipes {
        use actix_web::{App, http::StatusCode};
        use sqlx::PgPool;

        use crate::Postgres;

        use super::*;

        macro_rules! setup_app {
            ($pool:expr) => {{
                let postgres = Postgres::new($pool);

                let recipe_service = RecipeService::new(postgres);

                test::init_service(
                    App::new()
                        .service(list_recipes)
                        .service(create_recipe)
                        .service(update_recipe)
                        .service(delete_recipe)
                        .app_data(Data::new(recipe_service.clone())),
                )
                .await
            }};
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_should_return_200(pool: PgPool) {
            let app = setup_app!(pool);

            let request = test::TestRequest::get().uri("/recipes").to_request();
            let response = test::call_service(&app, request).await;

            assert2::check!(response.status() == StatusCode::OK);
        }
    }

    // TODO: I'd add other tests here checking more for http specific properties like the status
    // code. The test themselves would be very simular to the ones provided in the repository.
}
