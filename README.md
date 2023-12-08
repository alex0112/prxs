
# prxs

(Pronounced Praxis)

`prxs` is a web application penetration testing tool, that allows users to perform common pentesting tasks from their terminal. Users will find it familiar to tools such as [BurpSuite](https://portswigger.net/burp) or [MITMProxy](https://mitmproxy.org/). For our rationale and design philosophy, see [RATIONALE.md][https://github.com/alex0112/prxs/blob/main/RATIONALE.md].

## Getting Started
While in its alpha release, the best way to install praxis is to clone this repository, and build from source:




## Minimum Viable Product
- A user invokes the binary
- They see a basic terminal UI that allows them to see a list of captured requests
- Highlighting a request in the UI opens it up in a separate panel with the raw HTTP content exposed
- By default (for the MVP) the app just records the response and forwards the request automatically. The initial MVP is less a MITM tool and more a HTTP traffic inspector that can be iterated upon

### Potential Crates

- [ratatui](https://crates.io/crates/ratatui) for an easily-configurable tui
- [inquire](https://github.com/mikaelmello/inquire) for interactive cli functionality
- [rcgen](https://crates.io/crates/rcgen) for generating TLS certificates to decrypt traffic with
- [tokio](https://crates.io/crates/tokio) for an async runtime
- [crossterm](https://crates.io/crates/crossterm) for cross-platform keyboard event reading
- [axum](https://crates.io/crates/axum) for simple web-server functionality to facilitate the proxying
- [reqwest](https://crates.io/crates/reqwest) for forwarding HTTP requests

## Additional features / UX Notes:

- Config for user preferences in a file (version control)
- Session (contains target info, regexes, current work, represents a complete snapshot of a current project)
- Sane defaults for common workflows
- Fast startup time
- The tool should have high discoverability (no hidden options, don't make the user read long manuals)
- The tool should provide a good ad hoc experience for viewing HTTP/TLS traffic transparently.
- Basic workflows (such as selecting a target and filtering traffic) should be easily configurable in one step
- Should run well on both Linux and MacOS

## Going Forward (Stretch Goals)
If we finish that and have time, we consider the following features stretch goals:

- The ability to drop, forward, or copy a request for later repeating
- The ability to serialize a request into a specific replayable flow
- The ability to edit a request (preferably in the editor set to `$EDITOR`)
- Domain filtering with some kind of nice regex thing (it would be cool to integrate ripgrep, fzf, or nushell queries here)
- (This one's more of a big stretch goal) The ability to generate [nuclei](https://github.com/projectdiscovery/nuclei) templates from existing saved requests to codify an attack and make it repeatable

## Usage

For keybindings and configuration, see `USAGE.md`
