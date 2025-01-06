FROM rust:alpine AS builder

# Arguments
ARG CURRENT_COMMIT
ARG CURRENT_BUILD

# Set the working directory
WORKDIR /usr/src/app
COPY . .

# Install necessary build dependencies, including protobuf compiler
RUN apk add --no-cache musl-dev openssl-dev protobuf-dev

# Compile the controller with the wasm-drivers feature
RUN cargo build -p controller --features wasm-drivers --release

# Runtime stage: Use a minimal Alpine image
FROM alpine:latest

# Set the working directory
WORKDIR /app

# Copy the controller binary from the builder stage
COPY --from=builder /usr/src/app/target/release/controller /app/controller

# Expose the port
EXPOSE 12892

# Run the controller
CMD ["./controller"]