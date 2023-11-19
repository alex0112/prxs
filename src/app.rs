use crate::{
    event::EventHandler,
    request::{ProxyInteraction, Request, RequestInteraction},
    response_waiter::{RequestResponse, ResponseWaiter},
    tui::Tui,
    ProxyMessage,
};
use crossterm::event::{Event, KeyCode, KeyModifiers};
use futures_util::stream::StreamExt;
use std::{error, fmt::Debug, future::Future, io, pin::Pin};
use tokio::{
    select,
    sync::mpsc::{Sender, UnboundedReceiver},
};
use uuid::Uuid;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

/// Application.
pub struct App {
    /// Track what the current highlighted item in the list is.
    pub current_request_index: usize,

    /// Temporary list of requests
    pub requests: Vec<Request>,

    /// The the thread which handles key/mouse events asynchronously
    pub event_handler: EventHandler,

    /// The "Server" which may or may not return an error at some point, so we need to keep
    /// watching it
    pub proxy_server: Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>>,

    /// The receiver for events sent from the proxy
    pub proxy_rx: UnboundedReceiver<ProxyMessage>,

    pub response_waiter: ResponseWaiter,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(
        event_handler: EventHandler,
        proxy_server: Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>>,
        proxy_rx: UnboundedReceiver<ProxyMessage>,
    ) -> Self {
        Self {
            current_request_index: 0,
            requests: vec![],
            event_handler,
            proxy_server,
            proxy_rx,
            response_waiter: ResponseWaiter::default(),
        }
    }

    pub fn increment_list_index(&mut self) {
        if let Some(res) = self.current_request_index.checked_add(1) {
            self.current_request_index = res % self.requests.len();
        }
    }

    pub fn decrement_list_index(&mut self) {
        self.current_request_index = self
            .current_request_index
            .checked_sub(1)
            .unwrap_or(self.requests.len())
    }

    fn quit(&mut self, tui: &mut Tui) {
        // we're quitting. so what if we return an error.
        tui.exit().unwrap();
        std::process::exit(0);
    }

    pub async fn run(&mut self, tui: &mut Tui) {
        loop {
            tui.draw(self).expect("Couldn't draw tui");

            // this will just keep going until you kill the app, basically
            select! {
                ev = self.event_handler.next() => {
                    self.handle_event(ev, tui).await;
                }
                res = &mut self.proxy_server => {
                    println!("Got err from server: {res:?}");
                    self.quit(tui);
                }
                req = self.proxy_rx.recv() => {
                    if let Some(req) = req {
                        self.handle_request(req).await;
                    }
                }
                resp = &mut self.response_waiter.next() => {
                    if let Some(resp) = resp {
                        self.handle_request_response(resp).await;
                    }
                }
            }
        }
    }

    async fn handle_event(&mut self, event: io::Result<Event>, tui: &mut Tui) {
        match event {
            Err(e) => {
                println!("Agh! Everything has broken! {e}");
                self.quit(tui);
            }
            Ok(Event::Key(key)) => match key.code {
                KeyCode::Esc | KeyCode::Char('q') => self.quit(tui),
                KeyCode::Char('c') | KeyCode::Char('C')
                    if key.modifiers == KeyModifiers::CONTROL =>
                {
                    self.quit(tui)
                }
                KeyCode::Down | KeyCode::Char('j') => self.increment_list_index(),
                KeyCode::Up | KeyCode::Char('k') => self.decrement_list_index(),
                KeyCode::Char('f') | KeyCode::Char('F') => {
                    if let Some(req) = self.requests.get_mut(self.current_request_index) {
                        if let Some(mut rx) =
                            req.send_interaction(RequestInteraction::Forward).await
                        {
                            let id = req.id;
                            self.response_waiter.submit(Box::pin(async move {
                                let response = rx
                                    .recv()
                                    .await
                                    .expect("uhhh I don't know how to handle a None here");

                                RequestResponse { id, response }
                            }));
                        }
                    }
                }
                KeyCode::Char('d') | KeyCode::Char('D') => {
                    if let Some(req) = self.requests.get_mut(self.current_request_index) {
                        _ = req.send_interaction(RequestInteraction::Drop).await;
                    }
                }
                _ => {}
            },
            // eh. don't care.
            _ => {}
        }
    }

    async fn handle_request(
        &mut self,
        (req, sender): (hyper::Request<hyper::Body>, Sender<ProxyInteraction>),
    ) {
        // just add it to the list, then handle interacting with it in the `handle_event`
        // when it's selected
        self.requests.push(Request {
            id: Uuid::new_v4(),
            interaction_tx: Some(sender),
            inner: req,
            resp: None,
        });
        self.current_request_index = self.requests.len() - 1;
    }

    async fn handle_request_response(&mut self, resp: RequestResponse) {
        let Some(req) = self.requests.iter_mut().find(|r| r.id == resp.id) else {
            println!("Couldn't find request for id {}", resp.id);
            return;
        };

        req.store_response(resp).await;
    }
}

impl Debug for App {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("App")
            .field("current_request_index", &self.current_request_index)
            .field("requests", &self.requests)
            .field("event_handler", &self.event_handler)
            .finish()
    }
}
