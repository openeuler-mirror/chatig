# Base image
FROM openeuler/openeuler:22.03-lts-sp4
WORKDIR /var/lib/pgsql

# Install PostgreSQL and its dependencies
RUN dnf install -y postgresql postgresql-server postgresql-devel

# Switch to postgres user to configure PostgreSQL
USER postgres

# Initialize PostgreSQL data directory as the postgres user
RUN mkdir -p /var/lib/pgsql/data && \
    /usr/bin/initdb -D /var/lib/pgsql/data

# Configure PostgreSQL to allow external connections
RUN sed -i "s/#listen_addresses = 'localhost'/listen_addresses = '*'/g" /var/lib/pgsql/data/postgresql.conf && \
    echo "host    all             chatig           0.0.0.0/0           md5" >> /var/lib/pgsql/data/pg_hba.conf

# Start PostgreSQL and create a new user and database
USER postgres
CMD ["sh", "-c", "pg_ctl -D /var/lib/pgsql/data start && \
    sleep 5 && \
    psql -c \"CREATE USER chatig WITH PASSWORD 'chatig';\" && \
    psql -c \"CREATE DATABASE chatig OWNER chatig;\" && \
    tail -f /dev/null"]

# Expose PostgreSQL port
EXPOSE 5432
