#!/usr/bin/env bash
# scripts/pre-commit.sh - Pre-commit hook script
# This file is tracked in version control.
set -e

echo "Running pre-commit checks..."
just pre-commit

echo "All checks passed!"
