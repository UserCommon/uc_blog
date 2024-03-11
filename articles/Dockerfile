FROM rust:1.76.0-slim-buster AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static"
# DEPENDENCIES HERE
RUN apt update && apt install -y musl-dev musl-tools openssl libssl-dev pkg-config
WORKDIR /app
COPY ./ /app

RUN cargo build --release
RUN strip target/release/uc_blog_articles
RUN cargo install sqlx-cli
# Create db if not exists + run migrations

# SECOND STAGE?

FROM debian:buster-slim AS deploy
# Install deps
RUN apt update && apt install -y gcc openssl libssl-dev pkg-config
# ENV
ENV PROTOCOL=http
ENV DOMAIN=0.0.0.0:3000
ENV DATABASE_URL=sqlite://db/articles.sqlite3
ENV ARTICLES=./articles
ENV RUST_LOG=info

WORKDIR /app
# Copying binary
COPY --from=builder /app/target/release/uc_blog_articles ./
# Copying articles
COPY --from=builder /app/articles ./articles/
# Copying database
COPY --from=builder /app/db/ ./db/

ENTRYPOINT ["./uc_blog_articles"]
