# ---stage 1 (build)---
FROM rust:1.65.0 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release

# ---stage 2 (runtime)---
FROM debian:bullseye-slim
COPY --from=builder /app/target/release/aurora_dsql_sample /usr/local/bin/aurora_dsql_sample
EXPOSE 3000
CMD ["aurora_dsql_sample"]
