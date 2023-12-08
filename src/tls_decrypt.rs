// Note: Much of this file was cribbed from the hyper_rustls example
// https://github.com/rustls/hyper-rustls/blob/main/examples/server.rs

use std::vec::Vec;
use std::{env, fs, io};

use hyper::server::conn::AddrIncoming;
// use hyper::service::{make_service_fn, service_fn};
// use hyper::{Body, Method, Request, Response, Server, StatusCode};
use rustls::Certificate;
use rustls_pki_types::{CertificateDer, PrivateKeyDer};
use std::net::SocketAddr;

use hyper_rustls::TlsAcceptor;

/// When pointed at a local key.pem and cert.pem file pair,
/// construct a TLS Acceptor for use in the hyper server builder
///
pub fn build_tls_acceptor(
    certs: String,
    key: String,
    addr: SocketAddr,
) -> Result<TlsAcceptor, std::io::Error> {
    let certs: Vec<Certificate> = load_certs(&certs)?;
    let key = load_private_key(&key)?;

    let incoming = AddrIncoming::bind(&addr).unwrap();
    let acceptor = TlsAcceptor::builder()
        .with_single_cert(certs, key)
        .map_err(|e| error(format!("{}", e)))?
        .with_all_versions_alpn()
        .with_incoming(incoming);

    Ok(acceptor)
}

fn error(err: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, err)
}

// Load public certificate from file.
fn load_certs(filename: &str) -> io::Result<Vec<Certificate>> {
    // Open certificate file.
    let certfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(certfile);

    // Load and return certificate.
    Ok(rustls_pemfile::certs(&mut reader)?
        .into_iter()
        .map(Certificate)
        .collect())
}

// Load private key from file.
fn load_private_key(filename: &str) -> io::Result<rustls::PrivateKey> {
    // Open keyfile.
    let keyfile = fs::File::open(filename)
        .map_err(|e| error(format!("failed to open {}: {}", filename, e)))?;
    let mut reader = io::BufReader::new(keyfile);

    // Load and return a single private key.

    Ok(rustls_pemfile::rsa_private_keys(&mut reader)?
        .first()
        .map(|cert_data| rustls::PrivateKey(cert_data.clone()))
        .unwrap())
}
