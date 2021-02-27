FROM rust:1.50
RUN rustup target add wasm32-unknown-unknown && \
    cargo install wasm-bindgen-cli --version 0.2.71 && \
    cargo install --locked trunk --version 0.8.2
