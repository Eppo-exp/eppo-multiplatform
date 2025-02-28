#!/usr/bin/env bash

export CC_aarch64_linux_android="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/aarch64-linux-android35-clang"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="$CC_aarch64_linux_android"
export AR_aarch64_linux_android="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
export RANLIB="$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ranlib"

cargo build --release --target aarch64-linux-android -p eppo_dart
