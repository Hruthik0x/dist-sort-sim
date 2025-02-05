use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::thread;

fn handle_client(mut stream: TcpStream) {
    let client_addr = stream.peer_addr().unwrap(); // Get client's IP and port
    println!("New client connected: {}", client_addr);

    let mut buffer = [0; 512];

    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            let received_msg = String::from_utf8_lossy(&buffer[..bytes_read]);
            println!("Received from {}: {}", client_addr, received_msg);

            // Respond to client
            stream.write_all(b"Message received").unwrap();
        }
        Err(e) => println!("Failed to read from {}: {}", client_addr, e),
    }
}

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?; // Server listens on port 8080
    println!("Server running on 127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle_client(stream)); // Spawn thread for each client
            }
            Err(e) => println!("Connection failed: {}", e),
        }
    }

    Ok(())
}
