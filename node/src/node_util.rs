use std::net:: TcpStream;
use num_derive::FromPrimitive;

#[derive(FromPrimitive, Copy, Clone, Debug)]
pub enum Algo {
    OddEvenTransposition, 
    Sasaki, 
    Triplet
}

#[derive(FromPrimitive, PartialEq, Debug)]
pub enum Position {
    Left,
    Right,
    Middle,
}

#[derive(FromPrimitive, PartialEq, Debug)]
pub enum PartialOrder {
    LessThan,
    GreaterThan,
}

#[derive(Debug)]
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

// impl Node {
//     pub fn new() -> Node {
//         Node {
//             algo         : Algo::OddEvenTransposition, 
//             partial_order : PartialOrder::LessThan,
//             l_stream     : None, 
//             r_stream     : None, 
//             rounds       : 0, 
//             self_pos     : Position::Middle,
//             global_pos   : 0,
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