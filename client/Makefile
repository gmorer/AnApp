OUT_DIR := static

web: web-debug
web-debug:
	cargo build -Zfeatures=itarget --target wasm32-unknown-unknown
	rm -rf $(OUT_DIR)
	mkdir -p $(OUT_DIR)
	wasm-bindgen target/wasm32-unknown-unknown/debug/client.wasm --no-typescript --out-dir $(OUT_DIR) --web
	cp index.html $(OUT_DIR)/index.html

web-release:
	cargo build --release --target wasm32-unknown-unknown
	rm -rf $(OUT_DIR)
	mkdir -p $(OUT_DIR)
	wasm-bindgen target/wasm32-unknown-unknown/release/client.wasm --out-dir $(OUT_DIR) --target web


web-run: web-debug
	cd static && python3 -m http.server
