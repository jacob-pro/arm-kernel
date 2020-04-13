#!/bin/bash
set -x #echo on

if ! [ -x "$(command -v rustup)" ]; then
  echo 'Downloading rustup...'
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  source $HOME/.cargo/env
fi

if ! [ -x "$(command -v clang)" ]; then
  echo 'Error: please make sure clang is in your path'
  exit 1
fi

# armv7a currently only available in Rust 1.42
# https://github.com/rust-lang/rust/pull/68253
# allocator_api currently unstable
rustup override set nightly
rustup target add armv7a-none-eabi
# This is a hack to force build.rs to run every time
touch build.rs && cargo build --target=armv7a-none-eabi
