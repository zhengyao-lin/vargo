# vargo2

A wrapper around cargo so that it automatically runs verification (w/ [Verus](https://github.com/verus-lang/verus/)) on related crates.

Note:
- Not related to "vargo" as the internal build script in Verus (I need to rename at some point, open to suggestions).
- The way it works is also similar to [`cargo verus`](https://github.com/verus-lang/verus/pull/1138) (i.e., via `RUSTC_WRAPPER`),
but we require no change to the Verus repo, and we translate flags for `rustc` to flags for `verus`.

## System Requirements

Probably only Linux and macOS are supported (on x86 or ARM).

## Install

To build and install `vargo`, run the following:
```
cargo install --git https://github.com/zhengyao-lin/vargo.git
```
This will build the latest commit of [Verus](https://github.com/verus-lang/verus/), and then pack it along with `vargo`.
To use another fork of Verus, there are two options via environment variables:
1. To use another remote repo/branch/commit, run `VERUS_REPO=<Git URL> [VERUS_BRANCH=...] [VERUS_COMMIT=...] cargo install ...`
2. To use a local repo, run `VERUS_LOCAL=<Locally checked-out Verus repo> cargo install ...`

If `cargo install` is successful, a command `vargo` will be available in your system, with similar usage to `cargo`.

## Usage

Most commands are similar to `cargo`, for example,
```
vargo build [--release] # Verify and build the current crate/workspace
vargo check # Verify without building
vargo test
vargo bench
...
```
There is also a `vargo verus` command, which calls the `verus` executable packed in the current installed version of `vargo`.

During `vargo build`, if a crate contains a dependency called `vstd`, then Verus will be invoked (in addition to the original `rustc` call) to verify the given file.
The output will look something like
```
   Compiling normal_rust_crate_1 v0.1.0 (...)
   Compiling normal_rust_crate_2 v0.1.0 (...)
   Compiling verified_crate v0.1.0 (...)
   Verifying verified_crate v0.1.0 (...)
       Verus verified_crate: XX verified in XXs
```

If verification fails, `vargo` will also signal `cargo` to stop with an error message.
```
   Compiling verified_crate v0.1.0 (...)
   Verifying verified_crate v0.1.0 (...)
error: assertion failed
   --> src/main.rs:...
    |
136 |         assert(false);
    |                ^^^^^ assertion failed

       Verus verified_crate: XX verified, 1 failed, in XXs
Error: Failed to call Verus

Caused by:
    Verus failed with non-zero exit code
error: could not compile `verified_crate` (lib) due to 1 previous error
```

Since the build process is still managed by `cargo` (except we wrap `rustc` with `vargo rustc` via `RUSTC_WRAPPER`), build caches work the same way.

## Additional Verus flags

Sometimes it's helpful to provide additional flags to `verus`, such as increasing the rlimit.
This is done at the crate level by adding the following flag in `Cargo.toml` in your crate:
```
[verus]
extra_flags = "--rlimit 10086"
```

## Known issues

- Cannot explicitly import spec items from another crate (e.g. `use crate_a::spec_def`), since we still use `rustc` for compilation.
