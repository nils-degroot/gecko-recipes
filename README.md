# Gecko Recipes

A simple recipe management API built with Rust, Actix Web, and PostgreSQL.

## Overview

Gecko Recipes is a RESTful web service for managing cooking recipes. It allows you to create, read, update, and delete recipes with ingredients, cooking times, and meal type classifications.

## Project Structure

```
src/
├── core/           # Business logic and domain models
├── persistance/    # Database layer and repository pattern
├── presentation/   # HTTP handlers and API endpoints
├── lib.rs         # Library configuration and server setup
└── main.rs        # Application entry point
```

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- PostgreSQL (if not using Docker)

### Database Setup

1. Start the PostgreSQL database using Docker Compose:

```bash
docker compose up -d database
```

2. Run database migrations:

```bash
cargo install sqlx-cli
sqlx migrate run --database-url "postgresql://gecko_user:gecko_password@localhost:5432/gecko_recipes"
```

### Running the Application

1. Set up environment variables by copying over the `.envrc.example` to `.envrc` and populating the missing values
2. Run the application:

```bash
cargo run
```

The API will be available at `http://127.0.0.1:8080`

## API Endpoints

- `GET /recipes` - List all recipes
- `POST /recipes` - Create a new recipe
- `PUT /recipes/{id}` - Update an existing recipe
- `DELETE /recipes/{id}` - Delete a recipe

### Recipe Data Structure

```json
{
  "recipe_id": 1,
  "name": "Pancakes",
  "description": "Fluffy breakfast pancakes",
  "cooking_time": 1800,
  "meal_type": "Breakfast",
  "ingredients": [
    {
      "name": "Flour",
      "quantity": 2.0,
      "quantity_type": "Cup"
    },
    {
      "name": "Milk",
      "quantity": 300.0,
      "quantity_type": "Milliliter"
    }
  ]
}
```

### Supported Quantity Types

- `Count` - For countable items (e.g., 3 eggs)
- `Kilo` - Kilograms
- `Gram` - Grams
- `Liter` - Liters
- `Milliliter` - Milliliters

### Meal Types

- `Breakfast`
- `Lunch`
- `Dinner`

## Development

### Running Tests

```bash
# Start test database
docker compose up

# Run tests
cargo test
```

### Database Migrations

SQLx provides a migration system for managing database schema changes. All migration files are stored in the `migrations/` directory. We've opted for this migration provder since it very lean.

#### Creating Migrations

To create a new migration:
```bash
sqlx migrate add <migration_name>
```

This creates a new SQL file with a timestamp prefix (e.g., `20250825175334_migration_name.sql`).

#### Running Migrations

To apply all pending migrations:

```bash
sqlx migrate run
```

## Configuration

The application can be configured via environment variables or command-line arguments:

- `DATABASE_URL`: PostgreSQL connection string
- `HOST`: Server bind address (default: 127.0.0.1)
- `PORT`: Server port (default: 8080)

## Docker Support

The project includes Docker Compose configuration for easy database setup:

- `database`: Main PostgreSQL instance (port 5432)

# Time log


| Description | Duration |
|-|-|
| Implement first schema version | 15m |
| Implement repository and its test suite | 1h 30m |
| Changeup schema to better suite the app | 15m |
| Implement core layer | 30m |
| Implement web layer | 30m |
| Write README.md and module documentation | 30m |
