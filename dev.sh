#!/usr/bin/env bash
set -e

DB_NAME="lmha3"
export DATABASE_URL="host=/var/run/postgresql dbname=$DB_NAME user=$(whoami)"
export HA_TOKEN=${HA_TOKEN:-"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJkNDg3MjUzYmQ0MDg0ODU2OTY2YTU1ODA3NjhlMTFiMCIsImlhdCI6MTc3OTYyMTQ5MywiZXhwIjoyMDk0OTgxNDkzfQ.W0-9noqqRGQwILHmtmcVv9_8Ql83fF_7QZQrOrheGvY"}
export MQTT_USER=${MQTT_USER:-"admin"}
export MQTT_PASSWORD=${MQTT_PASSWORD:-"freebird"}

# 1. Ensure DB exists
if ! psql -lqt | cut -d \| -f 1 | grep -qw "$DB_NAME"; then
    echo "Creating database $DB_NAME..."
    createdb "$DB_NAME"
fi

# 2. Apply migrations
echo "Applying migrations..."
psql "$DB_NAME" -f migrations/001_initial_schema.sql > /dev/null
psql "$DB_NAME" -f migrations/002_add_sessions.sql > /dev/null

# 3. Check for admin user, create if missing
CHECK_USER=$(psql -t -A -c "SELECT count(*) FROM tenants WHERE username='admin';" "$DB_NAME")
if [ "$CHECK_USER" == "0" ]; then
    echo "Creating default 'admin' user (password: admin)..."
    cargo run -p lmha-admin -- --username admin --password admin
fi

# 4. Start Server
echo "Starting Server..."
cargo run -p server
