# Build Stage
FROM rust:1.80-slim-bookworm as builder
WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y libpcap-dev pkg-config protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY . .

# Build with features for Linux packet capture
RUN cargo build --release --features "packet-capture libpcap"

# Runtime Stage
FROM debian:bookworm-slim
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y libpcap0.8 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/apex_core .

# Default env vars
ENV CORE_GRPC_HOST=0.0.0.0
ENV CORE_GRPC_PORT=50051
ENV DEMO_MODE=true

CMD ["./apex_core"]
