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
RUN cargo build --release --bin hotel-backend

# ---- dev stage: used by docker-compose, hot reload via cargo-watch ----
FROM chef AS dev
RUN cargo install cargo-watch --locked \
    && cargo install sqlx-cli --locked --no-default-features --features postgres,rustls
EXPOSE 3000
CMD ["sh", "-c", "sqlx migrate run && cargo watch -x run"]

# ---- runtime stage: minimal prod image (not used by docker-compose) ----
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/hotel-backend /usr/local/bin/hotel-backend
COPY --from=builder /app/migrations ./migrations
EXPOSE 3000
CMD ["hotel-backend"]
