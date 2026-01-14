# justfile for mvln
set shell := ["bash", "-c"]

# Default recipe
default: pre-commit

# ---------- Main Entry Points ----------

# Run all checks before commit
pre-commit:
    just fmt
    just clippy
    just test

# CI checks (format check, not auto-fix)
ci:
    just fmt-check
    just clippy
    just test

# ---------- Internal Steps ----------

# Format all code and stage modified .rs files
fmt:
    cargo fmt --all
    git diff --name-only | grep '\.rs$' | xargs -r git add

# Check formatting without modifying (CI-friendly)
fmt-check:
    cargo fmt --all -- --check

# Run clippy
clippy:
    cargo clippy --all-features -- -D warnings

# Run all tests
test:
    cargo test --all-features

# ---------- Parameterized Commands ----------

# Run tests for a specific pattern
test-filter pattern:
    cargo test --all-features -- {{pattern}}

# ---------- Git Hooks ----------

# Install git hooks
hooks-install:
    @echo "Installing git hooks..."
    @mkdir -p .git/hooks
    @echo '#!/bin/bash' > .git/hooks/pre-commit
    @echo 'exec ./scripts/pre-commit.sh' >> .git/hooks/pre-commit
    @chmod +x .git/hooks/pre-commit
    @chmod +x scripts/pre-commit.sh
    @echo "✓ Pre-commit hook installed"

# Uninstall git hooks
hooks-uninstall:
    @rm -f .git/hooks/pre-commit
    @echo "✓ Pre-commit hook removed"
