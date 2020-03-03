
# armv7a currently only available in Rust 1.42
# https://github.com/rust-lang/rust/pull/68253

make:
	rustup override set beta
	rustup target add armv7a-none-eabi
	cargo build --target=armv7a-none-eabi
