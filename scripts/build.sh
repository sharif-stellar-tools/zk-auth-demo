#!/usr/bin/env bash
set -euo pipefail

# build.sh
# Build verifier contract(s) with WASM optimization enabled by default.
# Reports WASM sizes before and after optimization.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PACKAGE="${1:-zk-verifier}"

echo "=== Building contract package: $PACKAGE ==="

# Step 1: Build unoptimized for comparison
echo ""
echo "--- Building unoptimized (for size comparison) ---"
stellar contract build \
    --manifest-path "$PROJECT_ROOT/Cargo.toml" \
    --package "$PACKAGE" \
    --optimize=false 2>&1

UNOPTIMIZED_WASM=""
for f in "$PROJECT_ROOT/target/wasm32v1-none/release/"*.wasm; do
    if [ -f "$f" ]; then
        UNOPTIMIZED_WASM="$f"
        UNOPT_SIZE=$(stat -c%s "$f" 2>/dev/null || stat -f%z "$f" 2>/dev/null)
        break
    fi
done

# Step 2: Build with optimization (default)
echo ""
echo "--- Building optimized ---"
stellar contract build \
    --manifest-path "$PROJECT_ROOT/Cargo.toml" \
    --package "$PACKAGE" \
    --optimize 2>&1

OPTIMIZED_WASM=""
for f in "$PROJECT_ROOT/target/wasm32v1-none/release/"*.wasm; do
    if [ -f "$f" ]; then
        OPTIMIZED_WASM="$f"
        OPT_SIZE=$(stat -c%s "$f" 2>/dev/null || stat -f%z "$f" 2>/dev/null)
        break
    fi
done

# Report
echo ""
echo "=== Size Report ==="
if [ -n "$UNOPTIMIZED_WASM" ] && [ -n "$OPTIMIZED_WASM" ]; then
    echo "Unoptimized: $(numfmt --to=iec "$UNOPT_SIZE" 2>/dev/null || echo "$UNOPT_SIZE bytes")"
    echo "Optimized:   $(numfmt --to=iec "$OPT_SIZE" 2>/dev/null || echo "$OPT_SIZE bytes")"
    SAVED=$((UNOPT_SIZE - OPT_SIZE))
    PCT=$((SAVED * 100 / UNOPT_SIZE))
    echo "Reduction:   $SAVED bytes ($PCT%)"
elif [ -n "$OPTIMIZED_WASM" ]; then
    echo "Optimized WASM: $OPTIMIZED_WASM"
    echo "Size: $(numfmt --to=iec "$OPT_SIZE" 2>/dev/null || echo "$OPT_SIZE bytes")"
else
    echo "No WASM files found in target directory."
    echo "Checking alternative locations..."
    find "$PROJECT_ROOT/target" -name "*.wasm" 2>/dev/null
fi
