SHELL := /bin/bash

# =========================
# Config
# =========================
SERVICE ?= fulfilment-api
SERVICE_BIN ?= fulfilment-api
SERVICE_PORT ?= 8080

COMPOSE_DIR := ops/compose
COMPOSE_FILE := $(COMPOSE_DIR)/docker-compose.yml 
COMPOSE := docker compose -f $(COMPOSE_FILE)

DB_SVC ?= postgres
DATABASE_URL ?= postgres://shipyard:shipyard@localhost:5432/shipyard

LOG_TAIL ?= 200

ENV_FILE ?= .env

define LOAD_ENV
set -a; [ -f $(ENV_FILE) ] && . $(ENV_FILE); set +a;
endef

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
	@echo "  make fmt-check     - check code format (cargo fmt --check)"
	@echo "  make lint          - lint code (cargo clippy, denies warnings)"
	@echo "  make check         - fmt + lint + test"
	@echo "  make smoke         - smoke checks (service must be running)"
	@echo "  make env-check     - print key env vars as seen by Make"
	@echo ""
	@echo "Runtime (docker compose):"
	@echo "  make up            - start runtime stack (build + up -d)"
	@echo "  make down          - stop runtime stack"
	@echo "  make restart       - restart runtime stack"
	@echo "  make logs          - tail all runtime logs"
	@echo "  make logs-service  - tail one service logs (SVC=jaeger|otelcol|prometheus|fulfilment-api)"
	@echo ""
	@echo "DB (workflow):"
	@echo "  make dev-db-up     - start Postgres only (compose profile db)"
	@echo "  make dev-db-down   - stop Postgres only (compose profile db)"
	@echo "  make migrate       - apply $(SERVICE) migrations"
	@echo "  make test-db       - run DB integration tests (auto starts/stops Postgres)"
	@echo "  make db-logs       - tail Postgres logs"
	@echo ""

# =========================
# Dev (local cargo)
# =========================
.PHONY: dev build test fmt fmt-check lint check smoke env-check
dev:
	@bash -lc '$(LOAD_ENV) \
	if [ -z "$${DATABASE_URL:-}" ]; then \
		echo "ERROR: DATABASE_URL not set. Put it in $(ENV_FILE) (recommended) or export it."; \
		exit 1; \
	fi; \
	cargo run -p $(SERVICE) --bin $(SERVICE_BIN)'

build:
	cargo build --workspace

test:
	cargo test --workspace

fmt:
	cargo fmt --all

fmt-check:
	cargo fmt --all --check

lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

check: fmt-check lint test

smoke:
	@chmod +x scripts/smoke.sh
	@SERVICE_PORT=$(SERVICE_PORT) scripts/smoke.sh

env-check:
	@bash -lc '$(LOAD_ENV) env | grep -E "^DATABASE_URL$|^SERVICE_PORT$"'

# =========================
# Runtime (docker compose)
# =========================
.PHONY: up down restart logs logs-service
up:
	$(COMPOSE) up -d --build

down:
	$(COMPOSE) down

restart: down up

logs:
	$(COMPOSE) logs -f --tail=$(LOG_TAIL)

logs-service:
	@test -n "$(SVC)" || (echo "Usage: make logs-service SVC=<service>"; exit 1)
	$(COMPOSE) logs -f --tail=$(LOG_TAIL) $(SVC)

# =========================
# DB (workflow)
# =========================
.PHONY: dev-db-up dev-db-wait dev-db-down db-logs migrate test-db

dev-db-up:
	$(COMPOSE)  up -d $(DB_SVC)
	@$(MAKE) dev-db-wait
	@$(MAKE) migrate

dev-db-wait:
	@echo "Waiting for Postgres healthcheck..."
	@cid="$$( $(COMPOSE)  ps -q $(DB_SVC) )"; \
	until [ "$$(docker inspect -f '{{.State.Health.Status}}' $$cid 2>/dev/null)" = "healthy" ]; do \
		sleep 1; \
	done; \
	echo "Postgres is healthy."

dev-db-down:
	$(COMPOSE) down -v --remove-orphans >/dev/null 2>&1 || true

db-logs:
	$(COMPOSE)  logs -f --tail=$(LOG_TAIL) $(DB_SVC)

migrate:
	@bash -lc '$(LOAD_ENV) \
		DATABASE_URL=$${DATABASE_URL:-$(DATABASE_URL)} \
		cargo run -p $(SERVICE) --bin migrate'

test-db:
	@set -e; \
	$(MAKE) dev-db-up; \
	trap '$(MAKE) dev-db-down' EXIT; \
	bash -lc '$(LOAD_ENV) \
		DATABASE_URL=$${DATABASE_URL:-$(DATABASE_URL)} \
		cargo test -p $(SERVICE) db_ -- --ignored'