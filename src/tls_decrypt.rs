use hyper::{Body, Client, Method, Request, Response, Server};
use native_tls::{Identity, TlsAcceptor};
use std::net::{TcpListener, TcpStream};

pub async fn gen_tls_acceptor() -> TlsAcceptor {
    let cert_bytes = tokio::fs::read("./praxis.pfx")
        .await
        .expect("Couldn't read the raw .pfx cert file! Aborting.");

    let ident: Identity = Identity::from_pkcs12(cert_bytes.as_slice(), "")
        .expect("Unable to generate identity file from .pfx cert file."); // Takes in the bytes of the cert, and a password, which we assume is empty

    let acceptor: TlsAcceptor = TlsAcceptor::new(ident)
        .expect("Could not form a TlsAcceptor from the given identity file!");

    acceptor
}
pub async fn decrypt_tls_layer(req: &Request<Body>) -> Option<Request<Body>> {
    todo!()
}

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
