# syntax=docker/dockerfile:latest
FROM alpine

ARG TARGETPLATFORM

RUN if [ "$TARGETPLATFORM" == "linux/arm64" ]; \
		then export FOLDER="arm64"; \
		else export FOLDER="amd64"; \
		fi

COPY error.html .
COPY ${FOLDER}/error_server .

ENTRYPOINT [ "/app/error_server" ]
