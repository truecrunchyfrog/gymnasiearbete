FROM rust:latest

WORKDIR /usr/src/rust_backend
COPY . .

RUN cargo install --path .

CMD ["rust_backend"]