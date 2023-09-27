# prxs
Pronounced Praxis

## Minimal Feature Set

- A user invokes the binary
- They see a basic terminal UI that allows them to see a list of captured requests
- Highlighting a request in the UI opens it up in a separate panel with the raw HTTP content exposed
- By default (for the MVP) the app just records the response and forwards the request automatically. So I guess the initial app is less a MITM tool and more a HTTP traffic inspector

## Going Forward
If we finish that and have time, consider the following features stretch goals:

- The ability to drop, forward, or copy a request for later repeating
- The ability to edit a request (I would love it if the tool were able to open the request in a panel or something (not sure if that's possible) using $EDITOR and letting the user edit it in the TUI)
- Domain filtering with some kind of nice regex thing (it would be cool to integrate ripgrep, fzf, or nushell queries here)
- (This one's more of a big stretch goal) The ability to generate [nuclei](https://github.com/projectdiscovery/nuclei) templates from existing saved requests to codify an attack and make it repeatable
