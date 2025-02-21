use std::net:: {TcpStream, TcpListener} ;

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
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

// push it to Node


pub struct Utility;

impl Utility {
    pub fn connect_to_server (port: u16) -> TcpStream {
        TcpStream::connect(format!("127.0.0.1:{}", port))
            .expect(&format!("Failed to connect to 127.0.0.1:{}", port))
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
