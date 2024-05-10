start-api-dev:
	cargo run --bin fansland_api  -r -- --env test  --openlove-url http://18.142.5.61:50080 >> api.log 2>&1 &

start-api-test:
	cargo run --bin fansland_api  -r -- --env test  --openlove-url http://172.40.1.122:50080 >> api.log 2>&1 &

start-api-uat:
	cargo run --bin fansland_api  -r -- --env uat --openlove-url http://172.40.1.122:50080 >> api.log 2>&1 &

start-api-main:
	cargo run --bin fansland_api  -r -- --env pro --openlove-url http://172.40.1.122:50080  >> api.log 2>&1 &

#只在生产环境启动openapi
start-openapi-pro:
	cargo run --bin fansland_openapi -r > openapi.log 2>&1 &

start-nft-pro:
	cargo run --bin fansland_nft_ticket -r -- --env PRO --chainid 56 >> nft_ticket_56_pro.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --env PRO --chainid 204 >> nft_ticket_204_pro.log 2>&1 &

start-nft-uat:
	cargo run --bin fansland_nft_ticket -r -- --env UAT --chainid 56 >> nft_ticket_56_uat.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r -- --env UAT --chainid 204 >> nft_ticket_204_uat.log 2>&1 &


start-nft-test:
	cargo run --bin fansland_nft_ticket -r --  --env TEST --chainid 97 >> nft_ticket_97_test.log 2>&1 &
	cargo run --bin fansland_nft_ticket -r --  --env TEST --chainid 5611 >> nft_ticket_5611_test.log 2>&1 &

start-airdrop-pro:
	cargo run --bin fansland_nft_airdrop -r -- --env pro --chainid 204 >> nft_airdrop_204_pro.log 2>&1 &

start-airdrop-uat:
	cargo run --bin fansland_nft_airdrop -r -- --env uat --chainid 204 >> nft_airdrop_204_uat.log 2>&1 &

start-airdrop-test:
	cargo run --bin fansland_nft_airdrop -r -- --env test --chainid 5611 >> nft_airdrop_5611.log 2>&1 &

start-migrate-test:
	cargo run --bin fansland_points_migrate -r -- --env TEST >> migrate_test.log 2>&1 &

start-migrate-uat:
	cargo run --bin fansland_points_migrate -r -- --env UAT >> migrate_uat.log 2>&1 &

start-migrate-pro:
	cargo run --bin fansland_points_migrate -r -- --env PRO >> migrate_pro.log 2>&1 &


start-email:
	cd fansland_email_py && \
	(nohup python3 -u fansland_email.py >> ../email_output.log 2>&1 &) && \
	cd ..

start-discord-bot:
	cd fansland_discord && \
	(nohup python3 -u fansland_discord_bot.py TEST_BOT_TOKEN >> ../discord_bot_output.log 2>&1 &) && \
	cd ..

start-telegram-test-bot:
	cd fansland_telegram && \
	(nohup python3 -u fansland_telegram_bot.py TEST_CHATID TEST_TG_BOT_TOKEN >> ../telegram_bot_output.log 2>&1 &) && \
	cd ..

start-telegram-pro-bot:
	cd fansland_telegram && \
	(nohup python3 -u fansland_telegram_bot.py PRO_CHATID PRO_TG_BOT_TOKEN  >> ../telegram_bot_output.log 2>&1 &) && \
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

