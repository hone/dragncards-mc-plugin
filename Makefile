all: cerebro marvelcdb

cerebro: cards packs sets
cards:
	curl "https://cerebro-beta-bot.herokuapp.com/cards" | jq . > fixtures/cerebro/cards.json
packs:
	curl "https://cerebro-beta-bot.herokuapp.com/packs" | jq . > fixtures/cerebro/packs.json
sets:
	curl "https://cerebro-beta-bot.herokuapp.com/sets" | jq . > fixtures/cerebro/sets.json

marvelcdb:
	curl "https://marvelcdb.com/api/public/cards/?encounter=1" | jq . > fixtures/marvelcdb.json

lint:
	for file in $$(ls json/*.json); do \
		echo $$file; \
		cat $$file | jq . > /dev/null; \
	done
