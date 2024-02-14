api:
	cargo run --bin fansland_api

start-api:
	cargo run --bin fansland_api -r > api.log 2>&1 &

start-nft-main:
	cargo run --bin fansland_nft_ticket -r -- --chainid 137 > nft_ticket_137.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 56 > nft_ticket_56.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 42161 > nft_ticket_42161.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 10 > nft_ticket_10.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 43114 > nft_ticket_43114.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 1 > nft_ticket_1.log 2>&1 &

start-nft-test:
	cargo run --bin fansland_nft_ticket -r -- --chainid 80001 > nft_ticket_80001.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 97 > nft_ticket_97.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 11155111 > nft_ticket_11155111.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 421614 > nft_ticket_421614.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 11155420 > nft_ticket_11155420.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 43113 > nft_ticket_43113.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 1442 > nft_ticket_1442.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 5611 > nft_ticket_5611.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 84532 > nft_ticket_84532.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 59140 > nft_ticket_59140.log 2>&1 &

start-email:
	cd fansland_email_py && \
	(nohup python3 -u fansland_email.py > ../email_output.log 2>&1 &) && \
	cd ..

stop:
	ps aux | grep fansland | grep -v grep | awk '{print $$2}' | xargs kill


# 删除redis的 key
# redis-cli -a gooDluck4u -n 0  KEYS "email:*" | xargs redis-cli -a gooDluck4u -n 0  DEL
# redis-cli -a gooDluck4u -n 0  KEYS "nft:*" | xargs redis-cli -a gooDluck4u -n 0  DEL