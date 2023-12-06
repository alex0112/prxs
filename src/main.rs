#![allow(clippy::module_name_repetitions)]

use app::{App, AppResult};
use async_trait::async_trait;
use config::{Config, Session};
use event::EventHandler;
use request::ProxyInteraction;
use tui::Tui;

use flate2::read::GzDecoder;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use layout::LayoutState;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    future::Future,
    io::{stdout, Read},
    net::SocketAddr,
    pin::Pin,
};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    oneshot::{self, channel},
};

/// Application.
mod app;

/// Terminal events handler.
mod event;

/// Widget renderer.
mod ui;

/// Terminal user interface
mod tui;

/// Request wrapper struct
mod request;

/// The module that waits on responses from forwarded requests
mod response_waiter;

/// The config struct to manage options
mod config;

/// The struct to manage the state of what's shown to the user
mod layout;

/// To manage the state of the bottom input bar
mod input_state;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Config::get();

    let session = config
        .session_file
        .as_ref()
        .and_then(|file| Session::restore(file).ok())
        .unwrap_or_default();

    let (proxy_tx, proxy_rx) = unbounded_channel();
    let server = spawn_proxy(proxy_tx);

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal);
    tui.init()?;

    let layout = LayoutState::default();

    // Create an application.
    let mut app = App::new(events, server, proxy_rx, session);

    app.run(tui, layout).await;
    Ok(())
}

async fn handle_proxied_req(
    req: Request<Body>,
    tx: UnboundedSender<ProxyMessage>,
) -> Result<Response<Body>, hyper::Error> {
    let (send_req, hold_req) = req.clone().await;

    let (tui_tx, tui_rx) = channel();

    tx.send((send_req, tui_tx)).unwrap();

    let interaction = tui_rx.await.unwrap();

    match interaction {
        ProxyInteraction::Forward(tx) => {
            let client = Client::new();
            let resp = client.request(hold_req).await;

            let (send_resp, fw_resp) = match resp {
                Ok(resp) => {
                    let (send_resp, fw_resp) = resp.clone().await;
                    (Ok(send_resp), Ok(fw_resp))
                }
                Err(e) => (Err(e.to_string()), Err(e)),
            };

            if let Err(e) = tx.send(send_resp) {
                println!("Couldn't send response to tui: {e:?}");
            }

            fw_resp
        }
        ProxyInteraction::Drop => Ok(Response::new("".into())),
    }
}

pub type ProxyMessage = (Request<Body>, oneshot::Sender<ProxyInteraction>);

// We're erasing the type here 'cause afaict it's impossible to name the type that results from
// calling `serve` and also all we really care about is that it's a future which may return a
// result
fn spawn_proxy(
    tui_tx: UnboundedSender<ProxyMessage>,
) -> Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], Config::get().port));

    let make_svc = make_service_fn(move |_conn| {
        let tui_tx = tui_tx.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let tui_tx = tui_tx.clone();
                handle_proxied_req(req, tui_tx)
            }))
        }
    });

    Box::pin(Server::bind(&addr).serve(make_svc))
}

#[async_trait]
trait ConsumingClone
where
    Self: Sized,
{
    async fn clone(self) -> (Self, Self);
}

#[async_trait]
impl ConsumingClone for Request<Body> {
    async fn clone(self) -> (Self, Self) {
        fn build_builder(req: &Request<Body>) -> http::request::Builder {
            let mut builder = Request::builder()
                .uri(req.uri())
                .method(req.method())
                .version(req.version());

            for (name, value) in req.headers() {
                builder = builder.header(name, value);
            }

            builder
        }

        let first = build_builder(&self);
        let second = build_builder(&self);

        // TODO: Refuse to process a request if it's too large 'cause it could give us an OOM or
        // something with this function
        // also should we use async_trait for this? Honestly I don't want to pull it in as a
        // dependency and kinda just wanna wait until afit and rtitit are stabilized
        // in a few weeks
        let body_data = hyper::body::to_bytes(self)
            .await
            .expect("This req was already constructed by hyper so it's trustworthy");

        (
            first.body(body_data.clone().into()).unwrap(),
            second.body(body_data.into()).unwrap(),
        )
    }
}

#[async_trait]
impl ConsumingClone for Response<Body> {
    async fn clone(self) -> (Self, Self) {
        fn build_builder(resp: &Response<Body>) -> http::response::Builder {
            let mut builder = Response::builder()
                .status(resp.status())
                .version(resp.version());

            for (name, value) in resp.headers() {
                builder = builder.header(name, value);
            }

            builder
        }

        let first = build_builder(&self);
        let second = build_builder(&self);

        let body_data = hyper::body::to_bytes(self)
            .await
            .expect("This resp was already constructed by hyper so it's trustworthy");

        (
            first.body(body_data.clone().into()).unwrap(),
            second.body(body_data.into()).unwrap(),
        )
    }
}

#[async_trait]
impl<T: ConsumingClone + Send> ConsumingClone for Option<T> {
    async fn clone(self) -> (Self, Self) {
        match self {
            None => (None, None),
            Some(inner) => {
                let (inner1, inner2) = inner.clone().await;
                (Some(inner1), Some(inner2))
            }
        }
    }
}

fn gunzip(data: impl AsRef<[u8]>) -> std::io::Result<Vec<u8>> {
    let mut decoder = GzDecoder::new(data.as_ref());
    let mut v = Vec::new();
    decoder.read_to_end(&mut v)?;
    Ok(v)
}
