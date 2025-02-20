// Rough Idea : Distributor, Nodes
// We run the distributor with a bunch of args, these args consist of the numbers to sort and algo to use
// The distributor runs a TCP server.
// The distributor then invokes the nodes (creates a process for each node)
// All the nodes request a port_num from the OS, start their own TCP server
// These nodes would then connect to the distributors TCP server
// The nodes would send their port num to the distributor
// The distributor would then send a message consisting algo, no.of nodes, num assigned, neighbour port nums
// In the nodes :
// the server part is (responsible for) used to read data from neighbours
// the client part is (responsible for) used to send data to the neighbours
// thus mimicing two channels, one for reading and one for writing.
// each node will have one listener, which gets two streams (neigbours)

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use num_traits::FromPrimitive ;

mod utils;
use utils:: {Algo, CommFlags, Node, Position, Utility};

mod algos;

const DEBUG: bool = true;

macro_rules! log {
    ($($arg:tt)*) => {
        if DEBUG {
            println!($($arg)*);
        }
    };
}

struct Distributor;
struct Neigbour;


impl Distributor {

    // Handles the communication with the distributor
    pub fn handle_distributor(distributor_port: u16, node_port: u16, 
                              l_lock:Arc<(Mutex<Option<i32>>, Condvar)>, 
                              r_lock:Arc<(Mutex<Option<i32>>, Condvar)>,
                              ready_lock:Arc<(Mutex<u8>, Condvar)>){
        // Max 15 used by Order
        let mut buffer = [0u8; 15];
        let mut stream = Utility::connect_to_server(distributor_port);
        let mut node_data: Node = Node::new();
        let mut l_lock = Some(l_lock);
        let mut r_lock = Some(r_lock);
    
        log!("Connected to distributor");
    
        Self::report(node_port, &mut stream);

        // I'll receive the order then stop reading data from the server
        // When the connection ends 
    
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                log!("Received [{}] : {:?}", bytes_read, &buffer[..bytes_read]);
                let cmd = buffer[0];
                match cmd {
                    cmd if cmd == CommFlags::Order as u8 => node_data = 
                        Self::handle_order(&buffer[1..]) ,
                    def_val => panic!("Invalid command : {}", cmd),
                };
            }
            Err(e) => log!("Failed to read : {}", e),
        }

        let max_clients:u8 = match node_data.self_pos {
            Position::Middle => 2,
            _ => 1
        };

        // aquire lock 
        // use while to keep receiving updates
        // breaks out when node_data.pos stuff 
        let (lock, cvar) = &*ready_lock;
        let mut client_count = lock.lock().unwrap();
        while *client_count < max_clients {
            client_count = cvar.wait(client_count).unwrap();
        }

        assert_eq!(*client_count, max_clients);

        // setting current client count to 255 marking that 
        // listen_incoming should stop accepting connections
        *client_count = 255;

        // releasing the lock
        drop(client_count);

        // not gonna drop the lock to reject anymore incoming connections

        // Sending the ready flag
        assert_eq!(stream.write(&[CommFlags::Ready as u8]).expect("Failed to send ready msg"), 1);

        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                log!("Received [{}] : {:?}", bytes_read, &buffer[..bytes_read]);
                let cmd = buffer[0];
                match cmd {           
                    cmd if cmd == CommFlags::Start as u8 => {
                        buffer[0] = CommFlags::Finish as u8;
                        buffer[1..5].copy_from_slice(
                            &Self::start_sorting(&mut node_data, l_lock.take().unwrap(), 
                                                        r_lock.take().unwrap()).to_le_bytes());
                        assert_eq!(stream.write(&buffer[..2]).expect("Failed to send msg"), 2);
                        },
                    _ => panic!("Invalid command : {}", buffer[0]),
                }
            }
            Err(e) => log!("Failed to read : {}", e),
        }

    }

    // reports to the Distributor about its presence and its port num
    fn report(node_port: u16, stream: &mut TcpStream) {
        let mut buffer= [0u8; 5];
        buffer[0] = CommFlags::Report as u8;
        buffer[1..].copy_from_slice(&node_port.to_le_bytes());
        assert_eq!(stream.write(&buffer).expect("Failed to report to distributor"), 5);
    }

    fn handle_order(buffer: &[u8]) -> Node {
        if buffer.len() != 14 {
            panic!("Invalid order : {:?}", buffer);
        }
    
        else {

            let algo = buffer[0];
            let partial_order = buffer[1];
            let no_nodes = &buffer[2..4];
            let l_port = &buffer[4..6];
            let r_port = &buffer[6..8];
            let global_pos = &buffer[8..10];
            let num = &buffer[10..14];


            let algo = FromPrimitive::from_u8(algo)
                .expect(&format!("Unknown algo {} (0 : Odd-Even | 1 : Sasaki | 2 : Triplet)", algo));
        
            let partial_order = FromPrimitive::from_u8(partial_order)
                .expect(&format!("Unknow partial order {} (0 : LessThan | 1 : GreaterThan)", partial_order));

            let no_nodes = u16::from_le_bytes(
                no_nodes.try_into()
                .expect(&format!("Failed to parse {:?} into u16", no_nodes
            )));

            let l_port = u16::from_le_bytes(
                l_port.try_into()
                .expect(&format!("Failed to parse {:?} into u16", l_port
            )));

            let r_port = u16::from_le_bytes(
                r_port.try_into()
                .expect(&format!("Failed to parse {:?} into u16", r_port
            )));

            let global_pos = u16::from_le_bytes(
                global_pos.try_into()
                .expect(&format!("Failed to parse {:?} into u16", global_pos
            )));


            let num = i32::from_le_bytes(
                num.try_into()
                .expect(&format!("Failed to parse {:?} into i32", num
            )));


            assert!(l_port == 0 && r_port == 0);
            assert!(no_nodes == 0);

            let (l_stream, r_stream, self_pos) = Neigbour::connect_to_neighbours(l_port, r_port);
            let rounds = Utility::get_rounds(algo, no_nodes);
            
            Node {algo, partial_order, l_stream, r_stream, rounds, self_pos, global_pos, num}
        }
    }

    fn start_sorting(node_data:&mut Node, 
        l_lock:Arc<(Mutex<Option<i32>>, Condvar)>, 
        r_lock:Arc<(Mutex<Option<i32>>, Condvar)>) -> i32 {

        assert_ne!(node_data.rounds, 0); 

        match node_data.algo {
            Algo::OddEvenTransposition => algos::odd_even(node_data, r_lock),
            Algo::Sasaki               => algos::sasaki(node_data),
            Algo::Triplet              => algos::triplet(node_data),
        }
    }

}

