use std::io::Read;
use std::sync::Arc;
use std::{error, usize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use rcgen::{generate_simple_self_signed, CertifiedKey};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;

#[tokio::main]
async fn main(){
    setup_tls().await;
}


async fn setup_tls() {
    let (cert_pem, key_pem) = sign_cert_for_quic();

    // Write to temp files
    let cert_path = "cert.pem";
    let key_path = "key.pem";
    File::create(cert_path).unwrap().write_all(cert_pem.as_bytes()).unwrap();
    File::create(key_path).unwrap().write_all(key_pem.as_bytes()).unwrap();

    let certs: Vec<CertificateDer> = CertificateDer::pem_file_iter(cert_path)
        .unwrap()
        .map(|cert| cert.unwrap())
        .collect();

    let private_key = PrivateKeyDer::from_pem_file(key_path).unwrap();

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, private_key)
        .unwrap();

    let acceptor = TlsAcceptor::from(Arc::new(config));

    println!("TLS setup successful");

    
    let listener = TcpListener::bind("[::]:8080").await.unwrap();
    let (stream, addr) = listener.accept().await.unwrap();

    match acceptor.accept(stream).await {
        Ok(mut tls_stream) => {
            println!("Client connected from {}", addr);
            loop {
                let mut buf = [0u8; 8];
                match tls_stream.read(&mut buf).await {
                    Ok(0) => {
                        println!("Client disconnected.");
                        break;
                    }
                    Ok(n) => println!("Received ({} bytes): {:?}", n, &buf[..n]),
                    Err(e) => {
                        println!("Error reading: {:?}", e);
                        break;
                    }
                }
            }
        }
        Err(e) => println!("TLS handshake failed: {}", e),
    }
    

}



// for quic we need self signed tls certificate
// certificate, private key
fn sign_cert_for_quic() -> (String, String) {
    // Generate a certificate that's valid for "localhost" and "hello.world.example"
    let subject_alt_names = vec!["localhost:8080".to_string()];
    
    let CertifiedKey { cert, key_pair } = generate_simple_self_signed(subject_alt_names).unwrap();
    println!("{}", cert.pem());
    println!("{}", key_pair.serialize_pem());
    return (cert.pem(), key_pair.serialize_pem())
}

// openssl s_client -connect localhost:8080
