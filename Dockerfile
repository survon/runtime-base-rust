# Stage 1: Build the Rust application
FROM rust:1.72-alpine as builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev

# Set the working directory inside the container
WORKDIR /app

# Copy the entire project into the container
COPY . .

# Build the application in release mode for ARM (Raspberry Pi) and x86
ARG TARGETARCH
RUN cargo build --release --target $TARGETARCH-unknown-linux-musl

# Stage 2: Create a minimal runtime image
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache libgcc libstdc++ openssl

# Set the working directory inside the container
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/survon /app/survon

# Define the default command to run the binary
CMD ["/app/survon"]
