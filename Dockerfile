# syntax=docker/dockerfile:latest
FROM alpine

ARG TARGETPLATFORM

COPY error.html .
COPY $TARGETPLATFORM/error_server .

ENTRYPOINT [ "/app/error_server" ]
