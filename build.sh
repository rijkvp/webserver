#!/bin/sh
# Build script for linux using musl toolchain
cargo build --release --target x86_64-unknown-linux-musl
strip ./target/x86_64-unknown-linux-musl/release/webserver