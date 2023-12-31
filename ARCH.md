## Potential Architectures for `prxs`

### TUI Proxy server, Publish/Subscribe model from the command line.

#### User Flow
0. The user wishes to configure their machine for a MITM Attack
- They run `prxs --init` which will launch an interactive command line experience that walks them through generating and self-signing a cert. (It would be nice to consider a flow using [this library](https://github.com/mikaelmello/inquire)
- `prxs --init` will also create a file in `~/.config/prxs/prxs.conf` that stores persistent data such as user preferences, default editor (if they wish to override \$EDITOR) etc. This file is an excellent candidate for entering into the users personal version control
- It also creates a `.prxs.d/` directory in the home directory, where data such as sessions or anything the application needs to write to disk can be serialized and saved.

1. A user wishes to begin proxying/decrypting traffic. They spin up `prxs`, which launches a TUI showing them all incoming traffic
- This creates a new session file (separate from the config file)
- Bonus feature: The session file can be encrypted if they so desire. The file could be password protected or require a GPG key to decrypt etc.

2. At this point the user wishes to set the scope of their target and reduce the signal to noise ratio, they can:
  - Define an ad-hoc command that will apply filtering of some kind (possibly by hitting `|` and defining the command in the TUI, which then sends it to their command line)
  - Point the program at a domains file or a custom file that defines a target scope
  - Prompts them to paste a URL, and the program makes an intelligent guess about the scope
    - The default guess could be: the domain, any subdomains, and any additional requests those domains initiate to third party sources
  - Some combination of the above
  - Once the user is satisfied with the scope setting, they can save the scope as a consumable channel (nomenclature note: the channel could be called the outbox, it could be a custom named channel, it could be called a subscription, we'll work on it)

3. Now the user has a stream of HTTP Traffic narrowed in on their scope. They wish to:
   - Identify interesting requests related to their test and save them for further repeating/modification
   - Refine the scope related to testing a specific feature
     - They can essentially define a sub scope that inherits the filtering provided by the target selection, and then identify a set of interesting requests
     - This allows them to essentially build an arsenal of requests that they can name, play back, modify, script etc.
   - Map over all requests in an async fashion and apply a user defined function (e.g. to refresh a session token or JWT if it expires)


4. Now the user has a specific set of requests that they know are useful or interesting, they wish to copy, modify and define custom intercept rules for those requests:
- Annotate requests with variables that need to be filtered by, selected, modified etc.
  - A very very very minimalistic annotation DSL/markup lang would be nice here.
    - Something as simple as `%varname%` for a named variable that is pattern matched against, that could be selected against in a nushell pipe.
    - `%varname | <user defined function>%` could select a variable by pattern matching, and pipe that variable to a function, substituting its output for the variable space
    - `%["a", "list", "of", "values"]%` for a list of substitutions that should be made. (Any annotated request like this should actually be treated as a list of requests)
    -   It would also be good to have a way of defining temporary data like a expirable token, with a function that can produce a fresh value like: `%token: "eyJhbGciOiJIUzI...", exp: <a function which can determine if the token is expired>, ref: <a function which performs a token refresh and returns it>%`

5. Channels
At any point the user should be able to publish a target or subscope as a channel which can be read via command line in another context. In the CLI they can subscribe to the channel and consume it, piping each request received by that channel to programs and pipelines of their choosing. If at the end of the pipeline they wish to send a request, they can pipe it to a program like HTTPie, or HTTPx. And the output of our files should be designed with those use cases in mind. We may wish to include a custom formatter that can be piped to.

- In the TUI, the user should be able to select a current context (such as the entire target scope, or a subscope) and choose to publish it to a named outbox.
- We will write an additional tool that handles a channel subscription (let's say for now it's called `xsub` for "praxis subscribe") which take in the channel name and allows the building of a pipeline from it like:

    ```
    xsub <channel name> | sed 's/p/P/g' | { grep -oP 'set-cookie: \K[^"]+' >> cookies.txt; cat; } | some-other-program
    ```

- The above pipeline (which I totally tested and didn't just ask GPT3 to generate for me) capitalizes every 'p' character it finds, then looks for any set cookie headers and writes them to a file called "cookies.txt", then pipes the original request (with capital P's now) to a program named `some-other-program`.
- A user should additionally be able to publish modified requests *back* to the TUI. Imagine a command called `xpub`:

    ```
    xsub <channel name> | sed 's/p/P/g' | { grep -oP 'set-cookie: \K[^"]+' >> cookies.txt; cat; } | some-other-program | xpub <channel name>
    ```

- Then the user (In the TUI) could select packets that have been modified and published back to the inbox of the channel and modify, replay, save etc. to their heart's content.

6. Saving workflows
- At any point in the testing process a user should be able to define a workflow. A workflow might have steps like:
  - Log in
  - Make request A
  - Make Request B, select a piece of data
  - Generate request C, where the generation of C relies of the piece of data from request A
  - Pipe the output of the response of request C to a program and make some determination about it (e.g. if an exploit worked)
- Workflows should be completely serializable in a static file. [nuclei templates](https://docs.nuclei.sh/template-example/http/base-http) would be a powerful way to save a workflow and make it completely reproducible for further automation.
- Workflows should also be saved as a static file (of some kind), so that if a user leaves mid-session they never lose any of their current progress in a test.

7. Request/Replay functionality
The general philosophy of `prxs` should be to make a distinction between the exploratory, ad-hoc nature of penetration testing in the initial reconnaissance phase, and the saved reproducible nature of an exploit. Hacking in praxis should be an almost playful experience where the tester pushes an application in ways the original designers did not expect, and whenever they find an interesting result they are able to immediately save it and iterate on the attack in a useful reproducible way.

- The main view of the application should be a panel with an intercepted request on one side, and the response of the request on the other.
- The user should be able to specify the layout of the TUI in their config file.
- Bonus feature: color code and syntax highlight various requests/responses.
- The user should be able to highlight and select (in a keyboard driven way) portions of the request or response, and pipe the selected text to another tool.
  - e.g. `<selected text> | base64 --decode`
- This should be the main view of the TUI application and the first thing the user sees. (Similar to the repeater tab in BurpSuite)

#### Implementation Notes
- A good way to implement the channel functionality might be through one of the following:
    - [UNIX Domain Sockets](https://en.wikipedia.org/wiki/Unix_domain_socket)
    - plain [network sockets](https://en.wikipedia.org/wiki/Network_socket) (if the UNIX Domain Sockets prove untenable)
    - [Named pipes](https://linuxiac.com/how-to-use-pipes-and-named-pipes-in-linux-explained-with-examples/) ([example](https://askubuntu.com/questions/449132/why-use-a-named-pipe-instead-of-a-file))
        - Rust library for named pipes [here](https://docs.rs/unix-named-pipe/latest/unix_named_pipe/)

- As for the real time, async nature of the commands, our `xsub` and `xpub` commands should be able to operate as a continuous pipeline and continue running as long as the channel remains open. Implementing that might be one of the trickier parts of this project, but it may be possible to treat the socket as a stream and when the stream is published to the TUI portion of the program can provide a separator so that `xsub` knows how to break up the output of the stream into individual packets.
    - There is a tool built into the GNU Coreutils called `stdbuf` which can control buffering from real time streams. [This question](https://unix.stackexchange.com/questions/200235/how-to-use-sed-to-manipulate-continuously-streaming-output) on stackexchange deals with using it and another command in conjunction with `sed` to deal with streams of text. It may prove useful in building the tool. It's possible that the use of that tool or a similar approach may help solve this problem.
      - It may actually be more useful to use [unbuffer](https://manpages.debian.org/stretch/expect/unbuffer.1.en.html) but that's not a GNU coreutil. Leaving here for reference.

- Defining the storage of information to the disk:
  - The top level storage should be a session, this might be a combination of targets
  - A target is a top level scope definition, a list of optional subscopes, and defined workflows.
  - So:
      - `session` has many `targets` (but probably usually just one)
      - `target` has many `subscopes`
      - `target` has many `workflows`

  - The information could be stored in several ways:
    - Static JSON or YAML (nice because it's editable)
    - A `.sqlite` file (very lightweight and fast, with the drawback of not being user editable as plain text)

- The user can define a specific subset of requests (through a filter of domains, or just specifying one specific request, or all up until a specific time, whatever) to be opened in a new tab for processing of a specific type AND/OR passing all request which they have specified to a specific pipe, which can then be subscribed to outside of praxis and the requests can be processed as the user sees fit with their own tools

- We should provide a command that works like `eval "$(prxs --subscribe --channel=5 --commands=zsh)"` to create a list of commands in the current shell that could be used to interact with a specific channel in easy and simple ways
	- `prxs --subscribe --channel=5 --commands=zsh` would output a bunch of text which would define a bunch of commands that can be used in the shell specified, to interact with request being processed (`subscribe`) by the channel specified, and that would then be `eval`d to make the commands accessible to the user
	- It would also perhaps be nice to just make `prxs --subscribe --channel=5` output all request being piped through channel 5 so that they don't need to source some functions to interact

## Internal Message Passing

- most labels are just a message wrapped by the channel being used to send it
	- e.g. `proxy_tx(response)`
- if its `obj.call` then it's just a function call to facilitate a send
	- e.g. `res.send_interaction(RequestInteraction)`
- there are numbers to more clearly show the flow of things. they're slightly counter-intuitive (there are four 4's 'cause they happen effectively at the same time), but (imo) most clearly represent what's happening

```mermaid
flowchart LR
	proxy(["Proxy"])
	proxy_server(["Proxy Server"])
	events(["Events"])
	app(["App"])
	response_waiter(["Response Waiter"])
	request(["Request"])

	%% the main loop
	proxy_server-- "proxy_server(Result<(), hyper::Error>)" -->app
	proxy-- "1. proxy_rx(hyper::Request)" -->app
	events-- "2. event_handler(crossterm::Event)" -->app
	response_waiter-- "6. response_waiter(RequestResponse)" -->app

	%% when an interaction happens
	app-- "3. req.send_interaction(RequestInteraction)" -->request
	request-- "4. interaction_tx(RequestInteraction)" -->proxy
	app-- "4. response_waiter.submit(Future<Output = RequestResponse>)" -->response_waiter
	proxy-- "5. response_tx(hyper::Response)" -->response_waiter

	%% when we receive a response
	app-- "7. req.store_response(RequestResponse)" -->request
```

## UI Design
Each tab will have like  30% of the left-hand side taken up by a notes area, with like `:h for help` on the bototm? And the other 70% on the right-hand taken up by the request above the response

Help can just be a big long list of data? but maybe we could make it automatically jump to the help specific to this section (e.g. help on the main view would jump to help#main, and help on a tab would jump to help#tab-specific-inputs or whatever)

main view:
```
----------------------------------------------------------
| * Main | Request 1 | Hacking Request |                 |
----------------------------------------------------------
| * GET google.com          | Request data...            |
|   PUT site.com            |                            |
|                           |                            |
|                           |                           w|
|                           |----------------------------|
|                           | Response data...           |
|                           |                            |
|                           |                            |
|                           |                            |
|                          a|                           s|
----------------------------------------------------------
:help
```

other tab:
```
----------------------------------------------------------
| Main | Request 1 | * Hacking Request |                 |
----------------------------------------------------------
| site is probably | Request data...                     |
| vulnerable to i- |                                     |
| njection         |                                     |
|                  |                                    w|
|                  |-------------------------------------|
|                  | Response data...                    |
|                  |                                     |
|                  |                                     |
|                  |                                     |
|:h for help      a|                                    s|
----------------------------------------------------------
:help
```
