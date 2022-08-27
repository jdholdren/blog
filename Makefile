generated:
	cargo run --release

clean:
	rm -rf ./generated
.PHONY: generated

browse:
	open ./generated/index.html
.PHONY: browse
