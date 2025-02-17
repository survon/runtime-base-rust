# Use an ARM64 Alpine Linux image (musl-based)
FROM arm64v8/alpine:latest

# Install necessary packages
RUN apk update && apk add --no-cache ca-certificates

# Create the module directory in the container and copy the local directory into it
RUN mkdir -p /tmp/wasteland
COPY tmp/wasteland /tmp/wasteland

# Copy the cross-compiled runtime into the container
COPY target/aarch64-unknown-linux-musl/release/runtime-base-rust /usr/local/bin/survon
RUN chmod +x /usr/local/bin/survon

# Set the default command to run the Survon runtime
CMD ["/usr/local/bin/survon"]
