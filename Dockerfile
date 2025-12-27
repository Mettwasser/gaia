FROM lukemathwalker/cargo-chef:latest-rust-1-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Dependency cache
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

# Tell SQLx to use the .sqlx folder we copied instead of trying to connect to a DB
ENV SQLX_OFFLINE=true

# Build the final binary
RUN cargo build --release
# -------------------

FROM debian:bookworm-slim
WORKDIR /home/appuser
COPY --from=builder /app/target/release/gaia .
COPY --from=builder /app/migrations ./migrations

# required by idk.. poise/serenity (?)
RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

ENTRYPOINT ["./gaia"]