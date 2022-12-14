FROM rust:slim as builder

WORKDIR /app

RUN apt-get update && apt-get install -y gcc

RUN rustup toolchain install nightly

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo +nightly build --release -Z sparse-registry

FROM alpine

WORKDIR /app

COPY error.html .
COPY --from=builder /app/target/release/ErrorServer .

ENTRYPOINT [ "/app/ErrorServer" ]
