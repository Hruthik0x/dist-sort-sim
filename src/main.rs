use std::net:: { TcpListener, TcpStream };
use std::thread;

const DEBUG: bool = true;
const DISTRIBUTOR_PORT : u = 8000;

macro_rules! log {
    ($($arg:tt)*) => {
        if DEBUG {
            println!($($arg)*);
        }
    };
}

// Handles the communication with the distributor
fn handle_distributor(node_port: u16) {

    let mut order:Order;
    let mut buffer = [0u8; 512];
    let mut stream = connect_to_server(server_port);
    log!("Connected to distributor");

    report(node_port, &mut stream);

    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                log!("Received [{}] : {:?}", bytes_read, &buffer[..bytes_read]);
                let cmd = buffer[0];
                match cmd {
                    cmd if cmd == Command::Order as u8 => order = handle_order(&buffer[1..]) ,
                    cmd if cmd == Command::Start as u8 => start_sorting(&buffer[1..], order),
                    _ => panic!("Invalid command : {}", buffer[0]),
                }
            }
            Err(e) => log!("Failed to read : {}", e),
        }
    }
}

// reports to the Distributor about its presence and its port num
fn report(node_port: u16, stream: &mut TcpStream) {
    let mut buffer: [u8; 5] = [0u8; 5];
    buffer[0] = Command::Report as u8;
    buffer[1..].copy_from_slice(&node_port.to_le_bytes());
    stream.write(&buffer).unwrap();
}


fn handle_order(buffer: &[u8]) -> Order {
    if buffer.len() != 17 {
        panic!("Invalid order : {:?}", buffer);
    }

    else {
        let order = Order {
            algo     : buffer[0] as Algo,
            num      : i32::from_le_bytes(buffer[1..5].try_into().unwrap()),
            no_nodes : u16::from_le_bytes(buffer[5..9].try_into().unwrap()),
            l_port   : u16::from_le_bytes(buffer[9..13].try_into().unwrap()),
            r_port   : u16::from_le_bytes(buffer[13..17].try_into().unwrap()),
        };

        node_utils::init(order);
        return order;
    }
}


mod server_utils {
    use std::net::TcpStream;
    use std::convert::TryInto;
    use std::io::{Write, Read};
    use crate::common_utils::{connect_to_server, Order, Algo};
    use crate::node_utils;

    const DEBUG:bool = true;






        // panic when connection is broken
    }

    // PANIC




    // panic
    fn start_sorting(buffer: &[u8], order: Order) {

        if buffer.len() != 0 {
            panic!("Invalid command : {}", buffer[0]);
        }
        else {
            // Have to change this
            log!("Starting sorting");
            node_utils::sort();
        }
    }
}

mod node_utils
{
    use std::net::TcpStream;
    use std::io::{Read, Write};
    use crate::common_utils::{connect_to_server, Order, Algo, Position};
    // std::mem::MaybeUninit;

    const DEBUG:bool = true;
    
    #[repr(u8)] 
    enum Command {
        InitConnect,
        LeftNode,
        RightNode,
        Exchange,
    }

    struct Node {
        algo    : Algo,
        l_neigh : Option<TcpStream>,
        r_neigh : Option<TcpStream>,
        rounds  : u16,
        num     : i32,
        pos     : Position,
    }

    impl Node {
        pub fn new() -> Node {
            Node {
                algo    : Algo::OddEvenTransposition,
                l_neigh : None,
                r_neigh : None,
                rounds  : 0,
                num     : 0,
                pos     : Position::Middle,
            }
        }
    }

    pub fn init(order: Order) -> Node {

        let mut node: Node = Node::new();

        node.algo = order.algo;
        node.num = order.num;
        node.rounds = order.no_nodes;

        let mut buffer = [0u8; 2];
        buffer[0] = Command::InitConnect as u8;


        if order.l_port == 0 && order.r_port == 0 {
            panic!("Both ports are 0");
        }
        else if order.l_port == 0 {
            node.pos = Position::LeftCorner;
            node.l_neigh = Some(connect_to_server(order.r_port as u32));
            buffer[1] = Command::LeftNode as u8;

        }
        else if order.r_port == 0 {
            node.pos = Position::RightCorner;
            node.r_neigh = Some(connect_to_server(order.l_port as u32));
            buffer[1] = Command::RightNode as u8;
        }
        else {
            node.pos = Position::Middle;
            node.l_neigh = Some(connect_to_server(order.l_port as u32));
            buffer[1] = Command::LeftNode as u8;
            
            node.r_neigh = Some(connect_to_server(order.r_port as u32));
            buffer[1] = Command::RightNode as u8;
        }

        node
        
    }

    pub fn handle_neighbour(mut stream: TcpStream) {
        let mut buffer = [0u8; 512];

        match stream.read(&mut buffer) {
            Ok(bytes_read) => {

                log!("Receivced [{}] : {:?}", bytes_read, &buffer[..bytes_read]);
                let cmd = buffer[0];
                match cmd {
                    cmd if cmd == Command::InitConnect as u8 => init(&buffer[1..]),
                    cmd if cmd == Command::Exchange as u8 => handel_exchange(&buffer[1..]),
                    _ => panic!("Invalid command : {}", buffer[0]),
                }
            }
            Err(e) => log!("Failed to read : {}", e),
        }
    }


    fn handel_exchange(buffer: &[u8]) {
        if buffer.len() != 1 {
            panic!("Invalid command : {}", buffer[0]);
        }
        else {
            // Have to change this

        }
    }
}

fn main() -> std::io::Result<()> {
    
    let listener: TcpListener = TcpListener::bind("127.0.0.1:0")?;
    let node_port: u16 = listener.local_addr()?.port();
    log!("Assigned port : {}", node_port);

    thread::spawn(move || server_utils::handle_distributor(DISTRIBUTOR_PORT, node_port));

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // seperate thread to handle each client
                thread::spawn(move || self::node_utils::handle_neighbour(stream));
            }
            Err(e) => log!("Connection failed: {}", e),
        }
    }
    log!("baby");

    Ok(())
}
