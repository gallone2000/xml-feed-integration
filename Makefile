.PHONY: help fetch build check fmt lint clean

help:
	@echo "Comandi disponibili:"
	@echo "  make fetch   - Compila ed esegue il fetching del feed XML"
	@echo "  make build   - Build ottimizzato (release)"
	@echo "  make check   - Controlla errori senza compilare"
	@echo "  make fmt     - Formatta il codice con rustfmt"
	@echo "  make lint    - Linting con clippy"
	@echo "  make clean   - Rimuove la cartella target/"

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
