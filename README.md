# Snake (Rust/Wasm)

The old game we all know and love.

Use the arrow keys to begin, reset, and move in the game.

The purpose of this project was to learn the basics of Rust/Wasm and see how it could be integrated
into a JavaScript project without writing all the code in Rust.

## Instructions
1.  Install [Rust](https://rustup.rs/)
2.  Install [wasm-pack](https://rustwasm.github.io/wasm-pack/installer/)
3.  Run `wasm-pack build --release --target web` from the command line.
4.  Run a localhost server from the directoty with the index.html file in it, eg.
    * Python: `python -m http.server`
    * Node/NPM: `npx http-server`
