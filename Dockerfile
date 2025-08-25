FROM rust:latest AS builder

WORKDIR /app

COPY . .

ENV DATABASE_URL=sqlite://dev.db
ENV SQLX_OFFLINE=true

RUN cargo install sqlx-cli --no-default-features --features native-tls,sqlite

RUN cargo sqlx database create

RUN cargo sqlx migrate run

RUN cargo sqlx prepare

RUN cargo build --release

# --------------------------
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 && rm -rf /var/lib/apt/lists/*

WORKDIR /home/appuser

# Needed for DNS resolution - can't access api.warframestat.us otherwise
RUN apt-get update && apt-get install -y curl dnsutils


COPY --from=builder ./target/release/gaia .
COPY --from=builder ./migrations ./migrations

CMD ["./gaia"]