FROM rust:alpine AS builder

# Arguments
ARG CURRENT_COMMIT
ARG CURRENT_BUILD

# Set the working directory
WORKDIR /usr/src/app
COPY . .

# Install necessary build dependencies, including protobuf compiler
RUN apk add --no-cache musl-dev openssl-dev protobuf-dev

# Compile the controller with both plugin runtimes: pelican and cloudflare
# use the wasm runtime, the local plugin uses wasmite
RUN cargo build -p controller --features wasm-plugins,wasmite-plugins --release

# Runtime stage: Use a minimal Alpine image
FROM alpine:latest

# Install necessary runtime dependencies
RUN apk add --no-cache curl unzip openjdk21-jre

# Set the working directory
WORKDIR /app

# Copy entrypoint script
COPY .github/docker/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Copy the controller binary from the builder stage
COPY --from=builder /usr/src/app/target/release/controller /app/controller

# Expose the port
EXPOSE 8080

# Run the controller
CMD ["./entrypoint.sh"]