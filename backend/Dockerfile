# syntax=docker/dockerfile:1

FROM rust:1-slim-bookworm AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked

# ---- compute dependency recipe from Cargo.toml/Cargo.lock ----
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ---- build deps (cached layer) then the release binary ----
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin hotel-backend --bin seed

# ---- dev stage: used by docker-compose, hot reload via cargo-watch ----
FROM chef AS dev
RUN cargo install cargo-watch --locked \
    && cargo install sqlx-cli --locked --no-default-features --features postgres,rustls
EXPOSE 3000
CMD ["sh", "-c", "sqlx migrate run && cargo watch -x run"]

# ---- sqlx-cli binary (separate stage so it doesn't slow down the app builder) ----
FROM chef AS sqlx-installer
RUN cargo install sqlx-cli --locked --no-default-features --features postgres,rustls

# ---- runtime stage: minimal prod image ----
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/hotel-backend /usr/local/bin/hotel-backend
COPY --from=builder /app/target/release/seed /usr/local/bin/seed
COPY --from=sqlx-installer /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /app/migrations ./migrations
EXPOSE 3000
CMD ["sh", "-c", "sqlx migrate run && exec hotel-backend"]
