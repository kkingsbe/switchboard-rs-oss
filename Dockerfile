# Use Node.js base image
FROM node:22-slim

# Add labels for metadata
LABEL maintainer="Switchboard"
LABEL description="Switchboard agent container for scheduled Kilo Code CLI execution"

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    build-essential \
    procps \
    file \
    libssl-dev \
    pkg-config \
    sudo \
    && rm -rf /var/lib/apt/lists/*

# Install Playwright and its browsers
RUN npx playwright install --with-deps chromium

# Install Rust and Cargo using rustup
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install Discli
RUN cargo install discli

# Install the Kilo Code CLI
RUN npm i -g @kilocode/cli@0.26.0

# Copy .kilocode config to root's home directory
COPY .kilocode /root/.kilocode

# Create workspace directory
RUN mkdir -p /workspace

# Set working directory
WORKDIR /workspace

# Set the default entrypoint
ENTRYPOINT ["kilo"]