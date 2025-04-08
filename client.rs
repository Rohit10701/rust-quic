use std::fs::File;
use std::io::Read;
use std::net::SocketAddr;
use std::sync::Arc;

use quinn::{ClientConfig, Endpoint, Connection, RecvStream, SendStream};
use rustls::{pki_types::CertificateDer, RootCertStore};
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // load the server's certificate
    let cert_path = "cert.pem";
    let mut cert_file = File::open(cert_path)?;
    let mut cert_data = Vec::new();
    cert_file.read_to_end(&mut cert_data)?;
    
    // a client config ot trust the server's certificate
    let mut roots =rustls::RootCertStore::empty();
    let certs = rustls::pki_types::CertificateDer::try_from(cert_data);

    for cert in certs {
        roots.add(cert)?;
    }

    let roots = Arc::new(roots);
    
    let client_config = ClientConfig::with_root_certificates(roots).unwrap();
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    
    let server_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    println!("Connected to server: {}", connection.remote_address());
    
    // opne a bidirectional stream
    let (mut send, mut recv) = connection.open_bi().await?;
    

    let message = b"Hello, QUIC server!";
    send.write_all(message).await?;
    send.finish()?;
    
    // Receive the response
    let mut buf = [0; 8];
    recv.read(&mut buf);
        
    println!("Data {:?}", buf);
        
    // Close the connection
    connection.close(0u32.into(), b"Done");
    
    Ok(())
}