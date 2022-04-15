use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:10921";
    let bind_result = TcpListener::bind(addr).await;
    let listener = match bind_result {
        Ok(listener) => {
            println!("Server starts OK, listening {}", addr);
            listener
        }
        Err(e) => {
            println!("Failed to bind to {}, {:?}", addr, e);
            std::process::exit(1);
        }
    };

    loop {
        let accept_result = listener.accept().await;
        match accept_result {
            Ok((stream, socket_addr)) => {
                tokio::spawn(async move {
                    println!("Accepted a connection from {:?}", socket_addr);
                    process(stream, socket_addr).await;
                });
            }
            Err(e) => {
                println!("Failed to accept connection: {:?}", e);
            }
        }
    }
}

async fn process(stream: TcpStream, sock_address: SocketAddr) {
    let (mut rx, mut tx) = stream.into_split();
    let mut buf = [0u8; 8];
    let mut buf_index = 0usize;
    loop {
        match rx.read(&mut buf[buf_index..]).await {
            Ok(len) => {
                if 0 == len {
                    eprintln!("Connection to {} closed", sock_address);
                    return;
                }
                println!("Read {} bytes from {}", len, sock_address);
                buf_index += len;
                if buf_index >= 8 {
                    let mut seq = i64::from_be_bytes(buf);
                    println!("Client-Sequence: {}", seq);
                    seq += 1;
                    let bytes = seq.to_be_bytes();
                    buf_index = 0;
                    match tx.write_all(&bytes).await {
                        Err(e) => {
                            eprintln!("Failed to write data back to {}. {:?}", sock_address, e);
                            return;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read data from {}, {:?}", sock_address, e);
                return;
            }
        }
    }
}
