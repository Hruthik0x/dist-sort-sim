use utility::CommFlags;
use std::mem::swap;
use std::net::TcpStream;
use std::io::{Read, Write};

use crate::node_utils::{Link, Node, PartialOrder, RelativePos};
use utility::log;

#[derive(PartialEq, Debug)]
pub struct OddEven;


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

impl OddEven {

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
                    Some(should_swap_right as fn(PartialOrder, i32, i32) -> bool))
                }

                // Current round -> even round and node is at odd index or 
                // Current round -> odd round and node is at even index
                (false, pos) if pos != RelativePos::Left => {
                    (node_data.left_link.as_mut(),
                        Some(should_swap_left as fn(PartialOrder, i32, i32) -> bool))
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

#[derive(Debug)]
pub struct Sasaki {
    num : i32,
    is_marked : bool,
}

impl Sasaki {
    fn receive_val(read_stream:&mut TcpStream) -> Sasaki{
        // max 5 used by CommFlags:Exchange (1) + i32 (4)
        let mut buffer = [0u8; 6];

        match read_stream.read(&mut buffer) {

            Ok(0) => {
                // Should assert all rounds are done and disconnection is not abrupt - flag{pending}
                // Client disconnected
                panic!("Client disconnected abruptly");
            }

            Ok(bytes_read) => {

                assert_eq!(bytes_read, 6);
                assert_eq!(buffer[0], CommFlags::Exchange as u8);
                assert!(buffer[1] < 2);


                Sasaki {
                     num : i32::from_le_bytes(
                         buffer[2..].try_into()
                        .expect(&format!("Failed to parse {:?} into i32", &buffer[2..]
                     ))),
                     is_marked : buffer[1] == 1
                }

            },
            Err(e) => panic!("Failed to read data :{}",e),
        }
    }

    fn send_recv_data (link:&mut Link, buffer:&mut [u8], num:&Sasaki) -> Sasaki {
        let (write_stream, read_stream) = (&mut link.write_stream, &mut link.read_stream);
        buffer[1] = if num.is_marked {1} else {0};
        buffer[2..].copy_from_slice(&num.num.to_le_bytes());
        assert_eq!(write_stream.write(&buffer).expect("Failed to send val"), 6);
        Self::receive_val(read_stream)
    }

    pub fn sasaki(node_data: &mut Node) -> i32 {
        let mut area:i8 = match node_data.rel_pos{
            RelativePos::Left => -1,
            _ => 0,
        };
        let mut buffer = [0u8; 6];
        buffer[0] = CommFlags::Exchange as u8;

        
        let is_marked = if node_data.rel_pos == RelativePos::Middle { false } else { true };
        
        let mut left_num = Sasaki{num:node_data.num, is_marked};
        let mut right_num = Sasaki{num:node_data.num, is_marked};


        for round in 0..node_data.rounds {

            if node_data.left_link.is_some() {
                let rec_val = Sasaki::send_recv_data(node_data.left_link.as_mut().unwrap(), &mut buffer, &left_num);
                log!("{} {} Received from left : {:?}", round, node_data.glb_pos, rec_val);
                if should_swap_left(node_data.partial_order, left_num.num, rec_val.num) {
                    // left_num = rec_val;
                    if left_num.is_marked {
                        area += 1;
                    }

                    if rec_val.is_marked {
                        area -= 1;
                    }

                    left_num = rec_val;
                }
            }

            if node_data.right_link.is_some() {
                let rec_val = Sasaki::send_recv_data(node_data.right_link.as_mut().unwrap(), &mut buffer, &right_num);
                log!("{} {} Received from right : {:?}", round, node_data.glb_pos, rec_val);
                if should_swap_right(node_data.partial_order, right_num.num, rec_val.num) {
                    right_num = rec_val;
                }
            }

            if node_data.rel_pos == RelativePos::Middle {
                if ((left_num.num < right_num.num) && (node_data.partial_order == PartialOrder::GreaterThan)) || 
                   ((left_num.num > right_num.num) && (node_data.partial_order == PartialOrder::LessThan)) {
                        swap(&mut left_num, &mut right_num);
                }
            }
        }
        if area == -1 {
            right_num.num 
        }   
        else {
            left_num.num
        }
    }

}

pub fn triplet(node_data: &Node) -> i32 {
    0
}