// Note: Much of this file was cribbed from the hyper_rustls example
// https://github.com/rustls/hyper-rustls/blob/main/examples/server.rs

//! Simple HTTPS echo service based on hyper-rustls
//!
//! First parameter is the mandatory port to use.
//! Certificate and private key are hardcoded to sample files.
//! hyper will automatically use HTTP/2 if a client starts talking HTTP/2,
//! otherwise HTTP/1.1 will be used.

#![cfg(feature = "acceptor")]

use std::vec::Vec;
use std::{env, fs, io};

use hyper::server::conn::AddrIncoming;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use hyper_rustls::TlsAcceptor;
use pki_types::{CertificateDer, PrivateKeyDer};

fn main() {
    // Serve an echo service over HTTPS, with proper error handling.
    if let Err(e) = run_server() {
        eprintln!("FAILED: {}", e);
        std::process::exit(1);
    }
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

pub fn build_tls_acceptor(
    certs: Vec<CertificateDer<'static>>,
    key: PrivateKeyDer<'static>,
    addr: String,
) -> TlsAcceptor {
    let incoming = AddrIncoming::bind(&addr)?;
    let acceptor = TlsAcceptor::builder()
        .with_single_cert(certs, key)
        .map_err(|e| error(format!("{}", e)))?
        .with_all_versions_alpn()
        .with_incoming(incoming);

    acceptor
}

#[tokio::main]
async fn run_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // First parameter is port number (optional, defaults to )
    let port = match env::args().nth(1) {
        Some(ref p) => p.to_owned(),
        None => "7331".to_owned(),
    };
    let addr = format!("127.0.0.1:{}", port).parse()?;

    // Load public certificate.
    let certs = load_certs("examples/sample.pem")?;
    // Load private key.
    let key = load_private_key("examples/sample.rsa")?;
    // Build TLS configuration.

    let acceptor: TlsAcceptor = build_tls_acceptor(certs, key, addr);

    // Create a TCP listener via tokio.
    // let incoming = AddrIncoming::bind(&addr)?;
    // let acceptor = TlsAcceptor::builder()
    //     .with_single_cert(certs, key)
    //     .map_err(|e| error(format!("{}", e)))?
    //     .with_all_versions_alpn()
    //     .with_incoming(incoming);

    let service = make_service_fn(|_| async { Ok::<_, io::Error>(service_fn(echo)) });
    let server = Server::builder(acceptor).serve(service);

    // Run the future, keep going until an error occurs.
    println!("Starting to serve on https://{}.", addr);
    server.await?;
    Ok(())
}

// Custom echo service, handling two different routes and a
// catch-all 404 responder.
// async fn echo(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//     let mut response = Response::new(Body::empty());
//     match (req.method(), req.uri().path()) {
//         // Help route.
//         (&Method::GET, "/") => {
//             *response.body_mut() = Body::from("Try POST /echo\n");
//         }
//         // Echo service route.
//         (&Method::POST, "/echo") => {
//             *response.body_mut() = req.into_body();
//         }
//         // Catch-all 404.
//         _ => {
//             *response.status_mut() = StatusCode::NOT_FOUND;
//         }
//     };
//     Ok(response)
// }

// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<CertificateDer<'static>>> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    rustls_pemfile::certs(&mut reader).collect()
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<PrivateKeyDer<'static>> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.
    rustls_pemfile::private_key(&mut reader).map(|key| key.unwrap())
}

// use hyper::{Body, Client, Method, Request, Response, Server};
// use native_tls::{Identity, TlsAcceptor};
// use std::net::{TcpListener, TcpStream};

// pub async fn gen_tls_acceptor() -> TlsAcceptor {
//     let cert_bytes = tokio::fs::read("./praxis.pfx")
//         .await
//         .expect("Couldn't read the raw .pfx cert file! Aborting.");

//     let ident: Identity = Identity::from_pkcs12(cert_bytes.as_slice(), "")
//         .expect("Unable to generate identity file from .pfx cert file."); // Takes in the bytes of the cert, and a password, which we assume is empty

//     let acceptor: TlsAcceptor = TlsAcceptor::new(ident)
//         .expect("Could not form a TlsAcceptor from the given identity file!");

//     acceptor
// }
// pub async fn decrypt_tls_layer(req: &Request<Body>) -> Option<Request<Body>> {
//     todo!()
// }

// /// Given a hyper::Request<hyper::Body>,
// /// this function reads from a self signed cert file and
// /// creates a new native_tls::Identity from it.
// pub async fn decrypt_tls_layer(req: &Request<Body>) -> Option<Request<Body>> {
//     if req.method() != http::Method::CONNECT {
//         Some(*req)
//     } else {
//         let listener = TcpListener::bind("0.0.0.0:7331").unwrap(); // TODO: probably best to determine the port number for the connection dyamically at some point

//         let mut tcp_stream: Vec<TcpStream> = listener.incoming().collect().first();
//         let acceptor = gen_tls_acceptor().await;

//         let request: Request<Body> = read_tcp_stream(&mut tcp_stream, &acceptor);

//         Some(request)
//     }
// }

// fn read_tcp_stream(stream: &TcpStream, acceptor: &TlsAcceptor) -> Request<Body> {
//     let tlstream: Result<TlsStream<S>, HandshakeError<S>> = acceptor.accept(stream).unwrap();
//     parse_tcp(tlsstream)
// }

// fn parse_tcp(stream: TlsStream<TcpStream>) -> Request<Body> {
//     hyper::server::conn::Http::read(stream);
// }
