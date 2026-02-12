#!/usr/bin/env sh
set -eu

# One-time schema/bootstrap loader for PostgreSQL RDS.
# Usage:
#   DATABASE_URL='postgres://user:pwd@host:5432/app_db?sslmode=require' ./deploy/ec2/init-rds.sh
# Optional:
#   INCLUDE_SEED=0 ./deploy/ec2/init-rds.sh   # skip 13-e2br3-seed.sql
#   PROJECT_DIR=/path/to/repo ./deploy/ec2/init-rds.sh

DATABASE_URL="${DATABASE_URL:-}"
PROJECT_DIR="${PROJECT_DIR:-$(pwd)}"
INCLUDE_SEED="${INCLUDE_SEED:-1}"

if [ -z "${DATABASE_URL}" ]; then
  echo "DATABASE_URL is required."
  echo "Example:"
  echo "  DATABASE_URL='postgres://user:pwd@host:5432/app_db?sslmode=require' ./deploy/ec2/init-rds.sh"
  exit 1
fi

if ! command -v psql >/dev/null 2>&1; then
  echo "psql is required but not found on PATH."
  exit 1
fi

SQL_DIR="${PROJECT_DIR}/docs/dev_initial"
if [ ! -d "${SQL_DIR}" ]; then
  echo "SQL directory not found: ${SQL_DIR}"
  echo "Set PROJECT_DIR to your repository root."
  exit 1
fi

FILES="
03-safetydb-schema.sql
04-e2br3_N.sql
05-e2br3_C.sql
06-e2br3_D.sql
07-e2br3_E.sql
08-e2br3_F.sql
09-e2br3_G.sql
10-e2br3_H.sql
11-terminology.sql
12-triggers.sql
"

if [ "${INCLUDE_SEED}" = "1" ]; then
  FILES="${FILES}
13-e2br3-seed.sql"
fi

echo "Using SQL directory: ${SQL_DIR}"
echo "Applying SQL files to: ${DATABASE_URL}"

for f in ${FILES}; do
  path="${SQL_DIR}/${f}"
  if [ ! -f "${path}" ]; then
    echo "Missing file: ${path}"
    exit 1
  fi
  echo "==> ${f}"
  psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -f "${path}"
done

echo "RDS bootstrap complete."
