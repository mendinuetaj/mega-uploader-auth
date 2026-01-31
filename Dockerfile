# Stage 1: Planning
FROM --platform=$BUILDPLATFORM rust:1.92-bookworm AS chef
RUN cargo install cargo-chef && rustup target add aarch64-unknown-linux-gnu
WORKDIR /app

# Instalar herramientas de compilación cruzada y dependencias de sistema para arm64
RUN dpkg --add-architecture arm64 && \
    apt-get update && apt-get install -y \
    gcc-aarch64-linux-gnu \
    libc6-dev-arm64-cross \
    libssl-dev:arm64 \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Caching dependencies
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json

# Configure environment for cross-compilation to arm64
ENV PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig \
    CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
    CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc \
    CXX_aarch64_unknown_linux_gnu=aarch64-linux-gnu-g++

# Build dependencies - cross-compiled para arm64
RUN cargo chef cook --release --recipe-path recipe.json --target aarch64-unknown-linux-gnu

# Stage 3: Build application
COPY . .
RUN cargo build --release --target aarch64-unknown-linux-gnu && \
    cp target/aarch64-unknown-linux-gnu/release/mega-uploader-auth /app/mega-uploader-auth

# Runtime stage
FROM debian:bookworm-slim AS runtime

# Install CA certificates y librerías runtime necesarias
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from the build stage
COPY --from=builder /app/mega-uploader-auth /app/mega-uploader-auth

# Expose port 80
EXPOSE 80

# Set default environment variables
ENV SERVER_ADDR=0.0.0.0:80
ENV RUST_LOG=info

# Run the application
CMD ["./mega-uploader-auth"]
