use std::time::Duration;

use eyre::Context;
use sqlx::{PgPool, PgTransaction, QueryBuilder, types::Json};

use crate::persistance::recipe::{
    CreateRecipeError, DeleteRecipeError, IngredientEntity, ListRecipeError, MealType,
    MutableIngredientEntity, MutableRecipeEntity, RecipeEntity, RecipeRepository,
    UpdateRecipeError,
};

#[derive(Debug, Clone)]
pub(crate) struct Postgres {
    pool: PgPool,
}

impl Postgres {
    pub(crate) fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RecipeRepository for Postgres {
    async fn list_recipes(&self) -> Result<Vec<RecipeEntity>, ListRecipeError> {
        let data = sqlx::query!(
            r#"
                WITH ingredients_json AS (
                    SELECT recipe_id, ROW_TO_JSON(i) AS json FROM ingredient i
                ), ingredients_grouped AS (
                    SELECT recipe_id, JSON_AGG(ij.json) AS ingredients
                    FROM ingredients_json ij
                    GROUP BY recipe_id
                )

                SELECT
                    r.recipe_id,
                    r.name,
                    description,
                    cooking_time_secs,
                    ig.ingredients AS "ingredients: Json<Vec<IngredientEntity>>",
                    meal_type AS "meal_type: MealType"
                    FROM recipe r
                LEFT JOIN ingredients_grouped ig ON ig.recipe_id = r.recipe_id
            "#
        )
        .fetch_all(&self.pool)
        .await
        .wrap_err("Failed to get recipes")?;

        Ok(data
            .into_iter()
            .map(|row| RecipeEntity {
                recipe_id: row.recipe_id,
                name: row.name,
                description: row.description,
                ingredients: row
                    .ingredients
                    .map(|ingredient| ingredient.0)
                    .unwrap_or_default(),
                cooking_time: row
                    .cooking_time_secs
                    .map(|value| Duration::from_secs(value as u64)),
                meal_type: row.meal_type,
            })
            .collect())
    }

    async fn create_recipe(
        &self,
        entity: MutableRecipeEntity,
    ) -> Result<RecipeEntity, CreateRecipeError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .wrap_err("Failed to open transaction")?;

        let result = sqlx::query!(
            r#"
                INSERT INTO recipe (
                    name,
                    description,
                    cooking_time_secs,
                    meal_type
                ) VALUES ($1, $2, $3, $4)
                RETURNING recipe_id, name, description, cooking_time_secs, meal_type AS "meal_type: MealType"
            "#,
            entity.name,
            entity.description,
            entity.cooking_time.map(|time| time.as_secs() as i64),
            &entity.meal_type as &MealType
        ).fetch_one(&mut *tx).await.wrap_err("Failed to insert recipe")?;

        let ingredients = create_ingredients(&mut tx, result.recipe_id, &entity.ingredients)
            .await
            .wrap_err("Failed to create ingredients")?;

        tx.commit().await.wrap_err("Failed to commit transaction")?;

        Ok(RecipeEntity {
            recipe_id: result.recipe_id,
            name: result.name,
            description: result.description,
            ingredients,
            cooking_time: result
                .cooking_time_secs
                .map(|time| Duration::from_secs(time as u64)),
            meal_type: result.meal_type,
        })
    }

    async fn update_recipe(
        &self,
        recipe_id: i32,
        entity: MutableRecipeEntity,
    ) -> Result<RecipeEntity, UpdateRecipeError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .wrap_err("Failed to open transaction")?;

        let result = sqlx::query!(
            r#"
                UPDATE recipe SET
                    name = $1,
                    description = $2,
                    cooking_time_secs = $3,
                    meal_type = $4
                WHERE recipe_id = $5
                RETURNING recipe_id, name, description, cooking_time_secs, meal_type AS "meal_type: MealType"
            "#,
            entity.name,
            entity.description,
            entity.cooking_time.map(|time| time.as_secs() as i64),
            &entity.meal_type as &MealType,
            recipe_id
        ).fetch_one(&mut *tx).await.map_err(|error| {
            match error {
                sqlx::Error::RowNotFound => UpdateRecipeError::NotFound,
                error => UpdateRecipeError::Unknown(eyre::Report::from(error).wrap_err("Failed to update recipe")),
            }
        })?;

        sqlx::query!("DELETE FROM ingredient WHERE recipe_id = $1", recipe_id)
            .execute(&mut *tx)
            .await
            .wrap_err("Failed to clear out old ingredients")?;

