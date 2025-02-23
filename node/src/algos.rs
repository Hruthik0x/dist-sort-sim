use utility::CommFlags;
use std::net::TcpStream;
use std::io::{Read, Write};

use crate::node_util::{Node, Position, PartialOrder};
use utility::log;

#[derive(PartialEq, Debug)]
pub struct OddEven;

impl OddEven {

    fn should_swap_right (partial_order:PartialOrder, cur_num:i32, rec_val:i32) -> bool{
        (partial_order == PartialOrder::LessThan && 
            cur_num > rec_val) || 
        (partial_order == PartialOrder::GreaterThan &&
            cur_num < rec_val)   
    }

    fn should_swap_left (partial_order:PartialOrder, cur_num:i32, rec_val:i32) -> bool{
        (partial_order == PartialOrder::LessThan && 
            cur_num < rec_val) || 
        (partial_order == PartialOrder::GreaterThan &&
            cur_num > rec_val)   
    }

    fn receive_val(read_stream:&mut TcpStream) -> i32{
        // max 5 used by CommFlags:Exchange (1) + i32 (4)
        let mut buffer = [0u8; 5];

        match read_stream.read(&mut buffer) {

            Ok(0) => {
                // Should assert all rounds are done and disconnection is not abrupt - flag{pending}
                // Client disconnected
                panic!("Client disconnected abruptly");
            }

            Ok(bytes_read) => {

                log!("Receivced from neighbour [{}] : {:?}", bytes_read, &buffer[..bytes_read]);

                assert_eq!(bytes_read, 5);
                assert_eq!(buffer[0], CommFlags::Exchange as u8);

                i32::from_le_bytes(
                    buffer[1..].try_into()
                    .expect(&format!("Failed to parse {:?} into i32", &buffer[1..]
                )))

            },
            Err(e) => panic!("Failed to read data :{}",e),
        }
    }

    pub fn odd_even_transposition(node_data: &mut Node) -> i32{

        let mut is_odd_round     = true;
        let has_odd_index     = match node_data.global_pos % 2 {
                                        0 => false,
                                        1 => true,
                                        def_val => panic!("Is not supposed to happen ! returned : {}", def_val)
                                    };
        let mut buffer= [0u8; 5];

        buffer[0] = CommFlags::Exchange as u8;

        for _ in 0..node_data.rounds {

            let (read_stream, write_stream, compute_fn) =

            match (has_odd_index == is_odd_round, node_data.relative_pos) {

                // Current round -> odd round and node is at odd index or 
                // Current round -> even round and node is at even index
                (true, pos) if pos != Position::Right => {
                    (Some(&mut node_data.read_r), Some(&mut node_data.write_r), 
                    Some(Self::should_swap_right as fn(PartialOrder, i32, i32) -> bool))
                }

                // Current round -> even round and node is at odd index or 
                // Current round -> odd round and node is at even index
                (false, pos) if pos != Position::Left => {
                    (Some(&mut node_data.read_l), Some(&mut node_data.write_l), 
                        Some(Self::should_swap_left as fn(PartialOrder, i32, i32) -> bool))
                }
                _ => (None, None, None),
            };

            if let (Some(read_stream), Some(write_stream), Some(compute_fn)) =
                 (read_stream, write_stream, compute_fn) {

                buffer[1..].copy_from_slice(&node_data.num.to_le_bytes());

                assert_eq!(
                    write_stream.as_mut().unwrap()
                        .write(&buffer)
                        .expect("Failed to send the message"),
                    5
                );

                let rec_val = Self::receive_val(read_stream.as_mut().unwrap());

                // compute
                if compute_fn(node_data.partial_order, node_data.num, rec_val) {
                    node_data.num = rec_val;
                }
            }

            is_odd_round = !is_odd_round;
        }
        node_data.num
    }
}

// handle_should know the data ?
// starred 1
// non - starred 0

pub fn sasaki(node_data: &Node) -> i32 {
    let mut area:i8 = match node_data.relative_pos{
        Position::Left => -1,
        _ => 0,
    };
    // in sasaki rr-- ls++
    // receive a marked element on right stream, then area --
    // send the marked element on left stream, then area ++
    0
}

pub fn triplet(node_data: &Node) -> i32 {
    0
}