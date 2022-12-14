FROM rust:alpine as builder

WORKDIR /app

RUN rustup toolchain install nightly

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo +nightly build --release -Z sparse-registry --timings

FROM alpine

WORKDIR /app

COPY error.html .
COPY --from=builder /app/target/release/error_server .

ENTRYPOINT [ "/app/error_server" ]
