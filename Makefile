# Makefile for grow-rust project

# Default target
.PHONY: all
all: test

# Build the project
.PHONY: build
build:
	cargo build

# Run tests
.PHONY: test
test:
	cargo test

# Run the test binary
.PHONY: run
run:
	cargo run --bin test-grow

# Run as root (for GPIO access on Raspberry Pi)
.PHONY: sudo-run
sudo-run:
	sudo cargo run --bin test-grow

# Clean the build artifacts
.PHONY: clean
clean:
	cargo clean
