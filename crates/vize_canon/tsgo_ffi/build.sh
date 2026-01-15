#!/bin/bash
# Build tsgo FFI shared library

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Detect OS
OS=$(uname -s)
case "$OS" in
    Darwin)
        OUTPUT="libtsgo_ffi.dylib"
        ;;
    Linux)
        OUTPUT="libtsgo_ffi.so"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

echo "Building $OUTPUT..."

# Build shared library
CGO_ENABLED=1 go build -buildmode=c-shared -o "$OUTPUT" main.go

echo "Built: $SCRIPT_DIR/$OUTPUT"

# Generate header file is automatic with c-shared mode
# The header file will be named libtsgo_ffi.h