        let ingredients = create_ingredients(&mut tx, result.recipe_id, &entity.ingredients)
            .await
            .wrap_err("Failed to create ingredients")?;

        tx.commit().await.wrap_err("Failed to commit transaction")?;

        Ok(RecipeEntity {
            recipe_id: result.recipe_id,
            name: result.name,
            description: result.description,
            ingredients,
            cooking_time: result
                .cooking_time_secs
                .map(|time| Duration::from_secs(time as u64)),
            meal_type: result.meal_type,
        })
    }

    async fn delete_recipe(&self, recipe_id: i32) -> Result<(), DeleteRecipeError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .wrap_err("Failed to start transaction")?;

        // First delete all ingredients for this recipe
        sqlx::query!("DELETE FROM ingredient WHERE recipe_id = $1", recipe_id)
            .execute(&mut *tx)
            .await
            .wrap_err("Failed to delete ingredients")?;

        // Then delete the recipe
        let result = sqlx::query!("DELETE FROM recipe WHERE recipe_id = $1", recipe_id)
            .execute(&mut *tx)
            .await
            .wrap_err("Failed to delete recipe")?;

        if result.rows_affected() > 0 {
            tx.commit().await.wrap_err("Failed to commit transaction")?;
            Ok(())
        } else {
            tx.rollback()
                .await
                .wrap_err("Failed to rollback transaction")?;
            Err(DeleteRecipeError::NotFound)
        }
    }
}

