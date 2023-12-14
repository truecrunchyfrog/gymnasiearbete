# Use alpine as the base image
FROM alpine:latest

RUN groupadd -r user && useradd -r -g group user

ENTRYPOINT [ "timeout", "30", "tail", "-f", "/dev/null", "&&", "/app/program" ]