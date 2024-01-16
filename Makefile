api:
	cargo run --bin fansland_api

diesel-setup:
	diesel setup

diesel-migration-run:
	diesel migration run

diesel-migration-redo:
	diesel migration redo


diesel-model:
	cd fansland_api && diesel_ext