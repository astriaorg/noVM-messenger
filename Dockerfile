# Stage 1: Build the Rust project
FROM rust:1.81 AS builder

RUN apt-get update && apt-get install -y clang
# Set the working directory
WORKDIR /app

# Copy the Cargo manifest files
COPY Cargo.toml Cargo.lock ./

# Copy the actual source code
COPY . .

# Build the project in release mode to produce an optimized binary
RUN cargo build --release

# Stage 2: Create a minimal runtime image
FROM debian:bookworm-slim

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/uprollup /usr/local/bin/up

# Expose the gRPC port
EXPOSE 50051

# Set the binary as the container entry point
ENTRYPOINT ["/usr/local/bin/up"]
