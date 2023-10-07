# prxs

(Pronounced Praxis)

`prxs` is a web application penetration testing tool, that allows users to perform common pentesting tasks from their terminal.

## Existing Tools
In the web application security space it's common to set up a proxy server and self-sign a cert locally so that a pentester can intercept and edit HTTP requests as they go through a browser. There are several applications dedicated to doing this such as:

- [BurpSuite](https://portswigger.net/burp)
- [OWASP ZAP](https://www.zaproxy.org/)
- [MITMProxy](https://mitmproxy.org/)

(not exhaustive)

The tools listed above are useful and quite popular in the pentesting community. But we'd prefer to have an HTTP proxy program that is completely open-source, command-line oriented, and is able to be integrated with other powerful tools.

#### Problems with existing tools
Burpsuite is closed source and proprietary. Certain features are locked behind a paywall, and in order to use the application a user must operate entirely within the ecosystem that PortSwigger provides. The application works well and has many useful features, but ultimately restricts the user to working within its own ecosystem. Additionally many common workflows involve navigating to multiple tabs, jumping around, hovering over nested submenus, and other time consuming or tedious workflows.

OWASP ZAP is one of the original proxy programs used for web application pentesting. Like BurpSuite it requires a lock in to a UI system and must be run as a desktop application. Unlike Burpsuite it is completely open source and does not limit its users with a requirement to pay for additional features. While the open source nature of the project is praiseworthy, the project is difficult to install on MacOS (due to some Apple security restrictions) and it operates on similar flows to BurpSuite.

MITMProxy is likely the closest to the program we'd like to build: It runs in the terminal, is open source, and has importable libraries that allow it to be fully scriptable. Where we depart from it philosophically is that we would like to be able to choose our own tools to integrate with it rather than being locked into its workflows. Without experience it can be difficult to set up, and its feature set is relatively limited compared to other tools. Being a TUI app, it is possible to work from the terminal, but we'd like a more powerful solution.

## Our design philosophy

Most of the tools listed above have an extension system of some kind that allows adding to the program functionality. But we'd prefer something more in line with the original UNIX philosophy. [The Jargon File](http://www.catb.org/~esr/writings/taoup/html/ch01s06.html) quotes Doug McIlroy as saying:

> Make each program do one thing well. To do a new job, build afresh rather than complicate old programs by adding new features.

While extensions are a neat and powerful tool, they ultimately create ecosystem lock rather than allowing users to define workflows with whatever tools they choose. Sometimes the trade off is worth it, but due to the ad hoc, exploratory nature of pentesting we believe this is not the case for this problem set. A powerful part of the UNIX philosophy is to treat each program as something that will inherently live as part of a pipeline, and has a common text interface. We would like our tool to support this approach while providing powerful defaults that are easy to pipe to other programs of the users choosing.

Rather than going through an options menu to edit every request, we'd like to be able to use `$EDITOR` to directly modify requests. We'd like to save requests/responses as serialized files and pipe them to other programs. Essentially, what we'd like to create eventually is a UNIX version of MITMProxy, and eventually integrate it with tools such as ripgrep, fuzzy-finder, your code editor of choice, and the ability to write complex (and real-time) queries in a language like nushell. We would also like the end user to have the freedom of serializing requests into formats that can be piped to other arbitrary programs (such as [nuclei](https://github.com/projectdiscovery/nuclei)).

References:

- [The UNIX philosophy](http://www.catb.org/~esr/writings/taoup/html/ch01s08.html)
  - Esp. **"A tool should do one thing and do it well"**
- [Designing Good CLI tools](https://clig.dev/)


## Minimum Viable Product

- A user invokes the binary
- They see a basic terminal UI that allows them to see a list of captured requests
- Highlighting a request in the UI opens it up in a separate panel with the raw HTTP content exposed
- By default (for the MVP) the app just records the response and forwards the request automatically. The initial MVP is less a MITM tool and more a HTTP traffic inspector that can be iterated upon

### Potential Crates

- [ratatui](https://crates.io/crates/ratatui) for an easily-configurable tui
- [rcgen](https://crates.io/crates/rcgen) for generating TLS certificates to decrypt traffic with
- [tokio](https://crates.io/crates/tokio) for an async runtime
- [crossterm](https://crates.io/crates/crossterm) for cross-platform keyboard event reading
- [axum](https://crates.io/crates/axum) for simple web-server functionality to facilitate the proxying
- [reqwest](https://crates.io/crates/reqwest) for forwarding HTTP requests

## Possible Architectures:

1. Completely pipe-oriented. A suite of commands that allow for piping to one another
2. Solely TUI app: Just runs in a TUI, basically a terminal version of Burpsuite / A MITMProxy clone
3. A mix of 1 & 2, where the TUI allows for ad-hoc work, but custom commands can be developed or included with the installation, and the user can invoke a keystroke within the TUI to allow a selection or an entire request to be piped to a command, and the output to be reconsumed by the TUI
4. Use sockets: an input socket, and an output socket. The user invokes the TUI (which also spins up the proxy), and in a separate terminal commands they wish to use can read from the custom socket to get output

Additional thoughts on option 4 can be found in the file [ARCH.md](./ARCH.md)

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
