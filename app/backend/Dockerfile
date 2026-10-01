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


FROM debian:trixie-slim AS tools
ARG TARGETARCH
RUN apt-get update && apt-get install -y wget tar xz-utils
WORKDIR /tools

ENV TINYGO_VERSION=0.42.0
ENV GO_VERSION=1.25.1
ENV WASI_SDK_VERSION=34

RUN ARCH="${TARGETARCH:-$(dpkg --print-architecture)}" && \
    case "${ARCH}" in \
        "arm64"|"aarch64") GO_ARCH="arm64"; WASI_ARCH="arm64" ;; \
        *) GO_ARCH="amd64"; WASI_ARCH="x86_64" ;; \
    esac && \
    wget -qO- "https://github.com/tinygo-org/tinygo/releases/download/v${TINYGO_VERSION}/tinygo${TINYGO_VERSION}.linux-${GO_ARCH}.tar.gz" | tar -xz && \
    wget -qO- "https://go.dev/dl/go${GO_VERSION}.linux-${GO_ARCH}.tar.gz" | tar -xz && \
    mkdir -p /tools/wasi-sdk && \
    wget -qO- "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-${WASI_SDK_VERSION}/wasi-sdk-${WASI_SDK_VERSION}.0-${WASI_ARCH}-linux.tar.gz" | tar -xz -C /tools/wasi-sdk --strip-components=1

# WASI P1 -> P2 adapter component
RUN wget https://github.com/bytecodealliance/wasmtime/releases/download/v49.0.1/wasi_snapshot_preview1.command.wasm \
    -O /tools/wasi_snapshot_preview1.command.wasm && \
    wget https://github.com/bytecodealliance/wasmtime/releases/download/v49.0.1/wasi_snapshot_preview1.reactor.wasm \
    -O /tools/wasi_snapshot_preview1.reactor.wasm


FROM rust:slim-trixie AS runner
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang \
    nodejs \
    npm \
    python3 \
    python3-pip \
    ca-certificates \
    git \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add wasm32-wasip2
RUN cargo install --locked wasm-tools wit-bindgen-cli

COPY --from=tools /tools/go /usr/local/go
COPY --from=tools /tools/tinygo /usr/local/tinygo
COPY --from=tools /tools/wasi-sdk /opt/wasi-sdk
COPY --from=tools /tools/wasi_snapshot_preview1.command.wasm /opt/wasi_snapshot_preview1.command.wasm
COPY --from=tools /tools/wasi_snapshot_preview1.reactor.wasm /opt/wasi_snapshot_preview1.reactor.wasm

ENV PATH="/usr/local/go/bin:/usr/local/tinygo/bin:${PATH}"
ENV WASI_SDK_PATH="/opt/wasi-sdk"

# Install wit-bindgen-go for TinyGo
ENV GOBIN=/usr/local/bin
RUN CGO_ENABLED=0 go install go.bytecodealliance.org/cmd/wit-bindgen-go@latest

# Install componentize-py for Python components
RUN pip install --no-cache-dir --break-system-packages componentize-py

# Install jco, componentize-js, and preview2-shim for JavaScript components
RUN npm install -g @bytecodealliance/jco @bytecodealliance/componentize-js @bytecodealliance/preview2-shim && npm cache clean --force
ENV NODE_PATH="/usr/local/lib/node_modules"

WORKDIR /app

RUN groupadd -g 1001 appgroup && \
    useradd --no-log-init -u 1001 -g appgroup -m -s /usr/sbin/nologin appuser
COPY --chown=1001:1001 --from=builder /app/target/release/backend .

USER 1001:1001
ENV HOST="0.0.0.0"
ENV PORT=3000
EXPOSE 3000

CMD ["./backend"]
