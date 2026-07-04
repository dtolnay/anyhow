#!/bin/bash
# Run the error_cost benchmark under three backtrace configurations.
# Results are stored in target/criterion/ with HTML reports.
set -e

BENCH="cargo bench --bench error_cost --"

echo "============================================================"
echo "  Round 1: Backtrace DISABLED (RUST_LIB_BACKTRACE=0)"
echo "  This measures pure heap allocation cost."
echo "============================================================"
RUST_LIB_BACKTRACE=0 $BENCH --output-format bencher

echo ""
echo "============================================================"
echo "  Round 2: Backtrace ENABLED, default release (DWARF CFI)"
echo "  This is the expensive case without frame pointers."
echo "============================================================"
RUST_LIB_BACKTRACE=1 $BENCH --output-format bencher

echo ""
echo "============================================================"
echo "  Round 3: Backtrace ENABLED + Frame Pointers"
echo "  Faster unwinding via RBP chain."
echo "============================================================"
RUST_LIB_BACKTRACE=1 RUSTFLAGS="-C force-frame-pointers=yes" \
    cargo bench --bench error_cost -- --output-format bencher

echo ""
echo "============================================================"
echo "  Done. HTML reports at: target/criterion/report/index.html"
echo "============================================================"
