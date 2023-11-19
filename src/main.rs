use app::{App, AppResult};
use event::EventHandler;
use request::ProxyInteraction;
use tui::Tui;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    future::Future,
    io::{self, Write},
    net::SocketAddr,
    pin::Pin,
};
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

/// The module that waits on responses from forwarded requests
mod response_waiter;

#[tokio::main]
async fn main() -> AppResult<()> {
    invoke_tui().await
}

async fn invoke_tui() -> AppResult<()> {
    let (proxy_tx, proxy_rx) = unbounded_channel();
    let server = spawn_proxy(proxy_tx).await;

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

async fn clone_response(resp: Response<Body>) -> (Response<Body>, Response<Body>) {
    fn build_builder(resp: &Response<Body>) -> http::response::Builder {
        let mut builder = Response::builder()
            .status(resp.status())
            .version(resp.version());

        for (name, value) in resp.headers() {
            builder = builder.header(name, value);
        }

        builder
    }

    let first = build_builder(&resp);
    let second = build_builder(&resp);

    let body_data = hyper::body::to_bytes(resp)
        .await
        .expect("This resp was already constructed by hyper so it's trustworthy");

    (
        first.body(body_data.clone().into()).unwrap(),
        second.body(body_data.into()).unwrap(),
    )
}

async fn handle_proxied_req(
    req: Request<Body>,
    tx: UnboundedSender<ProxyMessage>,
) -> Result<Response<Body>, hyper::Error> {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("requests.log")
    {
        if let Err(e) = writeln!(file, "{req:?}") {
            eprintln!("Couldn't write to file: {e}");
        }
    }

    let (send_req, hold_req) = clone_request(req).await;

    let (tui_tx, mut tui_rx) = channel(1);

    tx.send((send_req, tui_tx)).unwrap();

    let interaction = tui_rx.recv().await.unwrap();

    match interaction {
        ProxyInteraction::Forward(tx) => {
            let client = Client::new();
            let resp = client.request(hold_req).await;

            let (send_resp, fw_resp) = match resp {
                Ok(resp) => {
                    let (send_resp, fw_resp) = clone_response(resp).await;
                    (Ok(send_resp), Ok(fw_resp))
                }
                Err(e) => (Err(e.to_string()), Err(e)),
            };

            if let Err(e) = tx.send(send_resp).await {
                println!("Couldn't send response to tui: {e}");
            }

            fw_resp
        }
        ProxyInteraction::Drop => Ok(Response::new("".into())),
    }
}

pub type ProxyMessage = (Request<Body>, Sender<ProxyInteraction>);

// We're erasing the type here 'cause afaict it's impossible to name the type that results from
// calling `serve` and also all we really care about is that it's a future which may return a
// result
async fn spawn_proxy(
    tui_tx: UnboundedSender<ProxyMessage>,
) -> Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

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
