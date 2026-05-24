#!/bin/bash
set -e

DB_NAME="lmha3"
export DATABASE_URL="host=/var/run/postgresql dbname=$DB_NAME user=$(whoami)"
export HA_TOKEN=${HA_TOKEN:-"eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJkNDg3MjUzYmQ0MDg0ODU2OTY2YTU1ODA3NjhlMTFiMCIsImlhdCI6MTc3OTYyMTQ5MywiZXhwIjoyMDk0OTgxNDkzfQ.W0-9noqqRGQwILHmtmcVv9_8Ql83fF_7QZQrOrheGvY"}

echo "Flushing database $DB_NAME..."
psql -d "$DB_NAME" -c "TRUNCATE tenants, sessions, devices, telemetry CASCADE;"

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