async fn create_ingredients(
    transaction: &mut PgTransaction<'_>,
    recipe_id: i32,
    ingredients: &[MutableIngredientEntity],
) -> Result<Vec<IngredientEntity>, sqlx::Error> {
    // If no ingredients are present and we try to run the query provided, an error would always be
    // returned since the query is invalid at that point
    if ingredients.is_empty() {
        return Ok(vec![]);
    }

    let mut query_builder = QueryBuilder::new(
        r#"INSERT INTO ingredient (recipe_id, ingredient_order, name, quantity, quantity_type) "#,
    );

    query_builder.push_values(
        ingredients.iter().enumerate(),
        |mut builder, (idx, ingredient)| {
            builder
                .push_bind(recipe_id)
                .push_bind(idx as i32)
                .push_bind(&ingredient.name)
                .push_bind(ingredient.quantity)
                .push_bind(&ingredient.quantity_type);
        },
    );

    query_builder.push(
        " RETURNING ingredient_id, recipe_id, ingredient_order, name, quantity, quantity_type",
    );

    query_builder
        .build_query_as::<'_, IngredientEntity>()
        .fetch_all(&mut **transaction)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistance::recipe::{MealType, QuantityType};
    use assert2::{check, let_assert};

    static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

    fn create_test_ingredient(
        name: &str,
        quantity: f32,
        quantity_type: QuantityType,
    ) -> MutableIngredientEntity {
        MutableIngredientEntity {
            name: name.to_string(),
            quantity,
            quantity_type,
        }
    }

    fn create_test_recipe(name: &str, meal_type: MealType) -> MutableRecipeEntity {
        MutableRecipeEntity {
            name: name.to_string(),
            description: None,
            ingredients: vec![
                create_test_ingredient("Ingredient 1", 2.0, QuantityType::Count),
                create_test_ingredient("Ingredient 2", 500.0, QuantityType::Gram),
            ],
            cooking_time: Some(Duration::from_secs(3600)),
            meal_type,
        }
    }

    mod list_recipes {
        use super::*;

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_returns_empty_list_when_no_recipes_exist(pool: PgPool) {
            let repository = Postgres::new(pool);

            let result = repository.list_recipes().await;

            let_assert!(Ok(recipes) = result);
            check!(recipes.is_empty());
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_returns_all_recipes_when_recipes_exist(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe1 = create_test_recipe("Pancakes", MealType::Breakfast);
            let recipe2 = create_test_recipe("Pasta", MealType::Dinner);

            let_assert!(Ok(_) = repository.create_recipe(recipe1).await);
            let_assert!(Ok(_) = repository.create_recipe(recipe2).await);

            let result = repository.list_recipes().await;

            let_assert!(Ok(recipes) = result);
            check!(recipes.len() == 2);
            check!(recipes.iter().any(|r| r.name == "Pancakes"));
            check!(recipes.iter().any(|r| r.name == "Pasta"));
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_includes_ingredients_in_recipe_list(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = create_test_recipe("Test Recipe", MealType::Lunch);
            let_assert!(Ok(_) = repository.create_recipe(recipe).await);

            let result = repository.list_recipes().await;

            let_assert!(Ok(recipes) = result);
            check!(recipes.len() == 1);

            let recipe = &recipes[0];

            check!(recipe.ingredients.len() == 2);
            check!(recipe.ingredients[0].name == "Ingredient 1");
            check!(recipe.ingredients[1].name == "Ingredient 2");
        }
    }

    mod create_recipe {
        use super::*;

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_creates_recipe_with_generated_id(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = create_test_recipe("New Recipe", MealType::Breakfast);

            let result = repository.create_recipe(recipe).await;

            let_assert!(Ok(created_recipe) = result);
            check!(created_recipe.recipe_id > 0);
            check!(created_recipe.name == "New Recipe");
            check!(matches!(created_recipe.meal_type, MealType::Breakfast));
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_creates_recipe_with_ingredients(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = create_test_recipe("Recipe with Ingredients", MealType::Dinner);

            let result = repository.create_recipe(recipe).await;

            let_assert!(Ok(created_recipe) = result);
            check!(created_recipe.ingredients.len() == 2);

            let ingredient1 = &created_recipe.ingredients[0];
            check!(ingredient1.name == "Ingredient 1");
            check!(ingredient1.quantity == 2.0);
            check!(matches!(ingredient1.quantity_type, QuantityType::Count));

            let ingredient2 = &created_recipe.ingredients[1];
            check!(ingredient2.name == "Ingredient 2");
            check!(ingredient2.quantity == 500.0);
            check!(matches!(ingredient2.quantity_type, QuantityType::Gram));
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_creates_recipe_without_ingredients(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = MutableRecipeEntity {
                name: "Simple Recipe".to_string(),
                description: None,
                ingredients: vec![],
                cooking_time: None,
                meal_type: MealType::Lunch,
            };

            let result = repository.create_recipe(recipe).await;

            let_assert!(Ok(created_recipe) = result);
            check!(created_recipe.name == "Simple Recipe");
            check!(created_recipe.ingredients.is_empty());
            let_assert!(None = created_recipe.description);
            let_assert!(None = created_recipe.cooking_time);
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_preserves_ingredient_order(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = MutableRecipeEntity {
                name: "Ordered Recipe".to_string(),
                description: None,
                ingredients: vec![
                    create_test_ingredient("First", 1.0, QuantityType::Count),
                    create_test_ingredient("Second", 2.0, QuantityType::Count),
                    create_test_ingredient("Third", 3.0, QuantityType::Count),
                ],
                cooking_time: None,
                meal_type: MealType::Lunch,
            };

            let result = repository.create_recipe(recipe).await;

            let_assert!(Ok(created_recipe) = result);
            check!(created_recipe.ingredients.len() == 3);
            check!(created_recipe.ingredients[0].name == "First");
            check!(created_recipe.ingredients[1].name == "Second");
            check!(created_recipe.ingredients[2].name == "Third");
            check!(created_recipe.ingredients[0].ingredient_order == 0);
            check!(created_recipe.ingredients[1].ingredient_order == 1);
            check!(created_recipe.ingredients[2].ingredient_order == 2);
        }
    }

    mod update_recipe {
        use super::*;

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_updates_existing_recipe(pool: PgPool) {
            let repository = Postgres::new(pool);

            let original_recipe = create_test_recipe("Original", MealType::Breakfast);
            let_assert!(Ok(created) = repository.create_recipe(original_recipe).await);
            let updated_recipe = MutableRecipeEntity {
                name: "Updated Recipe".to_string(),
                description: Some("Updated description".to_string()),
                ingredients: vec![create_test_ingredient(
                    "New Ingredient",
                    1.5,
                    QuantityType::Liter,
                )],
                cooking_time: Some(Duration::from_secs(1800)),
                meal_type: MealType::Dinner,
            };

            let result = repository
                .update_recipe(created.recipe_id, updated_recipe)
                .await;

            let_assert!(Ok(updated) = result);
            check!(updated.recipe_id == created.recipe_id);
            check!(updated.name == "Updated Recipe");

            let_assert!(Some(description) = updated.description);
            check!(description == "Updated description");

            let_assert!(Some(cooking_time) = updated.cooking_time);
            check!(cooking_time == Duration::from_secs(1800));
            check!(let MealType::Dinner = updated.meal_type);
            check!(updated.ingredients.len() == 1);
            check!(updated.ingredients[0].name == "New Ingredient");
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_replaces_all_ingredients_on_update(pool: PgPool) {
            let repository = Postgres::new(pool);

            let original_recipe = create_test_recipe("Recipe", MealType::Lunch);
            let_assert!(Ok(created) = repository.create_recipe(original_recipe).await);
            check!(created.ingredients.len() == 2);
            let updated_recipe = MutableRecipeEntity {
                name: "Updated".to_string(),
                description: None,
                ingredients: vec![
                    create_test_ingredient("A", 1.0, QuantityType::Count),
                    create_test_ingredient("B", 2.0, QuantityType::Count),
                    create_test_ingredient("C", 3.0, QuantityType::Count),
                ],
                cooking_time: None,
                meal_type: MealType::Lunch,
            };

            let result = repository
                .update_recipe(created.recipe_id, updated_recipe)
                .await;

            let_assert!(Ok(updated) = result);
            check!(updated.ingredients.len() == 3);
            check!(updated.ingredients[0].name == "A");
            check!(updated.ingredients[1].name == "B");
            check!(updated.ingredients[2].name == "C");
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_returns_not_found_error_for_nonexistent_recipe(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = create_test_recipe("Update", MealType::Breakfast);

            let result = repository.update_recipe(99999, recipe).await;

            let_assert!(Err(UpdateRecipeError::NotFound) = result);
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_updates_to_empty_ingredients_list(pool: PgPool) {
            let repository = Postgres::new(pool);

            let original_recipe = create_test_recipe("Recipe", MealType::Dinner);
            let_assert!(Ok(created) = repository.create_recipe(original_recipe).await);
            let updated_recipe = MutableRecipeEntity {
                name: "No Ingredients".to_string(),
                description: None,
                ingredients: vec![],
                cooking_time: None,
                meal_type: MealType::Dinner,
            };

            let result = repository
                .update_recipe(created.recipe_id, updated_recipe)
                .await;

            let_assert!(Ok(updated) = result);
            check!(updated.ingredients.is_empty());
        }
    }

    mod delete_recipe {
        use super::*;

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_deletes_existing_recipe(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = create_test_recipe("To Delete", MealType::Breakfast);
            let_assert!(Ok(created) = repository.create_recipe(recipe).await);

            let result = repository.delete_recipe(created.recipe_id).await;

            let_assert!(Ok(()) = result);
            let_assert!(Ok(list_result) = repository.list_recipes().await);
            check!(list_result.is_empty());
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_deletes_recipe_and_its_ingredients(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe = create_test_recipe("Recipe with Ingredients", MealType::Lunch);
            let_assert!(Ok(created) = repository.create_recipe(recipe).await);

            let ingredient_count = sqlx::query_scalar!(
                "SELECT COUNT(*) as \"count!\" FROM ingredient WHERE recipe_id = $1",
                created.recipe_id
            )
            .fetch_one(&repository.pool)
            .await
            .expect("Failed to fetch count");

            check!(ingredient_count == 2);

            let result = repository.delete_recipe(created.recipe_id).await;
            let_assert!(Ok(()) = result);

            let ingredient_count_after = sqlx::query_scalar!(
                "SELECT COUNT(*) as \"count!\" FROM ingredient WHERE recipe_id = $1",
                created.recipe_id
            )
            .fetch_one(&repository.pool)
            .await
            .expect("Failed to fetch count");

            check!(ingredient_count_after == 0);
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_returns_not_found_error_for_nonexistent_recipe(pool: PgPool) {
            let repository = Postgres::new(pool);

            let result = repository.delete_recipe(99999).await;

            let_assert!(Err(DeleteRecipeError::NotFound) = result);
        }

        #[sqlx::test(migrator = "super::MIGRATOR")]
        async fn it_does_not_affect_other_recipes_when_deleting(pool: PgPool) {
            let repository = Postgres::new(pool);

            let recipe1 = create_test_recipe("Keep This", MealType::Breakfast);
            let recipe2 = create_test_recipe("Delete This", MealType::Lunch);

            let_assert!(Ok(created1) = repository.create_recipe(recipe1).await);
            let_assert!(Ok(created2) = repository.create_recipe(recipe2).await);

            let result = repository.delete_recipe(created2.recipe_id).await;
            let_assert!(Ok(()) = result);

            let_assert!(Ok(remaining_recipes) = repository.list_recipes().await);
            check!(remaining_recipes.len() == 1);
            check!(remaining_recipes[0].recipe_id == created1.recipe_id);
            check!(remaining_recipes[0].name == "Keep This");
        }
    }
}
