api:
	cargo run --bin api

diesel-setup:
	diesel setup

diesel-migration-run:
	diesel migration run

diesel-migration-redo:
	diesel migration redo