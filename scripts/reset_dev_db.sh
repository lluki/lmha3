#!/bin/bash
set -e

LMHA_DB_NAME=${LMHA_DB_NAME:-"lmha3"}
export LMHA_DATABASE_URL=${LMHA_DATABASE_URL:-"host=/var/run/postgresql dbname=$LMHA_DB_NAME user=$(whoami)"}

echo "Flushing database $LMHA_DB_NAME..."
psql -d "$LMHA_DB_NAME" -c "TRUNCATE tenants, sessions, devices, telemetry CASCADE;"

echo "Creating admin user..."
cargo run -q -p lmha-admin -- --username admin --password admin

for X in 81 83; do
    for Y in {1..6}; do
        USERNAME="h${X}whg${Y}"
        echo "Creating user $USERNAME..."
        cargo run -q -p lmha-admin -- --username "$USERNAME" --password "$USERNAME"
    done
done

echo "Database setup complete."
