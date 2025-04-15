use clap::Parser;
use std::io::{Read, Write};

use utility::{CommFlags, Utility};

mod node_utils;
mod algos;

use node_utils::Distributor;

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
    let (listener, self_port_num) = Utility::create_server();

    // Max 15 used by Order (Order sent by the distributor)
    let mut buffer = [0u8; 15];
    let mut stream = Utility::connect_to_server(Args::parse().dist_port);
    let mut node_data;

    println!("Important note : Please use 'global position' in the logs
              \t Do not rely on node_{{id}} at the end of the file name
              \t for determining the position of the node\n");
    println!("Connected to distributor");

    // report back to the distributor, informing the node is 
    // ready and send its port num
    Distributor::report(self_port_num, &mut stream);

    // Receiving the order
    match stream.read(&mut buffer) {
        Ok(_) => {
            let cmd = buffer[0];
            match cmd {
                // First byte of the messge as CpmmFlags::Order 
                // indicates its an order from the distributor
                cmd if cmd == CommFlags::Order as u8 => node_data = 

                    // parses through the given order
                    Distributor::handle_order(&buffer[1..], listener) ,

                    def_val => panic!("Invalid command : {}", def_val),
            };
        }
        Err(e) => panic!("Failed to read : {}", e),
    }

    // Setting first byte as CommFlags::Finish, indicating the final result 
    // before sending it back to the distributor
    buffer[0] = CommFlags::Finish as u8;

    // The next 4 bytes are the final integer
    buffer[1..5].copy_from_slice(
            // Copies this output to `buffer`
            &Distributor::start_sorting(&mut node_data).to_le_bytes()
    );

    assert_eq!(stream.write(&buffer[..5]).expect("Failed to send msg"), 5);
}