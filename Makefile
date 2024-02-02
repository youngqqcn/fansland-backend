api:
	cargo run --bin fansland_api

start-api:
	cargo run --bin fansland_api -r > api.log 2>&1 &

start-nft:
	cargo run --bin fansland_nft_ticket -r -- --chainid 80001 > nft_ticket_80001.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 97 > nft_ticket_97.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 11155111 > nft_ticket_11155111.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 421614 > nft_ticket_421614.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 11155420 > nft_ticket_11155420.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 43113 > nft_ticket_43113.log 2>&1 &

start-email:
	cd fansland_email_py && \
	(nohup python3 -u main.py > ../email_output.log 2>&1 &) && \
	cd ..

stop:
	ps aux | grep fansland | grep -v grep | awk '{print $$2}' | xargs kill

