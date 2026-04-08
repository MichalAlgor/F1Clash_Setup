FROM rust:1.85 AS builder
WORKDIR /app
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/f1clash_setup /usr/local/bin/
COPY --from=builder /app/static /static
EXPOSE 3000
CMD ["f1clash_setup"]
