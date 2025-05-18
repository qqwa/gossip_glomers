
.PHONY: echo unique-ids broadcast-single broadcast-multi broadcast-faulty

TARGET_BASE = target/debug
TARGET_ = $(TARGET_BASE)/gossip_glomers
TARGET_ECHO = $(TARGET_BASE)/echo

$(TARGET_ECHO): $(wildcard src/**/*.rs) Cargo.toml
	cargo build

echo: $(TARGET_ECHO)
	# ./maelstrom test -w echo --bin $(TARGET_ECHO) --node-count 1 --time-limit 10

unique-ids: $(TARGET_ECHO)
	./maelstrom test -w unique-ids --bin $(TARGET_) --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast-single: $(TARGET_ECHO)
	./maelstrom test -w broadcast --bin $(TARGET_) --time-limit 20 --rate 10 --node-count 1

broadcast-multi: $(TARGET_ECHO)
	./maelstrom test -w broadcast --bin $(TARGET_) --time-limit 20 --rate 10 --node-count 5

broadcast-faulty: $(TARGET_ECHO)
	./maelstrom test -w broadcast --bin $(TARGET_) --time-limit 20 --rate 10 --node-count 5 --nemesis partition

serve:
	./maelstrom serve
