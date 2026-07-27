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
RUN apk add --no-cache wget musl-dev gcc zig
RUN rustup target add wasm32-wasip2
RUN cargo install wasm-tools --locked

RUN addgroup -S app && adduser -S app -G app
WORKDIR /app
COPY --from=builder --chown=app:app /app/target/release/backend .

USER app
ENV HOST="0.0.0.0"
ENV PORT=3000
CMD ["./backend"]
EXPOSE 3000
