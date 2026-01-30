# Stage 1: Planning
FROM rust:1.93-alpine AS chef
RUN apk add --no-cache musl-dev openssl-dev pkgconfig
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-json recipe.json

# Stage 2: Caching dependencies
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the layer that will be cached
RUN cargo chef cook --release --recipe-json recipe.json

# Stage 3: Build application
COPY . .
RUN cargo build --release && cp target/release/mega-uploader-auth /app/mega-uploader-auth

# Runtime stage
FROM alpine:3.21

# Install CA certificates (required for HTTPS calls to AWS/Cognito)
# and necessary runtime libraries
RUN apk add --no-cache ca-certificates libssl3

# Working directory in the final image
WORKDIR /app

# Copy the binary from the build stage
COPY --from=builder /app/mega-uploader-auth /app/mega-uploader-auth

# Expose port 80
EXPOSE 80

# Set default environment variables
# Listen on 0.0.0.0 to be accessible from outside the container
ENV SERVER_ADDR=0.0.0.0:80
ENV RUST_LOG=info

# Run the application
CMD ["./mega-uploader-auth"]
