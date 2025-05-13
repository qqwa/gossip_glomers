

TARGET = target/debug/gossip_glomers
$(TARGET): $(wildcard src/**/*.rs) Cargo.toml
	cargo build

echo: $(TARGET)
	./maelstrom test -w echo --bin $(TARGET) --node-count 1 --time-limit 10

unique-ids: $(TARGET)
	./maelstrom test -w unique-ids --bin $(TARGET) --time-limit 30 --rate 1000 --node-count 3 --availability total --nemesis partition

broadcast-single: $(TARGET)
	./maelstrom test -w broadcast --bin $(TARGET) --time-limit 20 --rate 10 --node-count 1

broadcast-multi: $(TARGET)
	./maelstrom test -w broadcast --bin $(TARGET) --time-limit 20 --rate 10 --node-count 5

broadcast-faulty: $(TARGET)
	./maelstrom test -w broadcast --bin $(TARGET) --time-limit 20 --rate 10 --node-count 5 --node-count 1 --nemesis partition

serve:
	./maelstrom serve
