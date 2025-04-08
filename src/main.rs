use std::fs::File;
use std::io::Write;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;

use quinn::{Connecting, Connection, Endpoint};
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::{ServerConfig, ServerConnection};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio_rustls::{TlsAcceptor, server::TlsStream};

#[path ="common/mod.rs"] mod common;

#[tokio::main]
async fn main() {
    setup_tls().await;
    run_server_over_quic().await;
}

async fn run_server_over_quic() {
    println!("run_server_over_quic to be implemented");
}

async fn setup_tls() {
    let (cert_pem, key_pem) = sign_cert_for_quic();

    let cert_path = "cert.pem";
    let key_path = "key.pem";

    File::create(cert_path)
        .unwrap()
        .write_all(cert_pem.as_bytes())
        .unwrap();

    File::create(key_path)
        .unwrap()
        .write_all(key_pem.as_bytes())
        .unwrap();

    let certs: Vec<CertificateDer> = CertificateDer::pem_file_iter(cert_path)
        .unwrap()
        .map(|cert| cert.unwrap())
        .collect();

    let private_key = PrivateKeyDer::from_pem_file(key_path).unwrap();

    let mut server_config = quinn::ServerConfig::with_single_cert(certs, private_key).unwrap();

    let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    let mut endpoint = Endpoint::server(server_config, addr).expect("Failed to create endpoint");

    println!("QUIC server listening on {}", addr);

    while let Some(connecting) = endpoint.accept().await {
        tokio::spawn(async move {
            match connecting.await {
                Ok(conn) => {
                    println!("Accepted connection from {:?}", conn.remote_address());
                }
                Err(err) => {
                    eprintln!("Failed to accept connection: {}", err);
                }
            }
        });
    }


}

fn sign_cert_for_quic() -> (String, String) {
    let subject_alt_names = vec!["localhost".to_string()];

    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names).unwrap();

    (cert.pem(), key_pair.serialize_pem())
}
