.PHONY: all build dev frontend clean run fmt lint

# Default target: build frontend + Rust release binary
all: frontend build

# Build release binary (requires frontend to be built first)
build: frontend
	cargo build --release

# Build debug binary (faster compilation, no optimizations)
dev: frontend
	cargo build

# Build frontend assets (Vue 3 + xterm.js)
frontend:
	cd frontend && npm run build

# Clean Rust build artifacts + frontend dist
clean:
	cargo clean
	if exist frontend\dist ( rmdir /s /q frontend\dist )

# Run the server (debug build)
run:
	cargo run

# Format Rust code
fmt:
	cargo fmt

# Run Rust linter
lint:
	cargo clippy
