# Praxis
[![cargo build](https://github.com/alex0112/prxs/actions/workflows/rust.yml/badge.svg)](https://github.com/alex0112/prxs/actions/workflows/rust.yml)
> Praxis (n.) 
> 
> *The practical means by which a thing is accomplished. The opposite of theory.*

![Praxis Application Preview](https://github.com/alex0112/prxs/assets/7142972/8f9c6b83-32ed-43f8-984b-67809bd0a3fe)

`prxs` is a web application penetration testing tool, that allows users to perform common pentesting tasks from their terminal. Users will find it similar to tools such as [BurpSuite](https://portswigger.net/burp) or [MITMProxy](https://mitmproxy.org/). For our rationale and design philosophy, see [RATIONALE.md](https://github.com/alex0112/prxs/blob/main/RATIONALE.md).

### Disclaimer
You know the drill: *This tool is intended for security research purposes, always gain permission before pentesting someone else's system. The developers of praxis are not liable for your actions or any damages you may cause* Be an [ethical hacker](https://www.synopsys.com/glossary/what-is-ethical-hacking.html#B).

Happy hacking.

## Getting Started

### Installation
To install `prxs` and make it available to your system, clone this repository and build from source:
```bash
    git clone git@github.com:alex0112/prxs.git;
    cd prxs && cargo build --release && cargo install --path .
```

Alternatively:

```bash
    git clone git@github.com:alex0112/prxs.git;
    cd prxs && cargo run
```

Support for a `cargo install` from crates.io is on our roadmap.

### Usage

#### args
```bash
Usage: prxs [OPTIONS]

Options:
  -c, --config <CONFIG_PATH>       The config file to parse
  -p, --port <PORT>                The port to run on
  -s, --session [<SESSION_FILE>]   The session file to open
      --auto-gunzip <AUTO_GUNZIP>  Whether to automatically gunzip request responses [possible values: true, false]
  -h, --help                       Print help
  -V, --version                    Print version
```

#### keystrokes
See [USAGE.md](https://github.com/alex0112/prxs/blob/main/USAGE.md) for a comprehensive list of keystrokes in the TUI.

(*TL;DR* navigation is vim-like, `j`, and `k` allow navigation through the request list)

#### proxy
In order to receive requests, the user must instruct their browser or application of choice to proxy traffic to the application. We find FoxyProxy ([Firefox](https://addons.mozilla.org/en-US/firefox/addon/foxyproxy-standard/), [Chrome](https://chromewebstore.google.com/detail/foxyproxy/gcknhkkoolaabfmlnjonogaaifnjlfnp?pli=1)) to be a useful tool in this regard. Point it at `localhost:8080` (or whichever port you specify) while praxis is running and you will start to see traffic.

#### TLS decryption
The primary reason http traffic inspection is useful is to observe what requests a site or application may be making in plaintext. As it currently stands the TLS decryption portion of praxis is currently under development in the branch `feature/rustls-connects`, and with luck will be merged into `main` soon. 

Until that is working, you will see any TLS encrypted traffic begin to hit the proxy as an [`HTTP CONNECT`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods/CONNECT) request against port 443 of the target. You may forward these requests (by pressing `f`) but the response will come back with an error until the TLS decryption layer is functioning properly.

### Editing
![praxis_editor_demo](https://github.com/alex0112/prxs/assets/7142972/1dbc1579-c111-4c03-970d-e7e8ea8bb801)

As mentioned in the usage document, When focused on a specific request, a user may press `e` to open the request annotations in an editor. Praxis will default to `$EDITOR` when determining what to use, and if nothing is specified will likely open `nano`. We have seen decent results in Neovim, Emacs (both with and without the `-nw` option), and Helix. It is also possible to open a request in VSCode/Codium, but there is a known issue preventing the edited text from being read back to praxis. Your mileage may vary.

## Roadmap:
(in no particular order)

- TLS decryption
- Editing focused requests/responses
- Filtering / Scope definition
- Session storage (serialized current workflow to a file)
- Certificate generation
- Publish crate to crates.io
- [nuclei](https://github.com/projectdiscovery/nuclei) templates from existing saved requests(?)
- Tests where appropriate (time was not spent on unit tests in this iteration since most of the code is network oriented)

Additionally, [ARCH.md](https://github.com/alex0112/prxs/blob/main/ARCH.md) contains some of our thoughts about features, design decisions, and possible implementations that may or may not come into use in the actual application.

# Authors

- [June Welker](github.com/itsjunetime)
- [Alex Larsen](github.com/alex0112)
