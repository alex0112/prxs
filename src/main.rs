use app::{App, AppResult};
use event::EventHandler;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use request::RequestInteraction;
use std::{future::Future, io, pin::Pin};
use tui::Tui;

use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use std::{convert::Infallible, io::Write, net::SocketAddr};
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

async fn handle_proxied_req(
    req: Request<Body>,
    tx: UnboundedSender<ProxyMessage>,
) -> Result<Response<Body>, Infallible> {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("requests.log")
    {
        if let Err(e) = writeln!(file, "{req:?}") {
            eprintln!("Couldn't write to file: {e}");
        }
    }

    // TODO: Refuse to process a request if it's too large 'cause it could give us an OOM or
    // something with this function
    let req_body = hyper::body::to_bytes(req)
        .await
        .expect("This req was already constructed by hyper so it's trustworthy");

    // So unfortunately we need to "clone" the Request (it's not actually cloning it as Request
    // doesn't impl Clone, see hyper#1300, but whatever) because we need to hold onto an owned
    // `Request` here to pass it into the `client.request` down below, but we also need the
    // `request::Request` to hold onto an owned `hyper::Request` so it can be replayed later or
    // viewed or whatever else. So this is what we have to do.
    let send_req = Request::builder()
        .body(Body::from(req_body.clone()))
        .expect("We just deconstructed this");
    let hold_req = Request::builder()
        .body(Body::from(req_body))
        .expect("We just deconstructed this");

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
        RequestInteraction::Drop => Ok(Response::new("".into()))
    }
}

pub type ProxyMessage = (Request<Body>, Sender<RequestInteraction>);

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
            Ok::<_, Infallible>(service_fn(move |req| {
                let tui_tx = tui_tx.clone();
                handle_proxied_req(req, tui_tx)
            }))
        }
    });

    Box::pin(Server::bind(&addr).serve(make_svc))
}
