# Use Debian as the base image
FROM debian:bullseye-slim

ARG DEBIAN_FRONTEND=noninteractive

# Update the package index and install required packages
RUN apt-get update && \
    apt-get install -y firejail

# Create a directory for the exposed folder
RUN mkdir /app

# Specify the entry point with Firejail
# ENTRYPOINT ["firejail", "--timeout=00:00:01", "--rlimit-cpu=1", "--rlimit-nofile=10", "--rlimit-fsize=0", "--rlimit-nproc=2", "--restrict-namespaces", "--private", "--read-only=/", "/app/program"]
ENTRYPOINT [ "firejail", "sh" ]