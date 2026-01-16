SHELL := /bin/bash

# =========================
# Config
# =========================
SERVICE ?= fulfilment-api
SERVICE_PORT ?= 8080

COMPOSE_DIR := ops/compose
COMPOSE_FILE := $(COMPOSE_DIR)/docker-compose.yml
COMPOSE := docker compose -f $(COMPOSE_FILE)

LOG_TAIL ?= 200

# =========================
# Help
# =========================
.PHONY: help
help:
	@echo "Shipyard Make targets:"
	@echo ""
	@echo "Dev (local cargo):"
	@echo "  make dev           - run the $(SERVICE) service locally"
	@echo "  make build         - build the workspace"
	@echo "  make test          - run tests for the workspace"
	@echo "  make fmt           - format code (cargo fmt)"
	@echo "  make lint          - lint code (cargo clippy, denies warnings)"
	@echo "  make check         - fmt + lint + test"
	@echo "  make smoke         - smoke checks (service must be running)"
	@echo "  make ci            - alias for check (used by CI)"
	@echo ""
	@echo "Runtime (docker compose):"
	@echo "  make up            - start runtime stack (build + up -d)"
	@echo "  make down          - stop runtime stack"
	@echo "  make restart       - restart runtime stack"
	@echo "  make restart-full  - restart runtime stack (down + up)"
	@echo "  make logs          - tail all runtime logs"
	@echo "  make logs-service  - tail one service logs (SVC=jaeger|otelcol|prometheus|fulfilment-api)"
	@echo ""

# =========================
# Dev (local cargo)
# =========================
.PHONY: dev build test fmt lint check smoke ci
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
	@chmod +x scripts/smoke.sh
	@SERVICE_PORT=$(SERVICE_PORT) scripts/smoke.sh

ci: check

# =========================
# Runtime (docker compose)
# =========================
.PHONY: up down restart logs logs-service

up:
	$(COMPOSE) up -d --build

down:
	$(COMPOSE) down

restart:
	$(COMPOSE) restart

restart-full: down up

logs:
	$(COMPOSE) logs -f --tail=$(LOG_TAIL)

logs-service:
	@test -n "$(SVC)" || (echo "Usage: make logs-service SVC=<service>"; exit 1)
	$(COMPOSE) logs -f --tail=$(LOG_TAIL) $(SVC)