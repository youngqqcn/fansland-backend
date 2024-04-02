start-api-test:
	cargo run --bin fansland_api  -r -- --env test > api.log 2>&1 &

start-api-uat:
	cargo run --bin fansland_api  -r -- --env uat > api.log 2>&1 &

start-api-main:
	cargo run --bin fansland_api  -r -- --env pro > api.log 2>&1 &

start-openapi:
	cargo run --bin fansland_api  -r  > openapi.log 2>&1 &

start-nft-main:
	cargo run --bin fansland_nft_ticket -r -- --chainid 137 > nft_ticket_137.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 56 > nft_ticket_56.log 2>&1 &


start-nft-test:
	cargo run --bin fansland_nft_ticket -r -- --chainid 80001 > nft_ticket_80001.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --chainid 97 > nft_ticket_97.log 2>&1 &

start-airdrop-pro:
	cargo run --bin fansland_nft_airdrop -r -- --env pro --chainid 137 > nft_airdrop_137_pro.log 2>&1 &

start-airdrop-uat:
	cargo run --bin fansland_nft_airdrop -r -- --env uat --chainid 137 > nft_airdrop_137_uat.log 2>&1 &


start-airdrop-test:
	cargo run --bin fansland_nft_airdrop -r -- --env test --chainid 80001 > nft_airdrop_80001.log 2>&1 &

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
	(nohup python3 -u fansland_discord_bot.py TEST_BOT_TOKEN > ../discord_bot_output.log 2>&1 &) && \
	cd ..

start-telegram-test-bot:
	cd fansland_telegram && \
	(nohup python3 -u fansland_telegram_bot.py TEST_CHATID TEST_TG_BOT_TOKEN > ../telegram_bot_output.log 2>&1 &) && \
	cd ..

start-telegram-pro-bot:
	cd fansland_telegram && \
	(nohup python3 -u fansland_telegram_bot.py PRO_CHATID PRO_TG_BOT_TOKEN  > ../telegram_bot_output.log 2>&1 &) && \
	cd ..


stop:
	ps aux | grep fansland | grep -v grep | grep -v fansland_web |  awk '{print $$2}' | xargs kill


# 删除redis的 key
# redis-cli -a gooDluck4u -n 0  KEYS "email:*" | xargs redis-cli -a gooDluck4u -n 0  DEL
# redis-cli -a gooDluck4u -n 0  KEYS "nft:*" | xargs redis-cli -a gooDluck4u -n 0  DEL


.PHONY:dep
install_dep:
	sudo apt install build-essential -y
	sudo apt install pkg-config -y
	sudo apt-get install libudev-dev libssl-dev -y
	sudo apt install python3-pip -y

