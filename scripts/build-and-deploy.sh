#!/usr/bin/env bash
set -euo pipefail

# build-and-deploy.sh
# Single utility: build -> optimize -> deploy verifier contract.
# Optimization is included by default (not opt-in).
# Usage: ./scripts/build-and-deploy.sh [deploy-options...]
#
# Examples:
#   ./scripts/build-and-deploy.sh
#   ./scripts/build-and-deploy.sh --network testnet --source alice --alias zk-verifier
#   ./scripts/build-and-deploy.sh --build-only  (build + optimize, skip deploy)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "╔══════════════════════════════════════════════════════════╗"
echo "║  ZK Verifier Contract — Build → Optimize → Deploy      ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Step 1: Build with optimization (enabled by default)
echo ">>> [1/3] Building contract (with WASM optimization)..."
"$SCRIPT_DIR/build.sh" "$@"

# Find the optimized WASM
WASM_FILE=""
for f in "$PROJECT_ROOT/target/wasm32v1-none/release/"*.wasm; do
    if [ -f "$f" ]; then
        WASM_FILE="$f"
        break
    fi
done

if [ -z "$WASM_FILE" ]; then
    echo "Error: Build did not produce a WASM file."
    exit 1
fi

# Step 2: Standalone optimization report (reusable step)
echo ""
echo ">>> [2/3] Applying WASM optimization..."

OPTIMIZED_DIR="$PROJECT_ROOT/target/optimized"
mkdir -p "$OPTIMIZED_DIR"
OPTIMIZED_FILE="$OPTIMIZED_DIR/$(basename "$WASM_FILE")"

if command -v wasm-opt &>/dev/null; then
    wasm-opt -Oz --strip-debug "$WASM_FILE" -o "$OPTIMIZED_FILE"
    cp "$OPTIMIZED_FILE" "$WASM_FILE"
    echo "Applied wasm-opt -Oz optimization"
elif stellar contract optimize --wasm "$WASM_FILE" --wasm-out "$WASM_FILE" 2>/dev/null; then
    echo "Applied stellar contract optimize"
else
    echo "Optimization already applied during build step (stellar contract build --optimize)."
fi

FINAL_SIZE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
echo "Final WASM size: $(numfmt --to=iec "$FINAL_SIZE" 2>/dev/null || echo "$FINAL_SIZE bytes")"

# Step 3: Deploy (unless --build-only was passed)
if [[ "$*" != *"--build-only"* ]]; then
    echo ""
    echo ">>> [3/3] Deploying contract..."
    "$SCRIPT_DIR/deploy.sh" "$@"
else
    echo ""
    echo ">>> [3/3] Skipped (--build-only)."
    echo ""
    echo "WASM ready for deployment: $WASM_FILE"
fi

echo ""
echo "=== Done ==="
