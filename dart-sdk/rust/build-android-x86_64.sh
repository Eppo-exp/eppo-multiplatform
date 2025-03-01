#!/usr/bin/env bash

# Need to set NDK_HOME env var

export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/x86_64-linux-android35-clang"
export RANLIB="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ranlib"
export AR_x86_64_linux_android="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
cargo build --release --target x86_64-linux-android -p eppo_sdk
