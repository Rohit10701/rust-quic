// Use rustls_pemfile to correctly parse the PEM certificate
use std::{fs::File, io::BufReader, net::SocketAddr, sync::Arc};
use quinn::{ClientConfig, Endpoint};
use rustls_pemfile;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the server's certificate
    let cert_path = "cert.pem";
    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    
    // Create a client config to trust the server's certificate
    let mut roots: rustls::RootCertStore = rustls::RootCertStore::empty();

    // Parse PEM certificates properly
    let certs = rustls_pemfile::certs(&mut reader);
    for cert in certs{
        match cert {
            Ok(certificates) => {
                roots.add(certificates)?;
            },
            Err(error) => println!("Error {:?}", error)
        }
    }
    

    // Add each certificate to the root store
    
    
    let client_config = ClientConfig::with_root_certificates(Arc::new(roots)).unwrap();
    let mut endpoint = Endpoint::client("0.0.0.0:0".parse()?)?;
    endpoint.set_default_client_config(client_config);
    
    // Connect to server
    let server_addr: SocketAddr = "127.0.0.1:8080".parse()?;
    let connection = endpoint.connect(server_addr, "localhost")?.await?;
    println!("Connected to server: {}", connection.remote_address());
    
    // Open a bidirectional stream
    let (mut send, mut recv) = connection.open_bi().await?;
    
    // Send a message
    let message = b"Hello, QUIC server!";
    send.write_all(message).await?;
    send.finish()?;
    
    // Receive the response - need to handle this properly
    let mut buffer = Vec::new();
    while let Some(chunk) = recv.read_chunk(1024, false).await? {
        buffer.extend_from_slice(&chunk.bytes);
    }
    
    println!("Received: {:?}", String::from_utf8_lossy(&buffer));
    
    // Close the connection
    connection.close(0u32.into(), b"Done");
    
    Ok(())
}