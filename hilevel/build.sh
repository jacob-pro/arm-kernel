#!/bin/bash
set -x #echo on

# armv7a currently only available in Rust 1.42
# https://github.com/rust-lang/rust/pull/68253
# allocator_api currently unstable
rustup override set nightly
rustup target add armv7a-none-eabi
# This is a hack to force build.rs to run every time
touch build.rs && cargo build --target=armv7a-none-eabi
