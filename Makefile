build: js
	cargo build --release

watch: js
	cargo run

js:
	browserify scripts/index.js -o static/index.js
	browserify scripts/alignment.js -o static/alignment.js
