use std::env;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[tokio::main]
async fn main() {
    let args = env::args();
    let remote = args
        .skip(1)
        .next()
        .unwrap_or(String::from("localhost:10921"));
    println!("Connecting {}", remote);

    let tcp_stream = match TcpStream::connect(&remote).await {
        Ok(stream) => {
            println!("Connected!");
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to {}. {:?}", remote, e);
            return;
        }
    };
    let i = 0i64;

    let (mut rx, mut tx) = tcp_stream.into_split();
    let bytes = i.to_be_bytes();
    match tx.write_all(&bytes).await {
        Err(e) => {
            eprintln!("Failed to write data to {}. {:?}", remote, e);
            return;
        }
        _ => {}
    };

    let mut buf = [0u8; 8];
    let mut index = 0usize;
    loop {
        match rx.read(&mut buf[index..]).await {
            Ok(len) => {
                if 0 == len {
                    eprintln!("Connection to {} reached EOF", remote);
                    return;
                }
                index += len;
                if index >= 8 {
                    let seq = i64::from_be_bytes(buf) + 1;
                    if seq % 100 == 0 {
                        println!("Seq: {}", seq);
                    }
                    index = 0;
                    let bytes = seq.to_be_bytes();
                    if let Err(e) = tx.write_all(&bytes).await {
                        eprintln!("Failed to write data to {}. {:?}", remote, e);
                        return;
                    }
                }
            }
            Err(e) => {
                eprintln!("Failed to read data from {}. {:?}", remote, e);
                return;
            }
        }
    }
}
