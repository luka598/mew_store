FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app/mew_store
COPY target/release/mew_store /app/mew_store

EXPOSE 3000
CMD ["/app/mew_store"]
