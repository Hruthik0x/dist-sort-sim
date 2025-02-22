use utility::CommFlags;
use std::sync::{Arc, Mutex, Condvar};
use std::io::Write;

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

    pub fn odd_even_transposition(node_data: &mut Node,
        l_lock:Arc<(Mutex<Vec<i32>>, Condvar)>,
        r_lock:Arc<(Mutex<Vec<i32>>, Condvar)>) -> i32{

        let mut is_odd_round     = true;
        let has_odd_index     = match node_data.global_pos % 2 {
                                        0 => false,
                                        1 => true,
                                        def_val => panic!("Is not supposed to happen ! returned : {}", def_val)
                                    };
        let mut buffer= [0u8; 5];

        buffer[0] = CommFlags::Exchange as u8;

        for _ in 0..node_data.rounds {

            let (lock, cvar, stream, compute_fn) = 

            match (has_odd_index == is_odd_round, node_data.relative_pos) {

                // Current round -> odd round and node is at odd index or 
                // Current round -> even round and node is at even index
                (true, pos) if pos != Position::Right => {
                    let (lock, cvar) = &*r_lock;
                    (Some(lock), Some(cvar), Some(&mut node_data.r_stream), 
                    Some(Self::should_swap_right as fn(PartialOrder, i32, i32) -> bool))
                }

                // Current round -> even round and node is at odd index or 
                // Current round -> odd round and node is at even index
                (false, pos) if pos != Position::Left => {
                    let (lock, cvar) = &*l_lock;
                    (Some(lock), Some(cvar), Some(&mut node_data.l_stream), 
                    Some(Self::should_swap_left as fn(PartialOrder, i32, i32) -> bool))
                }
                _ => (None, None, None, None),
            };

            if let (Some(lock), Some(cvar), Some(stream), Some(compute_fn)) = (lock, cvar, stream, compute_fn) {

                buffer[1..].copy_from_slice(&node_data.num.to_le_bytes());

                assert_eq!(
                    stream.as_mut().unwrap()
                        .write(&buffer)
                        .expect("Failed to send the message"),
                    5
                );

                let mut rec_val = lock.lock().unwrap(); 

                while rec_val.len() == 0 {
                    rec_val = cvar.wait(rec_val).unwrap();
                }

                let val: i32 = rec_val.remove(0);
                
                // releasing the lock, dropping the smart pointer, drops the mutex
                drop(rec_val);

                // compute
                if compute_fn(node_data.partial_order, node_data.num, val) {
                    node_data.num = val;
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

pub fn sasaki(node_data: &Node,         
    l_lock:Arc<(Mutex<Vec<i32>>, Condvar)>, 
    r_lock:Arc<(Mutex<Vec<i32>>, Condvar)> ) -> i32 {
    let mut area:i8 = match node_data.relative_pos{
        Position::Left => -1,
        _ => 0,
    };
    0
}

pub fn triplet(node_data: &Node,
    l_lock:Arc<(Mutex<Vec<i32>>, Condvar)>, 
    r_lock:Arc<(Mutex<Vec<i32>>, Condvar)> ) -> i32 {
    0
}