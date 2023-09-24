# Use the Rust base image
FROM rust:1.70.0-slim-bullseye AS build

# Set the working directory in the container to /app
WORKDIR /app

# Copy the local directory contents into the container
COPY . .
# Compile the Rust program
RUN cargo build --release

# Create a lightweight final stage
FROM debian:bullseye-slim

# Download latest listing of available packages:
RUN apt-get -y update
# Upgrade already installed packages:
RUN apt-get -y upgrade
# Install a new package:
RUN apt-get -y install firejail

# Copy the binary from the "build" stage
COPY --from=build /app/target/release/jail /tmp/app/

EXPOSE 8000
# Set the command to run the Rust program
CMD ["jail"]
