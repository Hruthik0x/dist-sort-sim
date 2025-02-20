use crate::utils:: {CommFlags, Node, Position, PartialOrder};
use std::sync::{Arc, Mutex, Condvar};
use std::io:: Write;
use std::mem::swap;

#[derive(PartialEq)]
enum OddEven {
    Odd,
    Even
}

// odd round, : 
//      nodes at odd numbers wait for data from right neighbour
//      nodes at even numbers send data to left neigbour

// Even round : 
//      nodes at even numbers wait for data from right neighbout
//      nodes at odd numbers send data to left neighbour


pub fn odd_even(node_data: &mut Node,
    r_lock:Arc<(Mutex<Option<i32>>, Condvar)>) -> i32{

    let mut sender;
    let mut receiver;
    let mut odd_round   = true;
    let self_pos: OddEven     = match node_data.global_pos % 2 {
                                    0 => OddEven::Even,
                                    1 => OddEven::Odd,
                                    def_val => panic!("Is not supposed to happen ! returned : {}", def_val)
                                };
    let mut buffer= [0u8; 5];

    buffer[0] = CommFlags::Exchange as u8;

    for _ in 1..node_data.rounds+1 {

        // avoided % operator for round as it is computationally expensive

        if odd_round {
            sender = OddEven::Even;
            receiver = OddEven::Odd;
            odd_round = false;
        }

        else {
            sender = OddEven::Odd;
            receiver = OddEven::Even;
            odd_round = true;
        }


        // have a neighbour at right, in other words :
        // if the node is leftmost or in middle 
        //   it has a neighbour at right, i,e 
        //   its position should not be rightmost
        if self_pos == receiver && node_data.self_pos != Position::Right {

            let (lock, cvar) = &*r_lock;

            // smart pointer (MutexGaurd) to the mutex 
            // no need to dereference lock => (*lock).lock().unwrap() 
            // as rust automatically dereferences it.
            let mut rec_val = lock.lock().unwrap(); 

            // no need to mention (*rec_val).is_some() as rust automatically dereferences it
            while ! rec_val.is_some() {
                rec_val = cvar.wait(rec_val).unwrap();
            }

            let mut send_val = rec_val.unwrap();

            // marking it as value consumed.
            *rec_val = None;
            
            // releasing the lock, dropping the smart pointer, drops the mutex
            drop(rec_val);

            // compute

            // partial order = LessThan : 
            //     send larger val to right, retain smaller val
            // partial order = GreaterThan :
            //     send smaller val to right, retain larger val
            if  (node_data.partial_order == PartialOrder::LessThan && 
                 node_data.num > send_val) || 
                (node_data.partial_order == PartialOrder::GreaterThan &&
                    node_data.num < send_val)    
            {
                swap(&mut node_data.num, &mut send_val);
            }

            buffer[1..].copy_from_slice(&send_val.to_le_bytes());

            // send data to right
            assert_eq!(
                node_data.r_stream.as_mut().unwrap()
                    .write(&buffer)
                    .expect("Failed to send the message"),
                5
            );

        }

        else if self_pos == sender {

            // send data to left neigbour

            buffer[1..].copy_from_slice(&node_data.num.to_le_bytes());

            assert_eq!(
                node_data.l_stream.as_mut().unwrap()
                    .write(&buffer)
                    .expect("Failed to send the message"),
                5
            );
        }
    }
    node_data.num
}

pub fn sasaki(node_data: &Node) -> i32 {
    0
}

pub fn triplet(node_data: &Node) -> i32{
    0
}