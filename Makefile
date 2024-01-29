api:
	cargo run --bin fansland_api

start:
	cargo run --bin fansland_api -r > api.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 80001 > nft_ticket.log 2>&1 &