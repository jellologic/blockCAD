#!/bin/bash
# Cross-validation test suite: blockCAD kernel vs trimesh (independent mesh library)
#
# Usage: ./tests/cross-validation/run.sh

set -e

echo "=== Step 1: Generate STL fixtures from blockCAD kernel ==="
cd "$(dirname "$0")/../../packages/kernel"
cargo test --test export_fixtures -- --nocapture 2>&1 | grep -E "ok|FAILED|Wrote"

echo ""
echo "=== Step 2: Validate with trimesh (independent Python library) ==="
cd "$(dirname "$0")"
pip3 install -q -r requirements.txt 2>/dev/null
python3 -m pytest -v .

echo ""
echo "=== Cross-validation complete ==="
