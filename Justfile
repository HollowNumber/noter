# DTU Notes - Development Justfile
# Run `just --list` to see all available commands

# Variables
binary_name := "noter"
release_dir := "target/release"
debug_dir := "target/debug"

# Default recipe - shows available commands
default:
    @just --list

# Build the project in debug mode
build:
    cargo build

# Build the project in release mode
build-release:
    cargo build --release

# Run the project with arguments
run *ARGS:
    cargo run -- {{ARGS}}

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture

# Run clippy for linting
lint:
    cargo clippy -- -D warnings

# Format code
fmt:
    cargo fmt

# Check formatting without applying changes
check-fmt:
    cargo fmt -- --check

# Run all checks (format, lint, test)
check: check-fmt lint test

# Clean build artifacts
clean:
    cargo clean

# Install the binary locally
install:
    cargo install --path .

# Show project information
info:
    @echo "DTU Notes CLI Tool"
    @echo "=================="
    @echo "Binary: {{binary_name}}"
    @echo "Version: $(cargo pkgid | cut -d# -f2)"
    @echo "Debug binary: {{debug_dir}}/{{binary_name}}.exe"
    @echo "Release binary: {{release_dir}}/{{binary_name}}.exe"

# Generate documentation
docs:
    cargo doc --open

# Watch for changes and rebuild
watch:
    cargo watch -x check

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Profile the release build
profile:
    cargo build --release
    @echo "Binary size: $(ls -lh {{release_dir}}/{{binary_name}}.exe | awk '{print $5}')"

# Run with specific subcommands for testing
test-setup:
    cargo run -- setup

test-status:
    cargo run -- status

test-courses:
    cargo run -- courses list

test-config:
    cargo run -- config show

# Development helpers
dev-all: fmt lint test build
    @echo "All development checks passed!"

# Release workflow
release: check build-release profile
    @echo "Release build complete!"
    @echo "Binary location: {{release_dir}}/{{binary_name}}.exe"

# Quick development cycle
dev: build test
    @echo "Quick development cycle complete!"

# Setup development environment
setup-dev:
    cargo install cargo-watch
    cargo install cargo-edit
    @echo "Development environment setup complete!"

# Generate coverage report (requires cargo-tarpaulin)
coverage:
    cargo tarpaulin --out Html

# Security audit
audit:
    cargo audit

# Update dependencies
update:
    cargo update

# Show dependency tree
deps:
    cargo tree

# Benchmark (if benchmarks exist)
bench:
    cargo bench

# Create a new tag and push
tag VERSION:
    git tag -a v{{VERSION}} -m "Release version {{VERSION}}"
    git push origin v{{VERSION}}

# Package for distribution
package:
    cargo package --allow-dirty
