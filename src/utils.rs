enum Algo {
    OddEvenTransposition, 
    Sasaki, 
    Triplet
}

enum Position {
    LeftCorner,
    RightCorner, 
    Middle,
}

enum Command {
    Order, 
    Start,
    Report,
}

struct Order {
    pub algo     : Algo,
    pub num      : i32,
    pub no_nodes : u16,
    pub l_port   : u16,
    pub r_port   : u16,
}

fn connect_to_server(port: u32) -> TcpStream {
    TcpStream::connect(format!("127.0.0.1:{}", port))
              .expect("Failed to connect to server")
}

