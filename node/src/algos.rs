use utility::CommFlags;
use std::net::TcpStream;
use std::io::{Read, Write};

use crate::node_utils::{Node, RelativePos, PartialOrder};
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
        let has_odd_index     = match node_data.glb_pos % 2 {
                                        0 => false,
                                        1 => true,
                                        def_val => panic!("Is not supposed to happen ! returned : {}", def_val)
                                    };
        let mut buffer= [0u8; 5];

        buffer[0] = CommFlags::Exchange as u8;

        for _ in 0..node_data.rounds {

            let (link, compute_fn) =

            match (has_odd_index == is_odd_round, node_data.rel_pos) {

                // Current round -> odd round and node is at odd index or 
                // Current round -> even round and node is at even index
                (true, pos) if pos != RelativePos::Right => {
                    (node_data.right_link.as_mut(),
                    Some(Self::should_swap_right as fn(PartialOrder, i32, i32) -> bool))
                }

                // Current round -> even round and node is at odd index or 
                // Current round -> odd round and node is at even index
                (false, pos) if pos != RelativePos::Left => {
                    (node_data.left_link.as_mut(),
                        Some(Self::should_swap_left as fn(PartialOrder, i32, i32) -> bool))
                }
                _ => (None, None),
            };

            if let (Some(link), Some(compute_fn)) =
                 (link, compute_fn) {

                let (write_stream, read_stream) = (&mut link.write_stream, &mut link.read_stream);

                buffer[1..].copy_from_slice(&node_data.num.to_le_bytes());

                assert_eq!(
                    write_stream
                        .write(&buffer).expect("Failed to send the message"),
                    5
                );

                let rec_val = Self::receive_val(read_stream);

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
    let mut area:i8 = match node_data.rel_pos{
        RelativePos::Left => -1,
        _ => 0,
    };

    // for _ in 0..node_data.rounds {
    //     if node_data.read_r
    // }

    // RelativePos => Left 
    // send element on right_W stream
    // receive element from right_r strem
    // update val, don't care if its marked or not

    // RelativePos => right 
    // send element on left_w stream

    // receive element from left_r stream
    // if update val 
    //       new val is marked (marked moved to right), area -- // receiver of marked element
    //       old val is marked (marked moved to left),  area ++ // sender   of marked element

    // middle 
    // send element on left_w stream
    // send element on right_w stream

    // recieve element on left_r stream
    //       new val is marked (marked moved to right), area -- // receiver of marked element
    //       old val is marked (marked moved to left),  area ++ // sender   of marked element

    // receive element on right_r stream
    // update val, don't care if val is marked or not

    if node_data.rel_pos == RelativePos::Left {
        
    }


    // in sasaki rr-- ls++
    // receive a marked element from left stream (marked element moved right) then self.area --
    // send the marked element on the left stream (markedd element moved left) then self.area ++
    0
}

pub fn triplet(node_data: &Node) -> i32 {
    0
}