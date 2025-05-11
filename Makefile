

TARGET = target/debug/gossip_glomers
$(TARGET): $(wildcard src/**/*.rs) Cargo.toml
	cargo build

echo: $(TARGET)
	./maelstrom test -w echo --bin $(TARGET) --node-count 1 --time-limit 10

serve:
	./maelstrom serve
