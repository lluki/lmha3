#!/usr/bin/env bash
set -e

# Ensure mandatory vars are set or have safe local defaults
LMHA_DB_NAME=${LMHA_DB_NAME:-"lmha3"}
export LMHA_DATABASE_URL=${LMHA_DATABASE_URL:-"host=/var/run/postgresql dbname=$LMHA_DB_NAME user=$(whoami)"}
export LMHA_SCHEDULER_DEBUG=${LMHA_SCHEDULER_DEBUG:-1}

# 1. Ensure DB exists
if ! psql -lqt | cut -d \| -f 1 | grep -qw "$LMHA_DB_NAME"; then
    echo "Creating database $LMHA_DB_NAME..."
    createdb "$LMHA_DB_NAME"
fi

# 2. (Migrations are now handled by the server on startup)
echo "Ensuring DB is ready..."

# 3. Check for admin user, create if missing
CHECK_USER=$(psql -t -A -c "SELECT count(*) FROM tenants WHERE username='admin';" "$LMHA_DB_NAME")
if [ "$CHECK_USER" == "0" ]; then
    echo "Creating default 'admin' user (password: admin)..."
    cargo run -p lmha-admin -- --username admin --password admin
fi

# 4. Start Server
echo "Starting Server..."
cargo run -p server
