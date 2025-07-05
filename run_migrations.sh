#!/bin/bash

# Database URL
DATABASE_URL="sqlite:quiz.db"

# Check if sqlx-cli is installed
if ! command -v sqlx &> /dev/null; then
    echo "sqlx-cli is not installed. Installing..."
    cargo install sqlx-cli
fi

# Create database if it doesn't exist
echo "Creating database if it doesn't exist..."
sqlx database create --database-url "$DATABASE_URL"

# Run migrations
echo "Running migrations..."
sqlx migrate run --database-url "$DATABASE_URL"

echo "Migrations completed successfully!"
echo "Database location: quiz.db"