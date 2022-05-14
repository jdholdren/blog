run: clean
	cargo run

clean:
	rm -rf ./generated/*

browse:
	open ./generated/index.html
