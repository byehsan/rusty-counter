FROM rust:1.83.0 as builder
WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./

# RUN cargo fetch
COPY . .
RUN cargo build --release



FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /usr/src/app/target/release/rusty-counter /app/rusty-counter
COPY .env /app/
VOLUME ["/app/data"]
ENV RUST_LOG=info
EXPOSE 8080
CMD ["/app/rusty-counter"]