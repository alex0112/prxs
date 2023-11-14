# Building

One builds this project just like any other rust project:
1. Install the rust toolchain from `rustup.rs`
2. Run `cargo build --release` in the root of this repo
3. The compiled binary will be placed at `./target/release/prxs`

Once run, prxs will start up a proxy server at `http://localhost:8080`. We recommend using a browser extension like [FoxyProxy](https://getfoxyproxy.org/) to facilitate the proxy connection.
