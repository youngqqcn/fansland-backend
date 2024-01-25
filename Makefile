api:
	cargo run --bin fansland_api

start:
	cargo run --bin fansland_api -r > api.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r > nft_ticket.log 2>&1 &