#!/bin/sh
set -eu
cd "$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
if command -v cargo >/dev/null 2>&1; then
    exec cargo run --release "$@"
elif [ -x "$HOME/.cargo/bin/cargo" ]; then
    PATH="$HOME/.cargo/bin:$PATH"
    export PATH
    exec cargo run --release "$@"
else
    printf '%s\n' 'Rust is required. Install it from https://rustup.rs/ and run this script again.' >&2
    exit 1
fi
