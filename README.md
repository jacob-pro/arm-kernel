# ARM Kernel

Bristol COMS20001_2019 Concurrent Computing (Yr 2), Coursework 2

Confirmed mark: 75

A minimal operating system kernel for the RealView Platform Baseboard for Cortex-A8.
Features include:

- Pre-emptive multi-tasking
- MLFQ Scheduler
- Fork, exec, and exit system calls
- Blocking IPC using Unix style pipes 

## Building

To build / launch just use the Makefile inside `./core`, it will work exactly as usual.

An explanation of how it works:
- The `c` and `s` files in `./core` are compiled with Linaro GCC as usual.
- However the linker is given an additional static library `libhilevel` which defines all of the `hilevel_handler_*` symbols.
- `libhilevel` is written in Rust and is automatically built using `./hilevel/build.sh`.
- `libhilevel` also depends on a number of symbols exported via `./core/bindings.h`; clang is used by the `bindgen` crate to parse the C header, and generate C->Rust bindings (see `./hilevel/build.rs`).

## Testing

The unit tests for the Rust library can be run using `cargo test` in the `./hilevel` directory.
(You may need to run `build.sh` at least once before, to configure cargo)

## Why Rust

As far as I know there are only 3 viable languages that have good support for systems programming on the target platform in a bare metal environment: C, C++ and Rust. 

The points I considered were that:
- Performance wise they are all very similar
- Rust and C++ have a large standard library including collections, but C does not.
- Rust is by far the most safe, with the compiler able to detect many errors. C is the least safe, C++ is somewhere in between depending on the version and which features are used.
- Rust has a tightly integrated and easy to use package manager (cargo)
- Most existing kernels and research have already focused on C, it would be interesting to find out what strengths and weaknesses Rust could have for kernel development.
 
