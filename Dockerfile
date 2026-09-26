FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app/store
COPY target/release/mew_service /app/mew_service

EXPOSE 3000
CMD ["/app/mew_service"]
