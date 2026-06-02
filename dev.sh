#!/usr/bin/env bash
set -e

# Load .env if it exists
if [ -f .env ]; then
    echo "Loading .env file..."
    export $(grep -v '^#' .env | xargs)
fi

# Ensure mandatory vars are set or have safe local defaults
DB_NAME=${DB_NAME:-"lmha3"}
export DATABASE_URL=${DATABASE_URL:-"host=/var/run/postgresql dbname=$DB_NAME user=$(whoami)"}
export LMHA_SCHEDULER_DEBUG=${LMHA_SCHEDULER_DEBUG:-1}

# 1. Ensure DB exists
if ! psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    echo "Creating database $DB_NAME..."
    createdb "$DB_NAME"
fi

# 2. (Migrations are now handled by the server on startup)
echo "Ensuring DB is ready..."

# 3. Check for admin user, create if missing
CHECK_USER=$(psql -t -A -c "SELECT count(*) FROM tenants WHERE username='admin';" "$DB_NAME")
if [ "$CHECK_USER" == "0" ]; then
    echo "Creating default 'admin' user (password: admin)..."
    cargo run -p lmha-admin -- --username admin --password admin
fi

# 4. Start Server
echo "Starting Server..."
cargo run -p server
