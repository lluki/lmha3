#!/usr/bin/env bash
set -e

DB_NAME="lmha3"

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
    # Using a pre-calculated Argon2 hash for 'admin' to avoid requiring lmha-admin build during boot
    # Hash for 'admin': $argon2id$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$P/PZ4+J9C9+J6J9C9+J6J9C9+J6J9C9+J6J9C9+J6J9
    # Actually, it's safer to just suggest running lmha-admin if user is missing, 
    # but for "bring up" convenience, we'll try to use cargo run.
    DATABASE_URL="postgres:///lmha3" cargo run -p lmha-admin <<EOF
admin
admin
admin
EOF
fi

# 4. Start API
echo "Starting API..."
DATABASE_URL="postgres:///lmha3" HA_TOKEN=dummy cargo run -p api
