#!/bin/bash -e

## Linux
cargo build --target=x86_64-unknown-linux-musl --release
# Create splitdebug
objcopy --only-keep-debug target/x86_64-unknown-linux-musl/release/cli target/x86_64-unknown-linux-musl/release/geometrify.debug
cp target/x86_64-unknown-linux-musl/release/cli target/x86_64-unknown-linux-musl/release/geometrify
strip --strip-debug --strip-unneeded target/x86_64-unknown-linux-musl/release/geometrify
objcopy --add-gnu-debuglink=target/x86_64-unknown-linux-musl/release/geometrify.debug target/x86_64-unknown-linux-musl/release/geometrify

## Windows
cargo build --target=x86_64-pc-windows-gnu --release
cp target/x86_64-pc-windows-gnu/release/cli.exe target/x86_64-pc-windows-gnu/release/geometrify.exe
