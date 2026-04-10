.PHONY: help fetch build check fmt lint clean db-up db-down migrate migrate-down sqlx-prepare test install-tools init db-reset

help:
	@echo "Comandi disponibili:"
	@echo "  make install-tools  - Installa sqlx-cli (richiesto una volta sola)"
	@echo "  make init           - Avvia il DB, applica le migration e prepara sqlx cache"
	@echo "  make db-reset       - Cancella volumi e dati, poi reinizializza da zero"
	@echo "  make fetch          - Compila ed esegue il fetching del feed XML"
	@echo "  make build          - Build ottimizzato (release)"
	@echo "  make check          - Controlla errori senza compilare"
	@echo "  make fmt            - Formatta il codice con rustfmt"
	@echo "  make lint           - Linting con clippy"
	@echo "  make clean          - Rimuove la cartella target/"
	@echo "  make db-up          - Avvia il container PostgreSQL"
	@echo "  make db-down        - Ferma e rimuove il container PostgreSQL"
	@echo "  make migrate        - Applica le migration (up)"
	@echo "  make migrate-down   - Reverte l'ultima migration (down)"
	@echo "  make sqlx-prepare   - Genera .sqlx/ per build offline (richiede DB attivo)"
	@echo "  make test           - Esegue i test (testcontainers avvia Postgres automaticamente)"

install-tools:
	cargo install sqlx-cli --no-default-features --features rustls,postgres

init: db-up
	@echo "Attendo che Postgres sia pronto..."
	@until docker compose exec postgres pg_isready -U "$$(grep POSTGRES_USER .env | cut -d= -f2-)" -d "$$(grep POSTGRES_DB .env | cut -d= -f2-)" > /dev/null 2>&1; do sleep 1; done
	@echo "Postgres pronto."
	$(MAKE) migrate

db-reset:
	docker compose down -v
	$(MAKE) init

fetch:
	cargo run -- fetch

build:
	cargo build --release

check:
	cargo check

fmt:
	cargo fmt

lint:
	cargo clippy

clean:
	cargo clean

db-up:
	docker compose up -d postgres

db-down:
	docker compose down

migrate:
	cargo sqlx migrate run --database-url "$$(grep DATABASE_URL .env | cut -d= -f2-)"

migrate-down:
	cargo sqlx migrate revert --database-url "$$(grep DATABASE_URL .env | cut -d= -f2-)"

sqlx-prepare:
	cargo sqlx prepare -- --all-targets

test:
	SQLX_OFFLINE=true RUST_LOG=info,testcontainers=debug cargo test -- --nocapture

