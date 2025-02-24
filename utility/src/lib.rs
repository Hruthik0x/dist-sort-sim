use std::net:: {TcpStream, TcpListener} ;

// log macro, works same as println macro
// will print only in debug mode
// will not show output in release mode
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

pub struct Utility;

impl Utility {

    // Connects to a socket server
    pub fn connect_to_server (port: u16) -> TcpStream {
        TcpStream::connect(format!("127.0.0.1:{}", port))
            .expect(&format!("Failed to connect to 127.0.0.1:{}", port))
    }

    // create sa socket server
    pub fn create_server() -> (TcpListener, u16) {
        let listener = TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind :(");
        
        let port_num = listener.local_addr()
            .expect("Failed to get the port for self_server")
            .port();
    
        (listener, port_num)
    }
}
