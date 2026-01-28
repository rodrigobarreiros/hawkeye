# Build stage
FROM rust:1.87-bookworm AS builder

# Install GStreamer development dependencies
RUN apt-get update && apt-get install -y \
    libgstreamer1.0-dev \
    libgstreamer-plugins-base1.0-dev \
    libgstreamer-plugins-bad1.0-dev \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    gstreamer1.0-tools \
    gstreamer1.0-rtsp \
    libgstrtspserver-1.0-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to cache dependencies
RUN mkdir -p src/main/rust && \
    echo "fn main() {}" > src/main/rust/main.rs && \
    echo "pub fn dummy() {}" > src/main/rust/lib.rs

# Build dependencies only (cache layer)
RUN cargo build --release && rm -rf src target/release/deps/pipeline* target/release/deps/libpipeline* target/release/.fingerprint/pipeline*

# Copy actual source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime GStreamer dependencies and curl for health checks
RUN apt-get update && apt-get install -y \
    libgstreamer1.0-0 \
    gstreamer1.0-plugins-base \
    gstreamer1.0-plugins-good \
    gstreamer1.0-plugins-bad \
    gstreamer1.0-plugins-ugly \
    gstreamer1.0-libav \
    gstreamer1.0-tools \
    gstreamer1.0-rtsp \
    libgstrtspserver-1.0-0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/pipeline-rtsp /app/pipeline-rtsp

# Copy video resources
COPY src/main/resources/*.mp4 /app/resources/

# Set default video path (can be overridden)
ENV VIDEO_PATH=/app/resources/camera1.mp4

EXPOSE 8554 9001

CMD ["/app/pipeline-rtsp"]
