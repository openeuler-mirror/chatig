# Base image
FROM openeuler/openeuler:22.03-lts-sp4
WORKDIR /app

# Install gcc, openssl-devel, libffi-devel, and Rust
RUN dnf install -y gcc openssl-devel libffi-devel

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

RUN source /root/.cargo/env

# Set up Rust environment
ENV PATH="/root/.cargo/bin:${PATH}"

# Copy the application code
COPY . /app

# Build the application
RUN cargo build --release

# Start the application
CMD ["cargo", "run", "--release"]

# Expose PostgreSQL port
EXPOSE 8081