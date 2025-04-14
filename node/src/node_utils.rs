use std::net::{TcpListener, TcpStream};
use num_derive::FromPrimitive;
use utility::{CommFlags, Utility, log};
use std::io::{Read, Write};
use num_traits::FromPrimitive;
use super::algos::{OddEven, Sasaki, Triplet};

#[derive(FromPrimitive, Copy, Clone, Debug)]
pub enum Algo {
    OddEvenTransposition, 
    Sasaki, 
    Triplet
}

#[derive(FromPrimitive, PartialEq, Debug, Clone, Copy)]
pub enum RelativePos {
    Left,       // Node is leftmost node (it has no left neighbout)
    Right,      // Node is rightmost node (it has no right neigbout)
    Middle,     // Node has both left and right neigbours
}

#[derive(FromPrimitive, PartialEq, Debug, Clone, Copy)]
pub enum PartialOrder {
    LessThan,
    GreaterThan,
}

#[derive(Debug)]
pub struct Link {
    pub write_stream : TcpStream,
    pub read_stream  : TcpStream,
}

pub fn get_rounds (algo : Algo, no_nodes : u16) -> u16 {
    match algo {
        Algo::OddEvenTransposition => no_nodes,
        Algo::Sasaki               => no_nodes - 1,
        Algo::Triplet              => no_nodes - 1
    }
}

#[derive(Debug)]
pub struct Node {
    pub algo          : Algo,
    pub partial_order : PartialOrder,
    pub left_link     : Option<Link>,
    pub right_link    : Option<Link>, 
    pub rounds        : u16,
    pub rel_pos       : RelativePos,    // position relative to other nodes
                                        // Left => Left extreme node and does not have any left neighbour 
                                        // Right => Right extreme node and does not have any right neighbour
                                        // Middle => has both right and left neigbours

    pub glb_pos       : u16,            // not used by sasaki
    pub num           : i32,
}

struct Neighbour;

impl Neighbour {

    fn get_read_streams(listener: TcpListener, rel_pos:RelativePos) -> (Option<TcpStream>, Option<TcpStream>) {
        let mut no_clients:u8 = 0;

        // if pos == left or right, i,e the node is at left extreme or right extreme 
        // then it has to accept only 1 conncection, if it has both left and right 
        // neighbouts, the accept two connections,
        let max_clients:u8 = match rel_pos {
            RelativePos::Middle => 2,
            _ => 1
        };
        let mut buffer = [0u8; 2];
        let mut l_read = None;
        let mut r_read = None;

        while no_clients != max_clients {

            // iterate through all the incoming connections
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

                                    // Left neigbout node connecting
                                    claimed_pos if claimed_pos == RelativePos::Left as u8 => {
                                        // Connot have leftmost node have a left neighbour
                                        assert_ne!(rel_pos, RelativePos::Left);
                                        l_read = Some(stream);

                                        // If this node is rightmost node or 
                                        // already has left node connected
                                        if rel_pos == RelativePos::Right || r_read.is_some() {
                                            return (l_read, r_read);
                                        }
                                    },

                                    // Right neigbour node connecting
                                    claimed_pos if claimed_pos == RelativePos::Right as u8 => {
                                        // Connot have rightmost node have a right neighbour
                                        assert_ne!(rel_pos, RelativePos::Right);
                                        r_read = Some(stream);

                                        // if the is leftmost node or 
                                        // already has right node connected
                                        if rel_pos == RelativePos::Left || l_read.is_some() {
                                           return (l_read, r_read);
                                        }
                                    },

                                    def_val => panic!("Unexpected value {}", def_val)
                                }
                            },
                            // Failed to read bytes
                            Err(e) => panic!("Error : {}", e),
                        }
                    },
                    // Failed to accept the connection
                    Err(e) => panic!("Error : {}",e),
                }
            }
        }
        // Return the read streams
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

        // Setting flag as NeighbourConnect, signifying neighbour is connecting
        buffer[0] = CommFlags::NeigbourConnect as u8;

        // both left and right neigbours do not exist
        if l_port == 0 && r_port == 0 {
            panic!("Both ports cannot be zero !!");
        }

        // && r_port != 0
        // left neighbour does not exist, i.e this node is left most node
        if l_port == 0 {
            rel_pos = RelativePos::Left;
            r_stream = Some(Utility::connect_to_server(r_port));
            l_stream = None;
        }

        // && l_port != 0
        // right neighbout does not exist, i.e this node is right most node
        else if r_port == 0 {
            rel_pos = RelativePos::Right;
            l_stream = Some(Utility::connect_to_server(l_port));
            r_stream = None;
        }

        // both left and right and left neighbours exist
        else {
            rel_pos = RelativePos::Middle;
            l_stream = Some(Utility::connect_to_server(l_port));
            r_stream = Some(Utility::connect_to_server(r_port));
        }

        // if l_stream is not none, i.e if left neighbour is available
        // send the connect message to left neighbout
        if let Some(ref mut stream) = l_stream {

            // have to report to the nodes right neighbout that this node is its left neighbour
            buffer[1] = RelativePos::Right as u8;
            assert_eq!(stream.write(&buffer)
                .expect(&format!("Failed to send the message")), 2);
        }

        // if r_stream is not none, i.e if right neighbout is available
        // send the connect message to the right neigbout
        if let Some(ref mut stream) = r_stream {

            // have to report to the nodes right neighbout that this node is its left neighbour
            buffer[1] = RelativePos::Left as u8;
            assert_eq!(stream.write(&buffer)
                .expect(&format!("Failed to send the message")), 2);
        }

        (l_stream, r_stream, rel_pos)
    }

    // gets the read write streams and rel_pos
    pub fn get_links_rel_pos(listener: TcpListener, l_port:u16, r_port:u16) 
    -> (Option<Link>, Option<Link>, RelativePos) {

        // get write streams and the relative position
        let (l_write_stream, r_write_stream, rel_pos) = Self::get_write_streams(l_port, r_port);

        // get the read streams
        let (l_read_stream, r_read_stream) = Self::get_read_streams(listener, rel_pos);
        
        // if read and write streams are not none, then link is not none 

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

pub struct Distributor;

impl Distributor {

    // reports to the Distributor about its presence and its port num
    pub fn report(node_port: u16, stream: &mut TcpStream) {
        let mut buffer= [0u8; 3];
        buffer[0] = CommFlags::Report as u8;
        buffer[1..].copy_from_slice(&node_port.to_le_bytes());
        assert_eq!(stream.write(&buffer).expect("Failed to report to distributor"), 3);
    }

    pub fn handle_order(buffer: &[u8], listener:TcpListener) -> Node {
        if buffer.len() != 14 {
            panic!("Invalid order : {:?}", buffer);
        }
    
        else {

            // parsing through the received order
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

            // get left link, right link and the rel pos (left, right, middle)
            
            // link will have read and right stream to the neighbour
            // left and right links are none if the respective neighbouts do no exist 
            let (left_link, right_link, rel_pos) = Neighbour::get_links_rel_pos(listener, l_port, r_port);
            
            Node {algo, partial_order, left_link, right_link, rounds, rel_pos, glb_pos, num}
        }
    }

    pub fn start_sorting(node_data:&mut Node) -> i32 {

        assert_ne!(node_data.rounds, 0); 

        match node_data.algo {
            Algo::OddEvenTransposition => OddEven::odd_even_transposition(node_data),
            Algo::Sasaki               => Sasaki::sasaki(node_data),
            Algo::Triplet              => Triplet::triplet(node_data),
        }
    }
}