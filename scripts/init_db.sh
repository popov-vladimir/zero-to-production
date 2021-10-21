#!/usr/bin/env bash
set -x
set -eo pipefail


DB_USER=${POSTGRES_USER:=postgres}

DB_PASSWORD="${POSTGRES_PASSWORD:=postgres}"


DB_NAME="${POSTGRES_DB:=newsletter}"

DB_PORT="${POSTGRES_PORT:=5432}"


if [[ -z "${SKIP_DOCKER}" ]]; then
    docker run --rm \
    --name postgres \
    -e POSTGRES_USER=${DB_USER} \
    -e POSTGRES_PASSWORD=${DB_PASSWORD} \
    -e POSTGRES_DB=${DB_NAME} \
    -p "${DB_PORT}":5432 \
    -d postgres \
    postgres -N 1000
fi

export PGPASSWORD=${DB_PASSWORD}

until  psql -h "localhost" -U "${DB_USER}" -p "${DB_PORT}" -d "${DB_NAME}" -d postgres  -c '\q'
do >&2 echo "postgres is not ready"
    sleep 1
done

>&2 echo "postgres is up and running"


export DATABASE_URL=postgres://${DB_USER}:${DB_PASSWORD}@localhost:${DB_PORT}/${DB_NAME}
sqlx database create

sqlx migrate run

>&2 echo "migration done"
