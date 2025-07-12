#!/bin/bash
set -e

# Navigate to the workspace directory
cd /mnt/persist/workspace

echo "Checking if our test file exists..."
ls -la tests/

echo "Checking content of our test file..."
head -20 tests/builtin_functions_test.rs