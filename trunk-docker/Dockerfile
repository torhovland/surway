FROM rust:1.53-slim
RUN apt-get -y update && \
    apt-get -y install \
    binaryen \
    build-essential \
    git \
    libssl-dev \
    pkg-config && \
    rm -rf /var/lib/apt/lists/* && \
    rustup target add wasm32-unknown-unknown && \
    cargo install wasm-bindgen-cli --version 0.2.74 && \
    cargo install trunk --version 0.11.0 && \
    rm -rf /usr/local/cargo/registry
