use crate::{
    config::Session,
    event::EventHandler,
    input_state::InputCommand,
    layout::{LayoutState, PaneSelector},
    request::RequestInteraction,
    response_waiter::{CopyableResponse, RequestResponse, ResponseWaiter},
    tui::Tui,
    ProxyMessage,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use futures_util::stream::StreamExt;
use std::{error, fmt::Debug, future::Future, io, pin::Pin};
use tokio::{select, sync::mpsc::UnboundedReceiver};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// The thread which handles key/mouse events asynchronously
    pub event_handler: EventHandler,

    /// The "Server" which may or may not return an error at some point, so we need to keep
    /// watching it
    pub proxy_server: Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>>,

    /// The receiver for events sent from the proxy
    pub proxy_rx: UnboundedReceiver<ProxyMessage>,

    /// The struct that handles waiting for request responses in the main event loop
    pub response_waiter: ResponseWaiter,

    /// The current session state
    pub session: Session,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(
        event_handler: EventHandler,
        proxy_server: Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>>,
        proxy_rx: UnboundedReceiver<ProxyMessage>,
        session: Session,
    ) -> Self {
        Self {
            event_handler,
            proxy_server,
            proxy_rx,
            response_waiter: ResponseWaiter::default(),
            session,
        }
    }

    fn quit(tui: &mut Tui, exit_code: i32) -> ! {
        // we're quitting. so what if we return an error.
        tui.exit().unwrap();
        std::process::exit(exit_code);
    }

    pub async fn run(&mut self, mut tui: Tui, mut layout: LayoutState) {
        loop {
            tui.draw(&mut layout).expect("Couldn't draw tui");

            // this will just keep going until you kill the app, basically
            select! {
                ev = self.event_handler.next() => {
                    self.handle_event(ev, &mut tui, &mut layout).await;
                }
                res = &mut self.proxy_server => {
                    println!("Got err from server: {res:?}");
                    Self::quit(&mut tui, 1);
                }
                req = self.proxy_rx.recv() => {
                    if let Some((req, sender)) = req {
                        layout.add_request(req, sender);
                    }
                }
                resp = &mut self.response_waiter.next() => {
                    if let Some(resp) = resp {
                        layout.handle_req_response(resp);
                    }
                }
            }
        }
    }

    async fn handle_event(
        &mut self,
        event: io::Result<Event>,
        tui: &mut Tui,
        layout: &mut LayoutState,
    ) {
        let ev = match event {
            Ok(ev) => ev,
            Err(e) => {
                println!("Agh! Everything has broken! {e}");
                Self::quit(tui, 1);
            }
        };

        if let Event::Key(key) = ev {
            // Just reset it once they start typing again
            layout.err_msg = None;

            if layout.input.selected {
                if let Some(cmd) = layout.input.route_keycode(key.code) {
                    self.handle_input_command(cmd, tui, layout);
                }
                return;
            }

            match key.code {
                // quit the app
                KeyCode::Esc | KeyCode::Char('q') => Self::quit(tui, 0),
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => Self::quit(tui, 0),
                // go down the list of requests
                KeyCode::Down | KeyCode::Char('j') => layout.scroll_down(),
                // go up the list of requests
                KeyCode::Up | KeyCode::Char('k') => layout.scroll_up(),
                // forward the currently-selected request
                KeyCode::Char('f' | 'F') => {
                    if let Some(req) = layout.current_req_mut() {
                        if let Some(rx) = req.send_interaction(RequestInteraction::Forward) {
                            let id = req.id;
                            self.response_waiter.submit(Box::pin(async move {
                                let response = match rx
                                    .await
                                    .expect("uhhh I don't know how to handle a None here")
                                {
                                    Ok(resp) => Ok(CopyableResponse::from_resp(resp).await),
                                    Err(e) => Err(e),
                                };

                                RequestResponse { id, response }
                            }));
                        }
                    }
                }
                // drop the currently-selected request
                KeyCode::Char('d' | 'D') => {
                    if let Some(req) = layout.current_req_mut() {
                        _ = req.send_interaction(RequestInteraction::Drop);
                    }
                }
                // start inputting a command
                KeyCode::Char(c @ ('i' | ':')) => {
                    layout.input.selected = true;
                    if c == ':' {
                        // to add the `:` that you'd expect
                        layout.input.route_keycode(key.code);
                    }
                }
                KeyCode::Char(c @ ('m' | 'w' | 'a' | 's')) => {
                    layout.select_pane_with_input(PaneSelector::Key(c));
                }
                // send the currently-selected request to a new tab
                KeyCode::Char('p' | 'P') => layout.separate_current_req().await,
                _ => {}
            }
        }
    }

    fn handle_input_command(&mut self, cmd: InputCommand, tui: &mut Tui, state: &mut LayoutState) {
        match cmd {
            // TODO: Handle errors here
            InputCommand::SaveSession(path) => self.session.save(path).unwrap(),
            InputCommand::Quit => Self::quit(tui, 0),
            InputCommand::SelectTab(idx) => state.select_pane_with_input(PaneSelector::Idx(idx)),
            InputCommand::GunzipCurrent => state.try_gunzip_current(),
        }
    }
}

#[allow(clippy::missing_fields_in_debug)]
impl Debug for App {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        // We can't print any information about `proxy_server` cause it's a future that's pending
        // until the app exits
        fmt.debug_struct("App")
            .field("event_handler", &self.event_handler)
            .field("proxy_rx", &self.proxy_rx)
            .field("response_waiter", &self.response_waiter)
            .field("session", &self.session)
            .finish()
    }
}
