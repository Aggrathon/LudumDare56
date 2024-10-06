# Instructions for building WASM

Taken from: https://github.com/bevyengine/bevy/blob/latest/examples/README.md#wasm

1. Install target: `rustup target install wasm32-unknown-unknown`
3. Install bindgen: `cargo install wasm-bindgen-cli`
4. Build for WASM: `cargo build --profile wasm-release --target wasm32-unknown-unknown`
5. Generate bindings: `wasm-bindgen --out-dir ./target/html --target web .\target\wasm32-unknown-unknown\wasm-release\ludum_dare_56.wasm`

Remember to update the names in the file `html/index.html`.

# Deploy to GitHub Pages

1. Do the above 
2. Switch to the `gh-pages` branch (created with `git switch --orphan gh-pages`)
3. Get the index: `git checkout master -- html/index.html; cp html/index.html .; rm html/index.html`
3. Get the assets: `git checkout master -- assets .gitignore`
5. Get the WASM files: `wasm-bindgen --out-dir . --target web .\target\wasm32-unknown-unknown\wasm-release\ludum_dare_56.wasm`
5. Test locally: `python -m http.server`
6. Commit and push
