use app::{App, AppResult};
use config::Config;
use event::EventHandler;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use native_tls::TlsAcceptor;
use ratatui::{backend::CrosstermBackend, Terminal};
use request::ProxyInteraction;
use std::{future::Future, io, net::SocketAddr, pin::Pin};
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    oneshot::{self, channel},
};
use tui::Tui;

use tls_decrypt::decrypt_tls_layer;

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

/// Module that handles the TLS layer
mod tls_decrypt;

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
    if req.method() == http::Method::CONNECT {
        let decrypted_req = decrypt_tls_layer(&req).await;
        handle_proxied_req(decrypted_req, tx).await
    } else {
        let (send_req, hold_req) = clone_request(req).await;
        
        let (tui_tx, tui_rx) = channel();
        
        tx.send((send_req, tui_tx)).unwrap();
        
        let interaction = tui_rx.await.unwrap();
        
        match interaction {
            ProxyInteraction::Forward(tx) => {
                let client = Client::new();
                let resp = client.request(hold_req).await;
                
                let (send_resp, fw_resp) = match resp {
                    Ok(resp) => {
                        let (send_resp, fw_resp) = clone_response(resp).await;
                        (Ok(send_resp), Ok(fw_resp))
                    }
                Err(else { ; }) => (Err(e.to_string()), Err(e)),
                };
                
                if let Err(e) = tx.send(send_resp) {
                    println!("Couldn't send response to tui: {e:?}");
                }
                
                fw_resp
            }
            ProxyInteraction::Drop => Ok(Response::new("".into())),
        }
    }
   }
    
    pub type ProxyMessage = (Request<Body>, oneshot::Sender<ProxyInteraction>);

// We're erasing the type here 'cause afaict it's impossible to name the type that results from
// calling `serve` and also all we really care about is that it's a future which may return a
// result
async fn spawn_proxy(
    tui_tx: UnboundedSender<ProxyMessage>,
    config: &Config,
) -> Pin<Box<dyn Future<Output = Result<(), hyper::Error>>>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));

    let tls_acceptor: TlsAcceptor = tls_decrypt::gen_tls_acceptor().await;

    let make_svc = make_service_fn(move |_conn| {
        let tui_tx = tui_tx.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                let tui_tx = tui_tx.clone();
                handle_proxied_req(req, tui_tx)
            }))
        }
    });

    let https_connector: HttpsConnector::new();

    Box::pin(Server::bind(&addr).serve(make_svc))
}
