.PHONY: setup
# Setup development environment
setup:
	rustup component add rustfmt clippy
	cargo fetch

.PHONY: clean
# Cleanup compilation outputs
clean:
	cargo clean

.PHONY: fmt-check fmt
# Check the code format
fmt-check:
	cargo fmt --all -- --check
# Format the code
fmt:
	cargo fmt --all

.PHONY: clippy clippy-release
# Run rust clippy with debug profile
clippy:
	cargo clippy --all --all-targets -- -D warnings
# Run rust clippy with release profile
clippy-release:
	cargo clippy --release --all --all-targets -- -D warnings

.PHONY: check check-release
# Check code with debug profile
check:
	cargo check
# Check code with release profile
check-release:
	cargo check --release

.PHONY: build build-release
# Build all binaries with debug profile
build:
	cargo build
# Build all binaries with release profile
build-release:
	cargo build --release

.PHONY: test test-release
# Run all unit tests with debug profile
test:
	cargo test --all
# Run all unit tests with release profile
test-release:
	cargo test --release --all

.PHONY: install
# Install the binary locally
install:
	cargo install --path .

.PHONY: lint
# Run skilo lint on test fixtures
lint:
	cargo run -- lint .

.PHONY: new-skill
# Create a new test skill (usage: make new-skill NAME=my-skill LANG=python)
new-skill:
	cargo run -- new $(NAME) --lang $(or $(LANG),python)

.PHONY: ci
# Run all CI checks (fmt, clippy, test, build, lint)
ci: fmt clippy test build lint

.PHONY: doc
# Generate documentation
doc:
	cargo doc --no-deps --open

.PHONY: help
# Show help
help:
	@echo ''
	@echo 'Usage:'
	@echo ' make [target]'
	@echo ''
	@echo 'Targets:'
	@awk '/^[a-zA-Z\-\_0-9]+:/ { \
	helpMessage = match(lastLine, /^# (.*)/); \
		if (helpMessage) { \
			helpCommand = substr($$1, 0, index($$1, ":")); \
			helpMessage = substr(lastLine, RSTART + 2, RLENGTH); \
			printf "\033[36m%-30s\033[0m %s\n", helpCommand,helpMessage; \
		} \
	} \
	{ lastLine = $$0 }' $(MAKEFILE_LIST)

.DEFAULT_GOAL := help
