# https://windsoilder.github.io/writing_dockerfile_in_rust_project.html
FROM rust:1.56 AS chef

ENV SQLX_OFFLINE=true \
    DATABASE_URL=''

RUN cargo install sqlx-cli --no-default-features --features postgres

# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN cargo install cargo-chef

WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY . .
RUN cargo install --path .

EXPOSE 3030

# We do not need the Rust toolchain to run the binary!
# FROM gcr.io/distroless/cc-debian11
FROM debian:bookworm-slim
COPY --from=builder /usr/local/cargo/bin/pluto-rs  /usr/local/bin

ENV APP_USER=ferris \
    RUST_LOG=info

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER

# RUN chown -R $APP_USER:$APP_USER ${APP}
RUN chown -R $APP_USER:$APP_USER /usr/local/bin/pluto-rs
USER $APP_USER

CMD ["/usr/local/bin/pluto-rs"]
