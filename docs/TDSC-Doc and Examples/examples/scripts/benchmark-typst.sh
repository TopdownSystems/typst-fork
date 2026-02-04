#!/bin/bash
# benchmark-typst.sh - Measure Typst compilation time
#
# Usage: ./benchmark-typst.sh [input.typ] [runs]
#
# Measures wall-clock time (start to end) for compiling a Typst document.
# Defaults to sidebar0.typ with 5 runs.

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TYPST_ROOT="$(cd "$SCRIPT_DIR/../../../.." && pwd)"
DEFAULT_INPUT="$SCRIPT_DIR/../typst/sidebar0.typ"
OUTPUT="/tmp/typst-benchmark-output.pdf"

INPUT="${1:-$DEFAULT_INPUT}"
RUNS="${2:-5}"

if [[ ! -f "$INPUT" ]]; then
    echo "Error: Input file not found: $INPUT"
    exit 1
fi

# Build release version if needed
echo "Building Typst (release)..."
cd "$TYPST_ROOT"
cargo build --release -q

TYPST_BIN="$TYPST_ROOT/target/release/typst"

echo ""
echo "=== Typst Compilation Benchmark ==="
echo "Input: $INPUT"
echo "Runs: $RUNS"
echo ""

# Warm-up run (not counted)
echo "Warm-up run..."
"$TYPST_BIN" compile --no-pdf-tags "$INPUT" "$OUTPUT" 2>/dev/null

# Benchmark runs
declare -a times
total=0

echo ""
echo "Benchmark runs:"
for ((i=1; i<=RUNS; i++)); do
    # Measure wall-clock time in milliseconds
    start=$(python3 -c 'import time; print(int(time.time() * 1000))')
    "$TYPST_BIN" compile --no-pdf-tags "$INPUT" "$OUTPUT" 2>/dev/null
    end=$(python3 -c 'import time; print(int(time.time() * 1000))')

    elapsed=$((end - start))
    times+=($elapsed)
    total=$((total + elapsed))

    printf "  Run %d: %d ms\n" "$i" "$elapsed"
done

# Calculate statistics
avg=$((total / RUNS))

# Find min and max
min=${times[0]}
max=${times[0]}
for t in "${times[@]}"; do
    ((t < min)) && min=$t
    ((t > max)) && max=$t
done

echo ""
echo "=== Results ==="
echo "  Min:     $min ms"
echo "  Max:     $max ms"
echo "  Average: $avg ms"
echo ""

# Cleanup
rm -f "$OUTPUT"
