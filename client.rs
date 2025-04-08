use std::{fs::File, io::{self, BufReader}, net::SocketAddr, sync::Arc};
use quinn::{ClientConfig, Endpoint};
use rustls_pemfile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cert_path = "cert.pem";
    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    
    let mut roots: rustls::RootCertStore = rustls::RootCertStore::empty();

    // parse PEM certificates properly ( cna handle multiple)
    let certs = rustls_pemfile::certs(&mut reader);
    for cert in certs{
        match cert {
            Ok(certificates) => {
                roots.add(certificates)?;
            },
            Err(error) => println!("Error {:?}", error)
        }
    }
    

    
    
    let client_config = ClientConfig::with_root_certificates(Arc::new(roots)).unwrap();
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    
    let server_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    println!("Connected to server: {}", connection.remote_address());
    
    loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
    
        if line.trim().is_empty() {
            continue;
        }
    
        let (mut send, mut recv) = connection.open_bi().await?;
    
        send.write_all(line.as_bytes()).await?;
        send.finish();
    
        let mut buf = [0u8; 1024];
        match recv.read(&mut buf).await? {
            Some(n) => println!("Received: {}", String::from_utf8_lossy(&buf[..n])),
            None => println!("Stream closed"),
        }
    }
    
}