# syntax=docker/dockerfile:latest
FROM alpine

WORKDIR /app

ARG TARGETPLATFORM

COPY $TARGETPLATFORM/error.html .
COPY $TARGETPLATFORM/error_server .

ENTRYPOINT [ "/app/error_server" ]
