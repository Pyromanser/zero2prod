FROM rust:1.71 as base

RUN apt update \
    && apt upgrade -qy \
    && apt install lld clang -y

FROM base AS chef

ARG SQLX_OFFLINE=true

RUN cargo install cargo-chef

WORKDIR /usr/src/app

FROM chef AS planner

COPY . .

RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder

COPY --from=planner /usr/src/app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .

RUN cargo build --release

FROM debian:bullseye-slim as runtime

ENV APP_ENVIRONMENT production

RUN apt update -y \
    && apt upgrade -qy \
    && apt install -y --no-install-recommends openssl ca-certificates \
    && apt clean autoclean -y \
    && apt autoremove -y \
    && apt clean -y \
    && rm -rf /var/lib/apt/lists/* \
    && rm -rf /var/lib/{apt,dpkg,cache,log}/

WORKDIR /usr/src/app

COPY --from=builder /usr/src/app/target/release/zero2prod zero2prod

COPY configuration configuration

ENTRYPOINT ["./zero2prod"]