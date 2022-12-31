FROM alpine

WORKDIR /app

COPY error.html .
COPY target/release/error_server error_server

ENTRYPOINT [ "/app/error_server" ]
