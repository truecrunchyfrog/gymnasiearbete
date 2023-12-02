# Stage 1: Build the Rust program
FROM rust:1.55 as builder
WORKDIR /usr/src/app
COPY . .
RUN cargo build --release

# Stage 2: Create the Docker image
FROM debian:buster-slim
WORKDIR /usr/src/app
COPY --from=builder /usr/src/app/target/release/my_program .
RUN useradd -m myuser
USER myuser
EXPOSE 8080
CMD ["./my_program"]
