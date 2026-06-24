#!/usr/bin/env bash
set -euo pipefail

# deploy.sh
# Deploy a verifier contract with WASM optimization enabled by default.
# Usage: ./scripts/deploy.sh [--network <network>] [--source <account>] [--alias <name>]
#
# Examples:
#   ./scripts/deploy.sh
#   ./scripts/deploy.sh --network testnet --source alice --alias zk-verifier
#
# Prerequisites:
#   - stellar CLI configured with identity/account
#   - Account funded with testnet XLM (for testnet deployment)
#   - Run `stellar network add` and `stellar identity add` first if not done

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Default values
NETWORK="${STELLAR_NETWORK:-testnet}"
SOURCE="${STELLAR_SOURCE:-default}"
ALIAS="zk-verifier"
PACKAGE="zk-verifier"
BUILD_ONLY=false

# Parse args
while [[ $# -gt 0 ]]; do
    case "$1" in
        --network|-n) NETWORK="$2"; shift 2 ;;
        --source|-s) SOURCE="$2"; shift 2 ;;
        --alias) ALIAS="$2"; shift 2 ;;
        --package|-p) PACKAGE="$2"; shift 2 ;;
        --build-only) BUILD_ONLY=true; shift ;;
        --help|-h)
            echo "Usage: $0 [--network <net>] [--source <acct>] [--alias <name>] [--build-only]"
            exit 0
            ;;
        *) echo "Unknown option: $1"; exit 1 ;;
    esac
done

echo "=== Deploying $PACKAGE to $NETWORK ==="
echo "Source account: $SOURCE"
echo "Contract alias: $ALIAS"
echo ""

# Build (with optimization enabled by default)
echo "--- Building contract ---"
stellar contract build \
    --manifest-path "$PROJECT_ROOT/Cargo.toml" \
    --package "$PACKAGE" \
    --optimize 2>&1

# Find the optimized WASM
WASM_FILE=""
for f in "$PROJECT_ROOT/target/wasm32v1-none/release/"*.wasm; do
    if [ -f "$f" ]; then
        WASM_FILE="$f"
        break
    fi
done

if [ -z "$WASM_FILE" ]; then
    echo "Error: No WASM file found after build."
    exit 1
fi

echo "WASM: $WASM_FILE"
WASM_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
echo "Size: $(numfmt --to=iec "$WASM_SIZE" 2>/dev/null || echo "$WASM_SIZE bytes")"

if [ "$BUILD_ONLY" = true ]; then
    echo "Build only mode. Skipping deployment."
    exit 0
fi

# Deploy with optimization (default) and cost reporting
echo ""
echo "--- Deploying contract ---"
echo "Network: $NETWORK"
echo "Source: $SOURCE"
echo ""

CONFIG_ARGS="--network $NETWORK --source-account $SOURCE"

if [ -n "$ALIAS" ]; then
    CONFIG_ARGS="$CONFIG_ARGS --alias $ALIAS"
fi

# Deploy with cost output
stellar contract deploy \
    $CONFIG_ARGS \
    --wasm "$WASM_FILE" \
    --cost \
    --optimize 2>&1

echo ""
echo "=== Deploy complete ==="
