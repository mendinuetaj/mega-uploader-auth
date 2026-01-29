# Build stage
FROM rust:1.84-alpine AS builder

# Install build dependencies for packages that use C (like openssl)
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Set the working directory
WORKDIR /app

# Copy dependency configuration files first to leverage Docker cache
COPY Cargo.toml ./

# Create a project skeleton to pre-compile dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy the actual source code
COPY src ./src

# Compile the actual application (this updates the previously compiled binary)
# We use touch to ensure main.rs is considered "new" and gets recompiled
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.21

# Install CA certificates (required for HTTPS calls to AWS/Cognito)
# and necessary runtime libraries
RUN apk add --no-cache ca-certificates libssl3

# Working directory in the final image
WORKDIR /app

# Copy the binary from the build stage
COPY --from=builder /app/target/release/mega-uploader-auth /app/mega-uploader-auth

# Expose port 80
EXPOSE 80

# Set default environment variables
# Listen on 0.0.0.0 to be accessible from outside the container
ENV SERVER_ADDR=0.0.0.0:80
ENV RUST_LOG=info

# Run the application
CMD ["./mega-uploader-auth"]
