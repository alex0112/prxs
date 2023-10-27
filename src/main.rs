use hyper::{
    service::{make_service_fn, service_fn},
    Body, Client, Request, Response, Server,
};
use std::{convert::Infallible, io::Write, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(hello_world)) });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {e}");
    }

    Ok(())
}

async fn hello_world(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("requests.log")
    {
        if let Err(e) = writeln!(file, "{req:?}") {
            eprintln!("Couldn't write to file: {e}");
        }
    }

    let client = Client::new();
    let response = client.request(req).await;
    println!("got response: {response:?}");

    Ok(Response::new("hello!".into()))
}
