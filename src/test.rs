use std::io::{Read, Write};
use std::net::TcpStream;

fn main() -> std::io::Result<()> {
    // Connect to the server
    let mut stream = TcpStream::connect("127.0.0.1:8080");

    // Send a message to the server
    let message = b"Hello, server!";
    stream.write_all(message)?;

    // Listen for the server's response
    let mut buffer = [0; 512];
    let n = stream.read(&mut buffer)?;

    // Print the server's response
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));

    Ok(())
}