use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use http_body_util::{BodyExt, Full};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Error, Response};
use hyper_util::rt::TokioIo;
use rustls_pemfile::{certs, pkcs8_private_keys};
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tokio_rustls::rustls::ServerConfig;
use tokio_rustls::rustls::pki_types::{CertificateDer, PrivateKeyDer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:4433";

    let cert_file = &mut BufReader::new(File::open("server.crt")?);
    let cert_chain: Vec<CertificateDer> = certs(cert_file).collect::<Result<Vec<_>, _>>()?;

    let key_file = &mut BufReader::new(File::open("server.key")?);
    let keys: Vec<PrivateKeyDer> = pkcs8_private_keys(key_file)
        .map(|key| key.map(PrivateKeyDer::from))
        .collect::<Result<Vec<_>, _>>()?;

    let server_key = keys
        .into_iter()
        .next()
        .ok_or("No private keys found in server.key")?;

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, server_key)?;

    let acceptor = TlsAcceptor::from(Arc::new(config));
    let listener = TcpListener::bind(addr).await?;

    println!("HTTPSS Server listening on {}", addr);

    loop {
        let (stream, peer_addr) = listener.accept().await?;
        let acceptor = acceptor.clone();

        tokio::spawn(async move {
            println!("Connection from: {}", peer_addr);

            let outer_stream = match acceptor.accept(stream).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("\t[L1 Error]: {}", e);
                    return;
                }
            };
            println!("\t[L1] Outer layer success.");

            let inner_stream = match acceptor.accept(outer_stream).await {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("\t[L2 Error]: {}", e);
                    return;
                }
            };
            println!("\t[L2] Inner layer success.");

            let io = TokioIo::new(inner_stream);

            let service = service_fn(|req| async {
                let bytes = req.collect().await.unwrap().to_bytes();
                println!("\tReceived: {bytes:?}");
                Ok::<_, Error>(Response::new(Full::new(bytes)))
            });

            if let Err(err) = http1::Builder::new().serve_connection(io, service).await {
                eprintln!("  [Hyper Error]: {:?}", err);
            }
        });
    }
}
