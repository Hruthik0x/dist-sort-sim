use std::net::{TcpListener, TcpStream};
use num_traits::FromPrimitive ;
use clap::Parser;
use std::io::{Read, Write};

use utility::{CommFlags, Utility, log};

mod algos;
mod node_utils;

use node_utils::{get_rounds, Algo, Link, Node, RelativePos};

struct Distributor;
struct Neigbour;


impl Distributor {

    // Handles the communication with the distributor
    pub fn handle_distributor(distributor_port: u16){

        let (listener, self_port_num) = Utility::create_server();

        // Max 15 used by Order
        let mut buffer = [0u8; 15];
        let mut stream = Utility::connect_to_server(distributor_port);
        let mut node_data;
    
        log!("Connected to distributor");
    
        Self::report(self_port_num, &mut stream);
    
        match stream.read(&mut buffer) {
            Ok(bytes_read) => {
                log!("Received from distributor [{}] : {:?}", bytes_read, &buffer[..bytes_read]);
                let cmd = buffer[0];
                match cmd {
                    cmd if cmd == CommFlags::Order as u8 => node_data = 
                        Self::handle_order(&buffer[1..], listener) ,

                        def_val => panic!("Invalid command : {}", def_val),
                };
            }
            Err(e) => panic!("Failed to read : {}", e),
        }

        buffer[0] = CommFlags::Finish as u8;
        buffer[1..5].copy_from_slice(
                &Self::start_sorting(&mut node_data).to_le_bytes()
        );
        assert_eq!(stream.write(&buffer[..5]).expect("Failed to send msg"), 5);
    }

    // reports to the Distributor about its presence and its port num
    fn report(node_port: u16, stream: &mut TcpStream) {
        let mut buffer= [0u8; 3];
        buffer[0] = CommFlags::Report as u8;
        buffer[1..].copy_from_slice(&node_port.to_le_bytes());
        assert_eq!(stream.write(&buffer).expect("Failed to report to distributor"), 3);
    }

    fn handle_order(buffer: &[u8], listener:TcpListener) -> Node {
        if buffer.len() != 14 {
            panic!("Invalid order : {:?}", buffer);
        }
    
        else {

            let algo = buffer[0];
            let partial_order = buffer[1];
            let no_nodes = &buffer[2..4];
            let l_port = &buffer[4..6];
            let r_port = &buffer[6..8];
            let glb_pos = &buffer[8..10];
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

            let glb_pos = u16::from_le_bytes(
                glb_pos.try_into()
                .expect(&format!("Failed to parse {:?} into u16", glb_pos
            )));


            let num = i32::from_le_bytes(
                num.try_into()
                .expect(&format!("Failed to parse {:?} into i32", num
            )));


            assert!(!(l_port == 0 && r_port == 0));
            assert!(no_nodes != 0);

            let rounds = get_rounds(algo, no_nodes);
            let (left_link, right_link, rel_pos) = Neigbour::get_links_rel_pos(listener, l_port, r_port);
            
            Node {algo, partial_order, left_link, right_link, rounds, rel_pos, glb_pos, num}
        }
    }

    fn start_sorting(node_data:&mut Node) -> i32 {

        assert_ne!(node_data.rounds, 0); 

        match node_data.algo {
            Algo::OddEvenTransposition => algos::OddEven::odd_even_transposition(node_data),
            Algo::Sasaki               => algos::Sasaki::sasaki(node_data),
            Algo::Triplet              => algos::Triplet::triplet(node_data),
        }
    }



}

impl Neigbour {

