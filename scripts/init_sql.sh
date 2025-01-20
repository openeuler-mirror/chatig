#!/bin/bash

DB_HOST=127.0.0.1
DB_PORT=5432
DB_NAME=chatig
SQL_FILE=$(dirname "$(realpath "$0")")/init.sql

while [[ $# -gt 0 ]]; do
    case $1 in
        -u)
            DB_USER="$2"
            shift 
            shift 
            ;;
        -p)
            DB_PASSWORD="$2"
            shift 
            shift 
            ;;
        *)
            shift 
            ;;
    esac
done

if [ -z "$DB_USER" ] || [ -z "$DB_PASSWORD" ]; then
    echo "Usage: sh init_sql.sh -u <username> -p <password>"
    exit 1
fi

echo "Initializing database"

PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -p $DB_PORT -U $DB_USER -d $DB_NAME -f $SQL_FILE

echo "Database initialization complete."