use utility::CommFlags;
use std::mem::swap;
use std::net::TcpStream;
use std::io::{Read, Write};

use crate::node_utils::{Link, Node, PartialOrder, RelativePos};

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

    fn send_val(write_stream:&mut TcpStream, num : i32, buffer:&mut [u8]) {
        
        // writing number to the next 4 bytes (1st byte is used for Exchange Flag)
        buffer[1..].copy_from_slice(&num.to_le_bytes());

        assert_eq!(
            write_stream
                .write(&buffer).expect("Failed to send the message"),
            5
        );
    }

    pub fn odd_even_transposition(node_data: &mut Node) -> i32{

        let mut is_odd_round  = true;
        let has_odd_index     = match node_data.glb_pos % 2 {
                                        0 => false,
                                        1 => true,
                                        def_val => panic!("Is not supposed to happen ! returned : {}", def_val)
                                    };
        let mut buffer= [0u8; 5];

        // Setting first byte as CommFlags::Exchange, indicating number exchange
        buffer[0] = CommFlags::Exchange as u8;

        for round in 0..node_data.rounds {

            println!("\n--------- Round : {} ---------", round);

            let (link, compute_fn, neighbour_string) =

            match (has_odd_index == is_odd_round, node_data.rel_pos) {

                // Current round is odd round and node is at odd index or 
                // Current round is even round and node is at even index
                // link = right_link and compute_fn = shyould_swap_right
                (true, pos) if pos != RelativePos::Right => {
                    (node_data.right_link.as_mut(),
                    Some(should_swap_right as fn(PartialOrder, i32, i32) -> bool),
                    "right")
                }

                // Current round is even round and node is at odd index or 
                // Current round is odd round and node is at even index
                // link = left_link and compute_fn = should_swap_left
                (false, pos) if pos != RelativePos::Left => {
                    (node_data.left_link.as_mut(),
                    Some(should_swap_left as fn(PartialOrder, i32, i32) -> bool),
                    "left")
                }

                // else link, compute_fn = None
                _ => (None, None, ""),
            };

            // if both link and compute_fn are not none
            if let (Some(link), Some(compute_fn)) =

                 // unpacked link and compute_fn
                 (link, compute_fn) {

                // unpacking write stream and left stream from link
                let (write_stream, read_stream) = (&mut link.write_stream, &mut link.read_stream);

                // sending this nodes number to appropriate neighbour
                Self::send_val(write_stream, node_data.num, &mut buffer);

                println!("Sent {} to {} neighbour", node_data.num, neighbour_string);

                // Receiving value from the appropriate neighbour
                let rec_val = Self::receive_val(read_stream);

                println!("Received {} from {} neighbour", node_data.num, neighbour_string);

                // compute, if the node has to update its value, depending on the partial order
                // and node's number and received number.
                if compute_fn(node_data.partial_order, node_data.num, rec_val) {
                    node_data.num = rec_val;
                }

                println!("Self num : {}", node_data.num);
            }

            is_odd_round = !is_odd_round;
        }
        node_data.num
    }
}

#[derive(Debug)]
pub struct Sasaki {
    num : i32,
    is_marked : bool,
}

impl Sasaki {
    fn receive_val(read_stream:&mut TcpStream) -> Sasaki{
        // max 6 used by CommFlags:Exchange (1) + val (4) + mark(1)
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

                // mark has to be either 0 or 1
                assert!(buffer[1] < 2);

                // This value is returned
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
    
        // Setting first byte as CommFlags::Exchange, indicating number exchange
        buffer[0] = CommFlags::Exchange as u8;
   
        let is_marked = if node_data.rel_pos == RelativePos::Middle { false } else { true };        
        let mut left_num = Sasaki{num:node_data.num, is_marked};
        let mut right_num = Sasaki{num:node_data.num, is_marked};

        println!("\nNote : '*' indicated the number is marked");

        for round in 0..node_data.rounds {
            println!("\n--------- Round : {} ---------", round);
            println!("Area : {}", area);

            // if left neigbour exists
            if node_data.left_link.is_some() {
                // Sending and receiving to and from left neighbour
                let rec_val = Sasaki::send_recv_data(node_data.left_link.as_mut().unwrap(), &mut buffer, &left_num);
   
                println!("Sent {}{} to left neigbour", left_num.num, Self::get_star_str(&left_num));
                println!("Received {}{} from left neigbour", rec_val.num, Self::get_star_str(&rec_val));

                // Check if the node has to update the local left val
                if should_swap_left(node_data.partial_order, left_num.num, rec_val.num) {

                    // if value sent to the left neighbour is marked
                    if left_num.is_marked {
                        area += 1;
                        println!("Area updated (area++) to {}", area);
                    }

                    // if the value received from the left neighbour is marked
                    if rec_val.is_marked {
                        area -= 1;
                        println!("Area updated (area--) to {}", area);
                    }

                    left_num = rec_val;
                }
            }

            // if right neighbout exists
            if node_data.right_link.is_some() {
                // Sending and receivng to and from the right neighbour
                let rec_val = Sasaki::send_recv_data(node_data.right_link.as_mut().unwrap(), &mut buffer, &right_num);

                println!("Sent {}{} to right neigbour", left_num.num, Self::get_star_str(&right_num));
                println!("Received {}{} from right neigbour", rec_val.num, Self::get_star_str(&rec_val));

                // Check if the node has to update the local right val
                if should_swap_right(node_data.partial_order, right_num.num, rec_val.num) {
                    right_num = rec_val;
                }
            }

            // internal sorting
            // it received values from both left and right 
            // so it has to arrange them
            if node_data.rel_pos == RelativePos::Middle {

                // partial order is > and left < right or 
                // partial order is < and right > left
                if ((left_num.num < right_num.num) && (node_data.partial_order == PartialOrder::GreaterThan)) || 
                   ((left_num.num > right_num.num) && (node_data.partial_order == PartialOrder::LessThan)) {
                        swap(&mut left_num, &mut right_num);
                }
                println!("After internal swapping : {}{} | {}{}", left_num.num, Self::get_star_str(&left_num),
                                                                  right_num.num, Self::get_star_str(&right_num));
            }
        }

        // After all rounds
        if area == -1 {
            // Return this
            right_num.num 
        }   
        else {
            // Return this
            left_num.num
        }
    }

