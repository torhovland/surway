FROM rust:1.51-slim
RUN apt-get -y update && \
    apt-get -y install \
    binaryen \
    build-essential \
    git \
    libssl-dev \
    pkg-config && \
    rm -rf /var/lib/apt/lists/* && \
    rustup target add wasm32-unknown-unknown && \
    cargo install wasm-bindgen-cli --version 0.2.73 && \
    cargo install trunk --version 0.10.0 && \
    rm -rf /usr/local/cargo/registry
