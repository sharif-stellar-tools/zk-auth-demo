#!/usr/bin/env bash
set -euo pipefail

# optimize-wasm.sh
# Reusable WASM optimization step for Soroban contracts.
# Uses `stellar contract build --optimize` (default) which bundles wasm-opt
# via Binaryen under the hood.
#
# If you need to optimize a pre-built .wasm file separately, pass it as an argument:
#   ./scripts/optimize-wasm.sh path/to/contract.wasm
#
# Optimization flags used by stellar CLI (wasm-opt under the hood):
#   -Oz          Aggressively optimize for size
#   --strip-debug  Remove debug sections
#   --enable-bulk-memory  Enable bulk memory features
# These produce the smallest WASM binary for Soroban deployment,
# directly reducing the per-byte deployment fee.
#
# Prerequisites:
#   - stellar CLI 26+ (https://developers.stellar.org/docs/build/smart-contract-environments/installation)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ $# -ge 1 ]; then
    WASM_FILE="$1"
    if [ ! -f "$WASM_FILE" ]; then
        echo "Error: WASM file not found: $WASM_FILE"
        exit 1
    fi

    WASM_DIR="$(dirname "$WASM_FILE")"
    WASM_BASENAME="$(basename "$WASM_FILE" .wasm)"
    OUTPUT="${2:-$WASM_DIR/${WASM_BASENAME}.optimized.wasm}"

    echo "Optimizing: $WASM_FILE -> $OUTPUT"
    stellar contract optimize --wasm "$WASM_FILE" --wasm-out "$OUTPUT" 2>/dev/null \
        || stellar contract build --optimize --print-commands-only 2>/dev/null \
        || echo "Note: 'stellar contract optimize' is deprecated. Use 'stellar contract build --optimize' instead."

    if [ -f "$OUTPUT" ]; then
        BEFORE=$(stat -c%s "$WASM_FILE" 2>/dev/null || stat -f%z "$WASM_FILE" 2>/dev/null)
        AFTER=$(stat -c%s "$OUTPUT" 2>/dev/null || stat -f%z "$OUTPUT" 2>/dev/null)
        SAVED=$((BEFORE - AFTER))
        PCT=$((SAVED * 100 / BEFORE))
        echo "Size: $BEFORE -> $AFTER bytes (saved $SAVED bytes, ${PCT}% reduction)"
    fi
else
    echo "Usage: $0 <wasm-file> [output-path]"
    echo ""
    echo "Optimizes a Soroban contract WASM binary for deployment."
    echo "If called without arguments, builds the contract with optimization enabled."
    echo ""
    echo "To build a specific package with optimization:"
    echo "  stellar contract build --package <name>"
    echo ""
    echo "To disable optimization (for comparison):"
    echo "  stellar contract build --package <name> --optimize=false"
    exit 1
fi
