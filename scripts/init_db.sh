#!/usr/bin/env bash
set -x
set -eo pipefail

if [[ -z "${SKIP_DOCKER}" ]] && ! [ -x "$(command -v docker)" ]; then
  echo >&2 "Error: docker is not installed."
  exit 1
fi
if ! [ -x "$(command -v psql)" ]; then
  echo >&2 "Error: psql is not installed."
  exit 1
fi
if ! [ -x "$(command -v sqlx)" ]; then
  echo >&2 "Error: sqlx is not installed."
  echo >&2 "Use:"
  echo >&2 "  cargo install sqlx-cli --no-default-features --features native-tls,postgres"
  echo >&2 "to install it."
  exit 1
fi

DB_USER=${POSTGRES_USER:=postgres}
DB_PASSWORD="${POSTGRES_PASSWORD:=password}"
DB_NAME="${POSTGRES_DB:=newsletter}"
DB_PORT="${POSTGRES_PORT:=5432}"
CONTAINER_NAME="${POSTGRES_CONTAINER_NAME:=zero2prod_db}"

if [[ -z "${SKIP_DOCKER}" ]]; then
  docker stop ${CONTAINER_NAME} || true

  docker run \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    --rm \
    --name ${CONTAINER_NAME} \
    -d postgres \
    postgres -N 1000
fi

export PGPASSWORD="${DB_PASSWORD}"
until psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "postgres" -c '\q'; do
  echo >&2 "Postgres is still unavailable - sleeping"
  sleep 1
done
echo >&2 "Postgres is up and running on port ${DB_PORT}!"

export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create
sqlx migrate run

echo >&2 "Postgres has been migrated, ready to go!"
