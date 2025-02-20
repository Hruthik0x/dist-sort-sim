use std::net:: {TcpStream, TcpListener} ;
use num_derive::FromPrimitive;

#[derive(FromPrimitive, Copy, Clone)]
pub enum Algo {
    OddEvenTransposition, 
    Sasaki, 
    Triplet
}

#[derive(FromPrimitive, PartialEq)]
pub enum Position {
    Left,
    Right,
    Middle,
}

pub enum CommFlags {
    // Sent by Distributor to Node
    Order,
    Start,

    // Sent by Node to Distributor
    Report,
    Ready,
    Finish,

    // Sent from one node to another
    NeigbourConnect,
    Exchange,
}

#[derive(FromPrimitive, PartialEq)]
pub enum PartialOrder {
    LessThan,
    GreaterThan,
}

// #[derive (Copy, Clone)]
// pub struct Order {
//     pub algo     : Algo,
//     pub num      : i32,
//     pub no_nodes : u16,
//     pub l_port   : u16,
//     pub r_port   : u16,

// }

// impl Order {
//     pub fn new() -> Order {
//         Order {
//             algo     : Algo::OddEvenTransposition,
//             num      : 0,
//             no_nodes : 0,
//             l_port   : 0,
//             r_port   : 0,
//         }
//     }
// }

pub struct Node {
    pub algo         : Algo,
    pub partial_order : PartialOrder,
    pub l_stream     : Option<TcpStream>,
    pub r_stream     : Option<TcpStream>,
    pub rounds       : u16,
    pub self_pos     : Position,
    pub global_pos   : u16,   // not used by sasaki
    pub num          : i32,
}

impl Node {
    pub fn new() -> Node {
        Node {
            algo         : Algo::OddEvenTransposition, 
            partial_order : PartialOrder::LessThan,
            l_stream     : None, 
            r_stream     : None, 
            rounds       : 0, 
            self_pos     : Position::Middle,
            global_pos   : 0,
            num          : 0, 
        }
    }
}

pub struct Utility;

impl Utility {
    pub fn connect_to_server (port: u16) -> TcpStream {
        TcpStream::connect(format!("127.0.0.1:{}", port))
            .expect(&format!("Failed to connect to 127.0.0.1:{}", port))
    }

    pub fn get_rounds (algo : Algo, no_nodes : u16) -> u16 {
        match algo {
            Algo::OddEvenTransposition => no_nodes,
            Algo::Sasaki               => no_nodes - 1,
            Algo::Triplet              => no_nodes - 1
        }
    }

    pub fn start_server() -> (TcpListener, u16) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind :(");
        
        let port_num = listener.local_addr()
            .expect("Failed to get the port for self_server")
            .port();
    
        (listener, port_num)
    }
}
