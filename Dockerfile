# syntax=docker/dockerfile:latest
FROM rust:alpine as builder

WORKDIR /app
ENV CROSS_CONTAINER_IN_CONTAINER=true
ARG TARGETPLATFORM

RUN rustup toolchain install nightly
RUN [ "$TARGETPLATFORM" == "linux/arm64"* ] && cargo install cross || return 0

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN <<EOF
case $TARGETPLATFORM in
	linux/amd64)
		cargo +nightly build --release -Z sparse-registry
		;;
	linux/arm64)
		cross +nightly build --release -Z sparse-registry --target aarch64-unknown-linux-musl
		cp target/{${{ matrix.architecture }}/,}release/error_server
		;;
	*)
		echo "Unknown target platform '$TARGETPLATFORM'!"
		exit 1
esac
EOF

FROM alpine

COPY error.html .
COPY --from=builder /app/target/release/error_server .

ENTRYPOINT [ "/app/error_server" ]
