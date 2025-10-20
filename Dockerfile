# Multi-stage build for minimal image size
FROM ubuntu:22.04 as builder

# Install only build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app
COPY . .

# Build release binary and strip it
RUN cargo build --release && \
    strip target/release/runtime-base-rust

# Runtime stage - minimal dependencies only
FROM ubuntu:22.04

# Install runtime GUI dependencies + ONE lightweight browser
RUN apt-get update && apt-get install -y \
    libgtk-3-0 \
    libwebkit2gtk-4.0-37 \
    libayatana-appindicator3-1 \
    netsurf-gtk \
    xvfb \
    x11vnc \
    fluxbox \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Copy only the binary from builder
COPY --from=builder /app/target/release/runtime-base-rust /usr/local/bin/runtime-base-rust

# Copy the wasteland directory with modules and knowledge
COPY --from=builder /app/wasteland /wasteland

# Set up X11 environment
ENV DISPLAY=:99
ENV RESOLUTION=1024x768x24

# Create minimal startup script
RUN echo '#!/bin/bash\n\
Xvfb :99 -screen 0 $RESOLUTION &\n\
sleep 2\n\
fluxbox &\n\
x11vnc -display :99 -forever -nopw -listen localhost -xkb &\n\
sleep 2\n\
exec "$@"' > /start.sh && chmod +x /start.sh

EXPOSE 5900

ENTRYPOINT ["/start.sh"]
CMD ["/usr/local/bin/runtime-base-rust"]
