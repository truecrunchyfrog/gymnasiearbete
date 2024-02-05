FROM rust:latest

WORKDIR /usr/src/rust_frontend
COPY . .

RUN cargo install --path .

CMD ["rust_backend"]