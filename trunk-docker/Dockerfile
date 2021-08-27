FROM rust:1.54-slim
RUN apt-get -y update && \
    apt-get -y install \
    binaryen \
    build-essential \
    git \
    libssl-dev \
    pkg-config && \
    rm -rf /var/lib/apt/lists/* && \ 
    cargo install trunk --version 0.13.1 && \
    rm -rf /usr/local/cargo/registry
