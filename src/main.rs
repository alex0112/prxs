use app::{App, AppResult};
use config::Config;
use event::EventHandler;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use request::RequestInteraction;
use std::{future::Future, io, pin::Pin};
use tui::Tui;

use http::request::Builder;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use std::{convert::Infallible, net::SocketAddr};
use tokio::sync::mpsc::{channel, unbounded_channel, Sender, UnboundedSender};

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

/// The config struct to manage options
mod config;

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = Config::retrieve();

    let (proxy_tx, proxy_rx) = unbounded_channel();
    let server = spawn_proxy(proxy_tx, &config).await;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new();
    let mut tui = Tui::new(terminal);
    tui.init()?;

    // Create an application.
    let mut app = App::new(events, server, proxy_rx);

    app.run(&mut tui).await;
    Ok(())
}

async fn clone_request(req: Request<Body>) -> (Request<Body>, Request<Body>) {
    fn build_builder(req: &Request<Body>) -> Builder {
        let mut builder = Request::builder()
            .uri(req.uri())
            .method(req.method())
            .version(req.version());

        for (name, value) in req.headers() {
            builder = builder.header(name, value);
        }

        builder
    }

    let first = build_builder(&req);
    let second = build_builder(&req);

    // TODO: Refuse to process a request if it's too large 'cause it could give us an OOM or
    // something with this function
    let body_data = hyper::body::to_bytes(req)
        .await
        .expect("This req was already constructed by hyper so it's trustworthy");

    (
        first.body(body_data.clone().into()).unwrap(),
        second.body(body_data.into()).unwrap(),
    )
}

async fn handle_proxied_req(
    req: Request<Body>,
    tx: UnboundedSender<ProxyMessage>,
) -> Result<Response<Body>, Infallible> {
    let (send_req, hold_req) = clone_request(req).await;

    let (tui_tx, mut tui_rx) = channel(1);

    tx.send((send_req, tui_tx)).unwrap();

    let interaction = tui_rx.recv().await.unwrap();

    match interaction {
        RequestInteraction::Forward => {
            let client = Client::new();
            match client.request(hold_req).await {
                Ok(resp) => Ok(resp),
                Err(e) => Ok(Response::new(format!("Couldn't proxy request: {e}").into())),
            }
        }
        RequestInteraction::Drop => Ok(Response::new("".into())),
    }
}

pub type ProxyMessage = (Request<Body>, Sender<RequestInteraction>);

// We're erasing the type here 'cause afaict it's impossible to name the type that results from
// calling `serve` and also all we really care about is that it's a future which may return a
// result
async fn spawn_proxy(
    tui_tx: UnboundedSender<ProxyMessage>,
    config: &Config,
) -> Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    let make_svc = make_service_fn(move |_conn| {
        let tui_tx = tui_tx.clone();
        async move {
            Ok::<_, Infallible>(service_fn(move |req| {
                let tui_tx = tui_tx.clone();
                handle_proxied_req(req, tui_tx)
            }))
        }
    });

    Box::pin(Server::bind(&addr).serve(make_svc))
}
