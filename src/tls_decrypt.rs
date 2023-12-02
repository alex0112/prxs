use hyper::{Body, Client, Method, Request, Response, Server};
use tokio::fs::read;

pub async fn decrypt_tls_layer(req: &Request<Body>) -> Request<Body> {
    let cert_bytes = tokio::fs::read("./praxis_cert.pem").await;
    let privkey_bytes = tokio::fs::read("./praxis_key.pem").await;

    Request::new(Method::GET)
}
