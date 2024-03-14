FROM rust:1.76.0-slim-buster AS builder
ENV RUSTFLAGS="-C target-feature=-crt-static"
# DEPENDENCIES HERE
RUN apt update && apt install -y musl-dev musl-tools openssl libssl-dev pkg-config
WORKDIR /app
COPY ./ /app

RUN cargo build --release
RUN strip target/release/proxy
# Create db if not exists + run migrations

# SECOND STAGE?

FROM debian:buster-slim AS deploy
# Install deps
RUN apt update && apt install -y gcc openssl libssl-dev pkg-config
# ENV
ENV PROTOCOL=http
ENV RUST_LOG=info
ENV ADMIN_USERNAME=usercommon
ENV ADMIN_PASSWORD=wow
ENV ARTICLES_DOMAIN=articles:3000
ENV PROXY_DOMAIN=0.0.0.0:3001



WORKDIR /app
# Copying binary
COPY --from=builder /app/target/release/proxy ./


ENTRYPOINT ["./proxy"]