    fn get_read_streams(listener: TcpListener, rel_pos:RelativePos) -> (Option<TcpStream>, Option<TcpStream>) {
        let mut no_clients:u8 = 0;
        let max_clients:u8 = match rel_pos {
            RelativePos::Middle => 2,
            _ => 1
        };
        let mut buffer = [0u8; 2];
        let mut l_read = None;
        let mut r_read = None;

        while no_clients != max_clients {
            for stream in listener.incoming() { 
                match stream {
                    Ok(mut stream) => {
                        no_clients += 1;
                        match stream.read(&mut buffer) {
                            Ok(bytes_read) => {

                                log!("Received from neigbour : {:?}", &buffer[..bytes_read]);
                                assert_eq!(bytes_read, 2);
                                assert_eq!(buffer[0], CommFlags::NeigbourConnect as u8);
    
                                let claimed_pos = buffer[1];

                                match claimed_pos {

                                    claimed_pos if claimed_pos == RelativePos::Left as u8 => {
                                        assert_ne!(rel_pos, RelativePos::Left);
                                        l_read = Some(stream);
                                        if rel_pos == RelativePos::Right || 
                                           r_read.is_some() {
                                            return (l_read, r_read);
                                        }
                                    },

                                    claimed_pos if claimed_pos == RelativePos::Right as u8 => {
                                        assert_ne!(rel_pos, RelativePos::Right);
                                        r_read = Some(stream);
                                        if rel_pos == RelativePos::Left || 
                                           l_read.is_some() {
                                           return (l_read, r_read);
                                        }
                                    },

                                    def_val => panic!("Unexpected value {}", def_val)
                                }
                            },
                            Err(e) => panic!("Error : {}", e),
                        }
                    },
                    Err(e) => panic!("Error : {}",e),
                }
            }
        }
        (l_read, r_read)
    }



    // Connects to nieghbour nodes and returns the streams
    // These streams are used to send data to the neighbours
    // Called by handle_distributor immediately after receiving 
    // order (CommFlags::Order) from the distributor
    fn get_write_streams(l_port:u16, r_port:u16) -> 
        (Option<TcpStream>, Option<TcpStream>, RelativePos) {
        
        let mut l_stream;
        let mut r_stream;
        let rel_pos;
        let mut buffer = [0u8; 2];
        buffer[0] = CommFlags::NeigbourConnect as u8;

        if l_port == 0 && r_port == 0 {
            panic!("Both ports cannot be zero !!");
        }

        // && r_port != 0
        if l_port == 0 {
            rel_pos = RelativePos::Left;
            r_stream = Some(Utility::connect_to_server(r_port));
            l_stream = None;
        }

        // && l_port != 0
        else if r_port == 0 {
            rel_pos = RelativePos::Right;
            l_stream = Some(Utility::connect_to_server(l_port));
            r_stream = None;
        }

        else {
            rel_pos = RelativePos::Middle;
            l_stream = Some(Utility::connect_to_server(l_port));
            r_stream = Some(Utility::connect_to_server(r_port));
        }

        // if l_stream is not none, i.e if left neighbour is available
        // send the connect message
        if let Some(ref mut stream) = l_stream {

            // have to report to self left neighbour as its right neighbour
            buffer[1] = RelativePos::Right as u8;
            assert_eq!(stream.write(&buffer)
                .expect(&format!("Failed to send the message")), 2);
        }

        // if r_stream is not none, i.e if right neighbout is available
        // send the connect message
        if let Some(ref mut stream) = r_stream {

            // have to report to self right neighbout as its left neighbour
            buffer[1] = RelativePos::Left as u8;
            assert_eq!(stream.write(&buffer)
                .expect(&format!("Failed to send the message")), 2);
        }

        (l_stream, r_stream, rel_pos)
    }

    fn get_links_rel_pos(listener: TcpListener, l_port:u16, r_port:u16) 
    -> (Option<Link>, Option<Link>, RelativePos) {
        let (l_write_stream, r_write_stream, rel_pos) = Neigbour::get_write_streams(l_port, r_port);
        let (l_read_stream, r_read_stream) = Neigbour::get_read_streams(listener, rel_pos);
        
        let l_link = if let (Some(write_stream), Some(read_stream)) = (l_write_stream, l_read_stream) {
            Some(Link{write_stream, read_stream})
        }
        else {
            None
        };

        let r_link = if let (Some(write_stream), Some(read_stream)) = (r_write_stream, r_read_stream) {
            Some(Link{write_stream, read_stream})
        }
        else {
            None
        };

        (l_link, r_link, rel_pos)
    }

    


}


#[derive(Parser)]
#[command(version, 
    about = "Distributed sorting simulator - Node",
    long_about = "This program simulates multiple distributed sorting algos using\n\
                  sockets and processes, where each process emulates a node.\n\
                  This program emulates node.\n",
    author = "Hruthik <hruthikchalamareddy.c22@iiits.in"
)]
struct Args {
    #[arg(short, long, help = "Enter the distributor port (u16)")]
    dist_port : u16,
}

fn main() {
    // Receive distributor port from the terminal
    Distributor::handle_distributor(Args::parse().dist_port);
}
