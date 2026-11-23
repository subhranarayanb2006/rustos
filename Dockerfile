# Dockerfile for RustOS 32-bit x86 Operating System
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    ca-certificates \
    grub-pc-bin \
    grub-common \
    xorriso \
    mtools \
    python3 \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly toolchain
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly-2024-05-15 \
    && rustup component add rust-src llvm-tools \
    && rustup target add i686-unknown-none

WORKDIR /rustos
COPY . /rustos

CMD ["python3", "tools/build.py", "--iso"]