    fn get_star_str(val: &Sasaki) -> &str {
        if val.is_marked {
            "*"
        }
        else  {
            ""
        }
    }

}

pub struct Alternate;
impl Alternate{
    fn receive_val(read_stream:&mut TcpStream) -> i32 {
        OddEven::receive_val(read_stream)
    }

    fn send_val(write_stream:&mut TcpStream, num : i32, buffer:&mut [u8]) {
        OddEven::send_val(write_stream, num, buffer);
    }

    pub fn alternate(node_data: &mut Node) -> i32 {
        let mut pos = node_data.glb_pos % 3;
        let mut buffer = [0u8;5];

        // Setting first byte as CommFlags::Exchange, indicating number exchange
        buffer[0] = CommFlags::Exchange as u8;

        for round in 0..node_data.rounds {
            println!("\n--------- Round : {} ---------", round);
            if pos == 1 {
                let mut nums = vec![node_data.num];

                // recieve values from neighbouts

                // receive from left neighbour (if exists)
                if let Some(link) = node_data.left_link.as_mut() {
                    let read_stream = &mut link.read_stream;
                    let val = Self::receive_val(read_stream);
                    println!("Received {} from left neighbour", val);
                    nums.push(val);
                } 

                // receive from right neighbour (if exists)
                if let Some(link) = node_data.right_link.as_mut() {
                    let read_stream = &mut link.read_stream;
                    let val = Self::receive_val(read_stream);
                    println!("Received {} from right neighbour", val);
                    nums.push(val);
                }

                // Conditional sorting as there are only thre nums 
                // quick sort would be overkill

                // Swap if out of order
                if nums[0] > nums[1] {
                    nums.swap(0, 1);
                }

                if nums.len() == 3 {
                    if nums[1] > nums[2] {
                        nums.swap(1, 2);
                    }
                    // Recheck 0 and 1, because 1 and 2 may have been swapped
                    if nums[0] > nums[1] {
                        nums.swap(0, 1);
                    }
                }

                // send appropriate values

                // Sending num to the left neighbur (if exists)
                if let Some(link) = node_data.left_link.as_mut() {
                    let write_stream = &mut link.write_stream;
                    let val = nums.remove(0);
                    Self::send_val(write_stream, val, &mut buffer);
                    node_data.num = nums.remove(0);
                    println!("Self number : {}", node_data.num);
                    println!("Sent {} to left neighbour", val);
                }

                // sending num to the right neighbour (if exists)
                if let Some(link) = node_data.right_link.as_mut() {
                    let write_stream = &mut link.write_stream;
                    if nums.len() == 2 {
                        node_data.num = nums.remove(0);
                        println!("Self number : {}", node_data.num);
                    }
                    let val = nums.remove(0);
                    Self::send_val(write_stream, val, &mut buffer);
                    println!("Sent {} to right neighbour", val);

                }
            }
            else {
                let link;
                // send and receive from the right neighbour
                if pos == 0 {
                    link = node_data.right_link.as_mut();
                }

                // send and receive from the left neighbour
                // pos == 2
                else { 
                    link = node_data.left_link.as_mut();
                }
                if let Some(link) = link {
                    let (write_stream, read_stream) = (&mut link.write_stream, &mut link.read_stream);
                    
                    // send num
                    Self::send_val(write_stream, node_data.num, &mut buffer);

                    if pos == 0 {
                        println!("Sent {} to right neighbour", node_data.num);
                    }
                    else  {
                        println!("Sent {} to left neighbour", node_data.num);
                    }
                    
                    // updte num to the received num
                    node_data.num = Self::receive_val(read_stream);

                    if pos == 0 {
                        println!("Received {} from right neighbour", node_data.num);
                    }
                    else  {
                        println!("Received {} from left neighbour", node_data.num);
                    }

                    println!("Self number : {}", node_data.num);
                }
            }

            // avoided % 3 for performance;
            pos += 1;
            if pos == 3  {
                pos = 0;
            }
        }
        // Return this
        node_data.num
    }
}