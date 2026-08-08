FROM lukemathwalker/cargo-chef:latest-rust-alpine AS chef
WORKDIR /app


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json


FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin backend


FROM rust:alpine AS runner
RUN apk add --no-cache wget curl musl-dev gcc
RUN apk add --no-cache zig go nodejs npm

RUN rustup target add wasm32-wasip2
RUN cargo install wasm-tools --locked

ENV TINYGO_VERSION=0.35.0
RUN wget https://github.com/tinygo-org/tinygo/releases/download/v${TINYGO_VERSION}/tinygo${TINYGO_VERSION}.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf tinygo${TINYGO_VERSION}.linux-amd64.tar.gz \
    && rm tinygo${TINYGO_VERSION}.linux-amd64.tar.gz
ENV PATH="${PATH}:/usr/local/tinygo/bin"

RUN npm install -g assemblyscript

WORKDIR /app

# WASI P1 -> P2 adapter component
RUN wget https://github.com/bytecodealliance/wasmtime/releases/download/v47.0.3/wasi_snapshot_preview1.command.wasm \
    -O /app/wasi_snapshot_preview1.wasm

RUN addgroup -S app && adduser -S app -G app
COPY --from=builder --chown=app:app /app/target/release/backend .

USER app
ENV HOST="0.0.0.0"
ENV PORT=3000
CMD ["./backend"]
EXPOSE 3000
