#!/bin/bash
set -e

echo "=== Running Integration Test Verification ==="

# Check if we have network access (skip if no network)
if ! ping -c 1 8.8.8.8 > /dev/null 2>&1; then
    echo "⚠️  No network access detected, skipping integration test"
    echo "✅ Skipping integration test (offline mode)"
    exit 0
fi

echo "Testing: cargo run --example cisco_conn ..."

# Run the integration test
cargo run --example cisco_conn 192.168.0.130 ayourtch cisco123 "show version" > /tmp/integration_output.txt 2>&1

# Check exit code
if [ $? -eq 0 ]; then
    echo "✅ Integration test PASSED"
    
    # Verify output contains expected content
    if grep -q "Cisco IOS Software" /tmp/integration_output.txt && \
       grep -q "AY-LIVING#" /tmp/integration_output.txt; then
        echo "✅ Output verification PASSED"
        exit 0
    else
        echo "❌ Output verification FAILED"
        cat /tmp/integration_output.txt
        exit 1
    fi
else
    echo "❌ Integration test FAILED"
    cat /tmp/integration_output.txt
    exit 1
fi