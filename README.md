# Concurrent Computing CW2

To build / launch just use the Makefile inside `./core` as usual.

An explanation of how it works:
- The `c` and `s` files in `./core` are compiled with Linaro GCC as usual.
- However the linker is given an additional static library `libhilevel` which defines all of the `hilevel_handler_*` symbols.
- `libhilevel` is written in Rust and is automatically built using `./hilevel/build.sh`.
- `libhilevel` also depends on a number of symbols exported via `./core/bindings.h`; clang is used by the `bindgen` crate to parse the C header, and generate C->Rust bindings (see `./hilevel/build.rs`).
