//! # Gecko Recipes
//!
//! A REST API for managing cooking recipes built with Actix Web and PostgreSQL.
//!
//! This application implements Clean Architecture principles with a clear separation of concerns
//! and dependency inversion. The project is organized into three main layers:
//!
//! ## Architecture
//!
//! - **Core Layer** (`core/`) - Contains business logic, domain models, and use cases
//! - **Persistence Layer** (`persistance/`) - Data access abstracted through repository interfaces
//! - **Presentation Layer** (`presentation/`) - HTTP handlers and external API contracts
//!
//! Dependencies flow inward: presentation → core ← persistence, ensuring the core
//! business logic remains independent of external concerns like databases or web frameworks.

use actix_web::{App, HttpServer, web::Data};
use eyre::Context;
use persistance::implementation::postgres::Postgres;
use secrecy::{ExposeSecret, SecretBox};
use sqlx::PgPool;

/// Core business logic and domain models for recipes and ingredients.
mod core;
/// Data persistence layer with repository pattern and database implementations.
mod persistance;
/// HTTP request handlers and API endpoint definitions.
mod presentation;

pub(crate) type RecipeService = crate::core::recipe::RecipeService<Postgres>;

#[derive(Debug)]
/// Configuration used to start the server
pub struct Config {
    /// Url used to connect to the database instance
    pub database_url: SecretBox<str>,
    /// Host to bind to
    pub host: String,
    /// Port to bind to
    pub port: u16,
}

pub async fn server(config: Config) -> eyre::Result<()> {
    tracing_subscriber::fmt::init();

    let pg_pool = PgPool::connect(config.database_url.expose_secret())
        .await
        .wrap_err("Failed to connect to database instance")?;

    let postgres = Postgres::new(pg_pool);

    let recipe_service = RecipeService::new(postgres);

    HttpServer::new(move || {
        App::new()
            .service(crate::presentation::recipe::list_recipes)
            .service(crate::presentation::recipe::search_recipes)
            .service(crate::presentation::recipe::create_recipe)
            .service(crate::presentation::recipe::update_recipe)
            .service(crate::presentation::recipe::delete_recipe)
            .app_data(Data::new(recipe_service.clone()))
    })
    .bind((config.host.as_str(), config.port))
    .wrap_err("Failed to bind server")?
    .run()
    .await?;

    Ok(())
}
