# prxs
Pronounced Praxis

In the web application security space it's common to set up a proxy server and self-sign a cert locally so that a pentester can intercept and edit HTTP requests as they go through a browser. There are several applications dedicated to doing this such as:

- [BurpSuite](https://portswigger.net/burp)
- [OWASP ZAP](https://www.zaproxy.org/)
- [MITMProxy](https://mitmproxy.org/)


## Our design philosopy
The tools listed above are useful and quite popular in the pentesting community. But we'd prefer to have an HTTP proxy program that is completely open-source, command-line oriented, and is able to be integrated with other powerful tools.

Burpsuite is closed source and proprietary. Certain features are locked behind a paywall, and in order to use the application a user must operate entirely within the ecosystem that PortSwigger provides. The application works, but it's difficult to extend using modern tools and locks itself into a certain ecosystem.

OWASP ZAP is one of the original proxy programs. Like BurpSuite it requires a lock in to a UI system and must be run as a desktop application. It's open source, but

MITMProxy is likely the closest to the program we'd like to build: It runs in the terminal, is open source, and has importable libraries that allow it to be fully scriptable. Where we depart from it philosophically is that we would like to be able to choose our own tools to integrate with it. Rather than going through an options menu to edit every request, we'd like to be able to use `$EDITOR` to directly modify requests. We'd like to save "flows" as serialized files and pipe those flows to other programs. Essentially, what we'd like to create eventually is a UNIX version of MITMProxy, and eventually integrate it with tools such as ripgrep, fuzzy-finder, your code editor of choice, and the ability to write complex (and real-time) queries in a language like nushell. We would also like to create the capability of serializing requests into formats that can be piped to other programs such as [nuclei](https://github.com/projectdiscovery/nuclei)

Most of the tools listed above have an extension system of some kind that allows adding to the program functionality. But we'd prefer something more in line with the original UNIX philosophy. The Jargon File quotes Doug McIlroy as saying:

> Make each program do one thing well. To do a new job, build afresh rather than complicate old programs by adding new features.

While extensions are a neat and powerful tool, they ultimately create ecosystem lock rather than allowing users to define workflows with whatever tools they choose. A powerful part of the unix philosophy is to treat each program as something that will inherently live as part of a pipeline.

References:

- [The UNIX philosopy](http://www.catb.org/~esr/writings/taoup/html/ch01s06.html)
  - Esp. **"A tool should do one thing and do it well"**
- [Designing Good CLI tools](https://clig.dev/)
- The tool should have high discoverability (no hidden options, don't make the user read long manuals)
- The tool should provide a good ad hoc experience for viewing HTTP/TLS traffic transparently.

## Minimal Feature Set
- A user invokes the binary
- They see a basic terminal UI that allows them to see a list of captured requests
- Highlighting a request in the UI opens it up in a separate panel with the raw HTTP content exposed
- By default (for the MVP) the app just records the response and forwards the request automatically. The initial MVP is less a MITM tool and more a HTTP traffic inspector

## Going Forward
If we finish that and have time, consider the following features stretch goals:

- The ability to drop, forward, or copy a request for later repeating
- The ability to serialize a request into a specific replayable flow
- The ability to edit a request (I would love it if the tool were able to open the request in a panel or something (not sure if that's possible) using $EDITOR and letting the user edit it in the TUI)
- Domain filtering with some kind of nice regex thing (it would be cool to integrate ripgrep, fzf, or nushell queries here)
- (This one's more of a big stretch goal) The ability to generate [nuclei](https://github.com/projectdiscovery/nuclei) templates from existing saved requests to codify an attack and make it repeatable