impl Neigbour {

    // listen to incoming conns from neighbour nodes
    pub fn listen_incoming(listener: TcpListener, 
                           l_lock : Arc<(Mutex<Option<i32>>, Condvar)>, 
                           r_lock : Arc<(Mutex<Option<i32>>, Condvar)>,
                           ready_lock:Arc<(Mutex<u8>, Condvar)> ){

        // wrapping in Some() so can be set to None when value is moved to threads.
        let mut l_lock = Some(l_lock);
        let mut r_lock = Some(r_lock);

        // Don't need more than 2 bytes
        let mut buffer = [0u8; 2];

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    
                    // trying to aquire lock for every new connection
                    let (lock, cvar) = &*ready_lock;
                    let mut client_count = lock.lock().unwrap();

                    // will be set by handle_distributor after max clients connected
                    // to reject the coming up connections
                    if *client_count == 255 {
                        break;
                    }

                    match stream.read(&mut buffer) {

                        Ok(bytes_read) => {

                            assert_eq!(bytes_read, 2);
                            assert_eq!(buffer[0], CommFlags::NeigbourConnect as u8);

                            let claimed_pos = buffer[1];
                            match claimed_pos {

                                claimed_pos if claimed_pos == Position::Left as u8 => {

                                    // should be some(lock), if it is none, it means
                                    // more than one connection claimed that its the left neighbour
                                    assert!(l_lock.is_some());

                                    // l_lock is moved to the thread and the local l_lock is set to None
                                    let lock = l_lock.take().unwrap();
                                    thread::spawn(move || Self::handle_neigbour(stream, lock));
                                },

                                claimed_pos if claimed_pos == Position::Right as u8 => {

                                    // should be some(lock), if it is none, it means
                                    // more than one connection claimed that its the right neighbour
                                    assert!(r_lock.is_some());

                                    // r_lock is moved to the thread and the locl r_lock is set to None
                                    let lock = r_lock.take().unwrap();
                                    thread::spawn(move || Self::handle_neigbour(stream, lock)); 
                                }

                                def_val => panic!("Unexpected value ! {}", def_val),
                            }

                            // update no.of clients connected
                            *client_count += 1;

                            // everytime a client is connected distributor is notified
                            cvar.notify_one();
                        }
                        Err(e) => panic!("Failed to read data : {}", e)
                    }
                }
                Err(e) => log!("Incoming Connection failed: {}", e),
            }
        }
    }

    // handle comms with the neighbour node
    pub fn handle_neigbour(mut stream: TcpStream, lock : Arc<(Mutex<Option<i32>>, Condvar)>) -> u8{
        // max 5 used by CommFlags:Exchange (1) + i32 (4)
        let mut buffer = [0u8; 5];
        let (lock, cvar) = &*lock;

        loop {
            match stream.read(&mut buffer) {

                Ok(bytes_read) => {

                    log!("Receivced [{}] : {:?}", bytes_read, &buffer[..bytes_read]);
                    let cmd = buffer[0];

                    match cmd {

                        cmd if cmd == CommFlags::Exchange as u8 => {
                            assert_eq!(buffer.len(), 5);   

                            let mut rec_val = lock.lock().unwrap();

                            // Can remove this line (was just a debugging check)
                            // checking if the previous value is consumed 
                            assert_eq!(*rec_val, None);

                            *rec_val = Some(
                                i32::from_le_bytes(
                                buffer[1..].try_into()
                                .expect(&format!("Failed to parse {:?} into i32", &buffer[1..]
                            ))));


                            // signal this to the thread handling sorting (start_sorting) 
                            // about the received number;
                            cvar.notify_one(); 
                        },

                        _ => panic!("Unknown command ! {}", cmd),

                    };
                },
                Err(e) => panic!("Error encountered :{}",e),
            }
        }
    }


    // Connects to nieghbour nodes and returns the streams
    // These streams are used to send data to the neighbours
    // Called by handle_distributor immediately after receiving 
    // order (CommFlags::Order) from the distributor
    fn connect_to_neighbours(l_port:u16, r_port:u16) -> 
        (Option<TcpStream>, Option<TcpStream>, Position) {
        
        let mut l_stream;
        let mut r_stream;
        let self_pos;
        let mut buffer = [0u8; 2];
        buffer[0] = CommFlags::NeigbourConnect as u8;

        if l_port == 0 && r_port == 0 {
            panic!("Both ports cannot be zero !!");
        }

        // && r_port != 0
        if l_port == 0 {
            self_pos = Position::Right;
            r_stream = Some(Utility::connect_to_server(r_port));
            l_stream = None;
        }

        // && l_port != 0
        else if r_port == 0 {
            self_pos = Position::Left;
            l_stream = Some(Utility::connect_to_server(l_port));
            r_stream = None;
        }

        else {
            self_pos = Position::Middle;
            l_stream = Some(Utility::connect_to_server(l_port));
            r_stream = Some(Utility::connect_to_server(r_port));
        }

        // if l_stream is not none, i.e if left neighbour is available
        // send the connect message
        if let Some(ref mut stream) = l_stream {

            // have to report to self left neighbour as its right neighbour
            buffer[1] = Position::Right as u8;
            assert_eq!(stream.write(&buffer)
                .expect(&format!("Failed to send the message")), 2);
        }

        // if r_stream is not none, i.e if right neighbout is available
        // send the connect message
        if let Some(ref mut stream) = r_stream {

            // have to report to self right neighbout as its left neighbour
            buffer[1] = Position::Left as u8;
            assert_eq!(stream.write(&buffer)
                .expect(&format!("Failed to send the message")), 2);
        }

        (l_stream, r_stream, self_pos)
    }


}


fn main() {
    let (listener, self_port_num) = Utility::start_server();
    log!("Assigned port : {}", self_port_num);
    let distributor_port:u16 = 32;

    let l_lock          = Arc::new((Mutex::new(None), Condvar::new()));
    let l_lock_clone    = Arc::clone(&l_lock);

    let r_lock          = Arc::new((Mutex::new(None), Condvar::new()));
    let r_lock_clone    = Arc::clone(&r_lock);

    let ready_lock         = Arc::new((Mutex::new(0u8), Condvar::new()));
    let ready_lock_clone   = Arc::clone(&ready_lock);


    // to handle comms with the distributor
    let distributor_handle = thread::spawn(move || { 
        Distributor::handle_distributor(distributor_port, self_port_num, 
                                        l_lock, r_lock, ready_lock) 
    });

    // to handle comms with other nodes
    Neigbour::listen_incoming(listener, l_lock_clone, r_lock_clone, ready_lock_clone);

    distributor_handle.join();
}
