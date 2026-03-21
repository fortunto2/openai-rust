#!/bin/bash
# Sync OpenAI OpenAPI spec and check for drift.
# Run periodically or in CI: ./scripts/sync-spec.sh
set -e

SPEC_URL="https://raw.githubusercontent.com/openai/openai-openapi/master/openapi.yaml"
SPEC_FILE="tests/openapi.yaml"
PYTHON_SDK="$HOME/startups/shared/openai-python"

echo "=== OpenAI Spec Sync ==="

# 1. Download latest spec
echo "Downloading latest spec..."
curl -sL "$SPEC_URL" -o /tmp/openapi-latest.yaml

# 2. Diff with current
if diff -q "$SPEC_FILE" /tmp/openapi-latest.yaml > /dev/null 2>&1; then
    echo "Spec is up to date."
else
    ADDED=$(diff "$SPEC_FILE" /tmp/openapi-latest.yaml | grep "^>" | wc -l | tr -d ' ')
    REMOVED=$(diff "$SPEC_FILE" /tmp/openapi-latest.yaml | grep "^<" | wc -l | tr -d ' ')
    echo "Spec changed: +$ADDED -$REMOVED lines"

    # Show new endpoints/schemas
    echo ""
    echo "New schemas:"
    diff "$SPEC_FILE" /tmp/openapi-latest.yaml | grep "^>" | grep -E "^\s+\w+:" | head -20

    echo ""
    echo "Run: cp /tmp/openapi-latest.yaml $SPEC_FILE"
    echo "Then: cargo test --test openapi_coverage -- --nocapture"
fi

# 3. Check Python SDK version
if [ -d "$PYTHON_SDK" ]; then
    PYTHON_VER=$(grep 'version' "$PYTHON_SDK/pyproject.toml" 2>/dev/null | head -1 | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "unknown")
    echo ""
    echo "Python SDK: v$PYTHON_VER"
    echo "Update: cd $PYTHON_SDK && git pull"
fi

# 4. Run coverage test
echo ""
echo "Running coverage test..."
cargo test --test openapi_coverage -- --nocapture 2>&1 | grep -E "(coverage|FAIL|OK|%)"

echo ""
echo "=== Done ==="
