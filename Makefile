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
	cargo run --bin fansland_nft_ticket -r -- --chainid 1101 > nft_ticket_1101.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 204 > nft_ticket_204.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 8453 > nft_ticket_8453.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 59144 > nft_ticket_59144.log 2>&1 &


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

start-rank:
	cargo run --bin fansland_points_rank -r > points_rank.log 2>&1 &

start-migrate:
	cargo run --bin fansland_points_migrate -r > migrate.log 2>&1 &


start-email:
	cd fansland_email_py && \
	(nohup python3 -u fansland_email.py > ../email_output.log 2>&1 &) && \
	cd ..

start-discord-bot:
	cd fansland_discord && \
	(nohup python3 -u discord_bot.py TEST_BOT_TOKEN > discord_output.log 2>&1 &) && \
	cd ..


stop:
	ps aux | grep fansland | grep -v grep | awk '{print $$2}' | xargs kill


# 删除redis的 key
# redis-cli -a gooDluck4u -n 0  KEYS "email:*" | xargs redis-cli -a gooDluck4u -n 0  DEL
# redis-cli -a gooDluck4u -n 0  KEYS "nft:*" | xargs redis-cli -a gooDluck4u -n 0  DEL


.PHONY:dep
install_dep:
	sudo apt install build-essential -y
	sudo apt install pkg-config -y
	sudo apt-get install libudev-dev libssl-dev -y
	sudo apt install python3-pip -y

