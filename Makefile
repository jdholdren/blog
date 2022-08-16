run: clean
	cargo run

clean:
	rm -rf ./generated
	rm -f db.sqlite

browse:
	open ./generated/index.html
