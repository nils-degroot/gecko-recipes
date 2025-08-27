CREATE TYPE meal_type AS ENUM ('Breakfast', 'Lunch', 'Dinner');

CREATE TABLE recipe (
	recipe_id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	name TEXT NOT NULL CHECK ("name" <> ''),
	description TEXT CHECK ("description" <> ''),
	cooking_time_secs BIGINT,
	meal_type meal_type NOT NULL
);

CREATE TYPE quantity_type AS ENUM ('Count', 'Kilo', 'Gram', 'Liter', 'Milliliter');

CREATE TABLE ingredient (
	ingredient_id INTEGER GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
	recipe_id INTEGER NOT NULL REFERENCES recipe ("recipe_id"),
	ingredient_order INTEGER NOT NULL,
	name TEXT NOT NULL CHECK ("name" <> ''),
	quantity REAL NOT NULL,
	quantity_type quantity_type NOT NULL
);
