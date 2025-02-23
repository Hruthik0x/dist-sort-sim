use std::net:: TcpStream;
use num_derive::FromPrimitive;

#[derive(FromPrimitive, Copy, Clone, Debug)]
pub enum Algo {
    OddEvenTransposition, 
    Sasaki, 
    Triplet
}

#[derive(FromPrimitive, PartialEq, Debug, Clone, Copy)]
pub enum RelativePos {
    Left,
    Right,
    Middle,
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

#[derive(Debug)]
pub struct Node {
    pub algo          : Algo,
    pub partial_order : PartialOrder,
    pub left_link     : Option<Link>,
    pub right_link    : Option<Link>, 
    pub rounds        : u16,
    pub rel_pos  : RelativePos,    // position relative to other nodes
    pub glb_pos    : u16,         // not used by sasaki
    pub num           : i32,
}

// impl Node {
//     pub fn new() -> Node {
//         Node {
//             algo         : Algo::OddEvenTransposition, 
//             partial_order : PartialOrder::LessThan,
//             l_stream     : None, 
//             r_stream     : None, 
//             rounds       : 0, 
//             rel_pos     : RelativePos::Middle,
//             glb_pos   : 0,
//             num          : 0, 
//         }
//     }
// }

pub fn get_rounds (algo : Algo, no_nodes : u16) -> u16 {
    match algo {
        Algo::OddEvenTransposition => no_nodes,
        Algo::Sasaki               => no_nodes - 1,
        Algo::Triplet              => no_nodes - 1
    }
}