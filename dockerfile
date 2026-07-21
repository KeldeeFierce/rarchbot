# -----Build stage------
FROM rust:1.91 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src


ENV SQLX_OFFLINE=true
COPY . .
RUN touch src/main.rs
RUN cargo build --release -vv

#-----Runtime stage------
FROM debian:trixie-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libc-bin && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rarchbot ./rarchbot
RUN useradd -m rarchbot && chown rarchbot:rarchbot /app
RUN mkdir /docker_data && chown rarchbot:rarchbot /docker_data 
RUN touch /docker_data/app.db && chown rarchbot:rarchbot /docker_data/app.db 
USER rarchbot
CMD ["./rarchbot"]
