# Colors for better visibility in output
GREEN := \033[0;32m
BOLD := \033[1m
RESET := \033[0m

# Setup variables.
POSTGRES_IMAGE=pgvector/pgvector:pg17

# Default values.
DB_URL=postgres://postgres:password@localhost:5432/postgres

.PHONY: all
all:
	@echo "Makefile for Local Development"
	@echo ""
	@echo "$(BOLD)Usage$(RESET): make <target>"
	@echo ""
	@echo "$(BOLD)Targets$(RESET):"
	@echo " • pull-postgres: Pull Postgres image from Docker Hub"
	@echo " • run-postgres: Run Postgres Docker container"
	@echo " • stop-postgres: Stop and remove Postgres container"
	@echo " • setup-dev: Set up environment for local development"
	@echo " • teardown-dev: Tear down local development environment"

.PHONY: pull-postgres
pull-postgres:
	@echo "Pulling Postgres image from Docker Hub..."
	@docker pull $(POSTGRES_IMAGE)
	@docker tag $(POSTGRES_IMAGE) linnear-postgres:latest
	@echo "$(GREEN)Postgres image pulled successfully as:$(RESET)"
	@echo "linnear-postgres:latest"

.PHONY: run-postgres
run-postgres:
	@echo "Running Postgres Docker container..."
	@docker run -d --name linnear-postgres \
	-e POSTGRES_PASSWORD=password \
	-p 5432:5432 linnear-postgres:latest

	@echo "$(GREEN)Postgres Docker container is running on:$(RESET)"
	@echo "$(DB_URL)"

.PHONY: stop-postgres
stop-postgres:
	@echo "Stopping Postgres Docker container..."
	@docker stop linnear-postgres
	@docker rm linnear-postgres
	@echo "$(GREEN)Postgres Docker container stopped and removed.$(RESET)"

.PHONY: setup-dev
setup-dev:
	@echo "Setting up environment for local development..."
	@$(MAKE) pull-postgres
	@$(MAKE) run-postgres

	@touch .env
	@echo "DB_URL=$(DB_URL)" > .env
	@echo "OPENAI_API_KEY=xxx" >> .env

	@cargo run migrate
	@echo "$(GREEN)Environment setup complete:$(RESET)"
	@echo "Please provide the .env file with the correct values."

.PHONY: teardown-dev
teardown-dev:
	@echo "Tearing down environment for local development..."
	@$(MAKE) stop-postgres
	@echo "$(GREEN)Environment teardown complete.$(RESET)"
