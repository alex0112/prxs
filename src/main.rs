use crossterm::event::Event;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use std::{convert::Infallible, io::Write, net::SocketAddr};
use tokio::sync::mpsc::{channel, unbounded_channel, Sender, UnboundedReceiver, UnboundedSender};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tui, tx) = Tui::new();
    let proxy = Proxy::new(tx).await;
    // and then just wait, show the TUI
    // TUI will listen to proxy_rx. When it gets a message, if it's a request, it'll query for
    // interaction, then send the response on the provided `Sender`, where it'll be processed
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

    let (tui_tx, mut tui_rx) = channel(1);

    tx.send(ProxyMessage::Request(req, tui_tx)).unwrap();

    let (interaction, req) = tui_rx.recv().await.unwrap();

    match interaction {
        RequestInteraction::Forward => {
            let client = Client::new();
            let response = client.request(req).await;
            println!("got response: {response:?}");

            Ok(Response::new("hello!".into()))
        }
        RequestInteraction::Drop => {
            todo!()
        }
    }
}

enum RequestInteraction {
    r#Drop,
    Forward,
}

enum ProxyMessage {
    HyperErr(hyper::Error),
    Request(Request<Body>, Sender<(RequestInteraction, Request<Body>)>),
}

enum TuiMessage {}

struct Tui {
    proxy_rx: UnboundedReceiver<ProxyMessage>,
    input_rx: UnboundedReceiver<std::io::Result<Event>>,
}

impl Tui {
    fn new() -> (Self, UnboundedSender<ProxyMessage>) {
        let (input_tx, input_rx) = unbounded_channel();
        std::thread::spawn(move || loop {
            if let Err(e) = input_tx.send(crossterm::event::read()) {
                eprintln!("Couldn't send event to Tui: {e})");
            }
        });

        let (tx_to_tui, proxy_rx) = unbounded_channel();

        (Self { input_rx, proxy_rx }, tx_to_tui)
    }
}

struct Proxy {}

impl Proxy {
    async fn new(tui_tx: UnboundedSender<ProxyMessage>) -> Self {
        let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

        let tx = tui_tx.clone();
        let make_svc = make_service_fn(move |_conn| {
            let tui_tx = tui_tx.clone();
            async move {
                Ok::<_, Infallible>(service_fn(move |req| {
                    let tui_tx = tui_tx.clone();
                    handle_proxied_req(req, tui_tx)
                }))
            }
        });

        let server = Server::bind(&addr).serve(make_svc);

        tokio::spawn(async move {
            if let Err(e) = server.await {
                eprintln!("server error: {e}");
                if let Err(e) = tx.send(ProxyMessage::HyperErr(e)) {
                    eprintln!(
                        "Agh! And now we can't tell anyone else that we ran into an error: {e}"
                    );
                }
            }
        });

        Self {}
    }
}