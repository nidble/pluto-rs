FROM rust:1.56 as builder

RUN USER=root cargo install sqlx-cli --no-default-features --features postgres
RUN USER=root cargo new --bin pluto-rs

ENV SQLX_OFFLINE=true

WORKDIR /pluto-rs
COPY ./Cargo.toml ./Cargo.toml
RUN touch ./src/lib.rs
RUN cargo build --release --bin pluto-rs
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/pluto_rs*
RUN cargo build --release --bin pluto-rs


FROM debian:bookworm-slim
ARG APP=/usr/src/app

EXPOSE 3030

# RUN apt-get update \
#     && apt-get install -y ca-certificates tzdata \
#     && rm -rf /var/lib/apt/lists/*
# ENV TZ=Etc/UTC 

RUN apt update && apt install -y openssl libssl-dev

ENV APP_USER=ferris \
    RUST_LOG=info \
    DATABASE_URL=''

RUN groupadd $APP_USER \
    && useradd -g $APP_USER $APP_USER \
    && mkdir -p ${APP}

COPY --from=builder /pluto-rs/target/release/pluto-rs ${APP}/pluto-rs

RUN chown -R $APP_USER:$APP_USER ${APP}

USER $APP_USER
WORKDIR ${APP}

CMD ["./pluto-rs"]