FROM alpine

WORKDIR /app

COPY error.html .
COPY target/aarch64-unknown-linux-gnu/release/error_server .

ENTRYPOINT [ "/app/error_server" ]
