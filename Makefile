
.PHONY: serve echo unique-ids broadcast-single broadcast-multi broadcast-faulty

TARGET_BASE = target/debug
TARGET_ = $(TARGET_BASE)/gossip_glomers
TARGET_ECHO = $(TARGET_BASE)/echo
TARGET_UNIQUE = $(TARGET_BASE)/unique_ids
TARGET_BROADCAST_SIMPLE = $(TARGET_BASE)/broadcast_simple

$(TARGET_): $(wildcard src/**/*.rs) Cargo.toml
	cargo build

echo: $(TARGET_)
	./maelstrom test -w echo --bin $(TARGET_ECHO) --node-count 1 --time-limit 10

unique-ids: $(TARGET_)
	./maelstrom test -w unique-ids --bin $(TARGET_UNIQUE) --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast-single: $(TARGET_)
	./maelstrom test -w broadcast --bin $(TARGET_BROADCAST_SIMPLE) --time-limit 20 --rate 10 --node-count 1

broadcast-multi: $(TARGET_)
	./maelstrom test -w broadcast --bin $(TARGET_BROADCAST_SIMPLE) --time-limit 20 --rate 10 --node-count 5

broadcast-faulty: $(TARGET_)
	./maelstrom test -w broadcast --bin $(TARGET_) --time-limit 20 --rate 10 --node-count 5 --nemesis partition

serve:
	./maelstrom serve

