use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;

use hyper::Request;
use hyper::body::Bytes;
use hyper_util::rt::TokioIo;
use itertools::Itertools;
use rustls_pki_types::ServerName;
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let host = "localhost";
    let addr = "127.0.0.1:4433";

    let input = std::env::args().skip(1).join(" ");

    let mut root_cert_store = RootCertStore::empty();

    let mut pem = BufReader::new(File::open("ca.crt")?);
    for cert in rustls_pemfile::certs(&mut pem) {
        root_cert_store.add(cert?)?;
    }

    let config = Arc::new(
        ClientConfig::builder()
            .with_root_certificates(root_cert_store)
            .with_no_client_auth(),
    );

    let connector = TlsConnector::from(config);
    let domain = ServerName::try_from(host)?.to_owned();

    let tcp_stream = TcpStream::connect(addr).await?;

    let outer_tls_stream = connector.connect(domain.clone(), tcp_stream).await?;
    let inner_tls_stream = connector.connect(domain, outer_tls_stream).await?;

    let io = TokioIo::new(inner_tls_stream);
    let (mut sender, conn) = hyper::client::conn::http1::handshake(io).await?;

    tokio::spawn(async move {
        if let Err(err) = conn.await {
            eprintln!("Connection driver error: {:?}", err);
        }
    });

    let req = Request::builder()
        .uri(format!("https://{}/", host))
        .header("Host", host)
        .body(http_body_util::Full::new(Bytes::from(input)))?;

    let res = sender.send_request(req).await?;

    println!("\nResponse Status: {}", res.status());
    let body = http_body_util::BodyExt::collect(res.into_body()).await?;
    println!("Response Body: {:?}", body.to_bytes());

    Ok(())
}
