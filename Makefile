SHELL := /bin/bash

SERVICE ?= fulfilment-api
SERVICE_PORT ?= 8080

.PHONY: help dev build test fmt lint check smoke ci

help:
	@echo "Shipyard (dev) Make targets:"
	@echo "  make dev        - run the $(SERVICE) service locally"
	@echo "  make build      - build the workspace"
	@echo "  make test       - run tests for the workspace"
	@echo "  make fmt        - format code (cargo fmt)"
	@echo "  make lint       - lint code (cargo clippy, denies warnings)"
	@echo "  make check      - fmt + lint + test"
	@echo "  make smoke      - curl health endpoints (service must be running)"
	@echo "  make ci         - alias for check (used by CI)"

dev:
	cargo run -p $(SERVICE)

build:
	cargo build --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

check: fmt lint test

smoke:
	@echo "Smoke test: /healthz"
	curl -i http://localhost:$(SERVICE_PORT)/healthz
	@echo ""
	@echo ""
	@echo "Smoke test: /readyz"
	curl -i http://localhost:$(SERVICE_PORT)/readyz
	@echo ""
	@echo ""

ci: check