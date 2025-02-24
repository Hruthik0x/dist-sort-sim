
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use clap::Parser;
use std::process::{Command, Stdio};
use utility::{CommFlags, log, Utility};
use rand::Rng;

struct Node {
    port   : u16,
    stream : TcpStream,
}

#[derive(Parser)]
#[command(version, 
    about = "Distributed sorting simulator",
    long_about = "This program simulates multiple distributed sorting algos using\n\
                  sockets and seperate processes",
    author = "Hruthik <hruthikchalamareddy.c22@iiits.in"
)]


#[command(version, 
    about = "Distributed sorting simulator - Node",
    long_about = "This program simulates multiple distributed sorting algos using\n\
                  sockets and processes, where each process emulates a node.\n\
                  This program emulates distributor.\n",
    author = "Hruthik <hruthikchalamareddy.c22@iiits.in"
)]
struct Args {

    #[arg(short, long, 
        default_value_t = 2,
        value_parser = clap::value_parser!(u8).range(1..=3),
        help = "Select your algorithm :     \n\
                \t 1.Odd Even Transposition \n\
                \t 2.Sasaki                 \n\
                \t 3.Triplet (Alternate n-1)",
    )]
    algo: u8,

    #[arg(short, long, 
        default_value_t = 1,
        value_parser = clap::value_parser!(u8).range(1..=2),
        help = "Select partial order :   \n\
                \t 1. Less than order    \n\
                \t 2. Greater than order",
    )]
    partial_order: u8,

    #[arg(short, long,
        default_value_t = String::new(),
        help = "Comma seperated numbers to sort e.g. `--nums 5,3,8,1` \n\
                (No spaces between numbers).\n\
                If nums and test both mentioned, test will be ignored"
    )]
    nums: String,

    #[arg(short, long,
        default_value_t = 500,
        help = "No.of random generated values to be used for testing.\n\
                Recommended to keep it under 2000, depending on the no.of processes\n\
                your system can handle",
    )]
    test : u16
}

fn parse_nums(inp_str:&str) -> isize{
    inp_str.trim()
           .parse::<isize>()
           .expect(&format!("Failed to parse '{}'", inp_str))
}

// gets the port number of the server hosted by the connected node
fn get_node_port (mut stream: TcpStream) -> Node{
    let mut buffer = [0u8; 15];
    match stream.read(&mut buffer) {
        Ok(bytes_read) => {
            assert_eq!(bytes_read, 3);
            assert_eq!(buffer[0], CommFlags::Report as u8);
            let port_num = u16::from_le_bytes(
                           buffer[1..3].try_into()
                           .expect(&format!("Failed to parse {:?} into u16", &buffer[1..])
            ));
            Node {
                port : port_num,
                stream,
            }
        },
        Err(e) =>  panic!("Failed to read :{}", e)
    } 
}

// generate random numbers for --test
fn gen_random_nums(count: u16) -> Vec<i32> {
    let mut rng = rand::rng();
    (0..count).map(|_| rng.random_range(1..=(count as i32))).collect()
}

// verifies if the recieved result from the nodes is correct
fn verify_results(mut input_nums:Vec<i32>, output_nums:Vec<i32>, partial_order : u8) -> bool {
    match partial_order {
        1 => input_nums.sort(),
        2 => input_nums.sort_by(|a, b| b.cmp(a)) ,
        def_val => panic!("Unexpected partial order given {}", def_val),
    };
    input_nums == output_nums
}

// Invokes all nodes with the distributor's port as an argument
fn invoke_nodes(distributor_port : u16, no_nodes : u16) {
    let node_executable = if cfg!(debug_assertions) {
        "./target/debug/node"
    } else {
        "./target/release/node"
    };

    for i in 0..no_nodes {
        let args = vec!["--dist-port".to_string(), distributor_port.to_string()];
        
        Command::new(node_executable)
            .args(&args)
            // .stdout(Stdio::inherit())
            // .stderr(Stdio::inherit())
            .stdout(Stdio::null()) 
            .stderr(Stdio::null()) 
            .spawn()
            .expect(&format!("Failed to start node process {}", i));
    }
}

// accepts incoming connections from nodes and stores their port numbers
fn accept_nodes(listener: TcpListener, node_data : &mut Vec<Node>, max_conn : u16) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                node_data.push(get_node_port(stream));
                if node_data.len() as u16 == max_conn {
                    break;
                }
            },
            Err(e) => log!("Incoming Connection failed: {}", e),
        }
    }
}

// Prepares the buffer to be sent to each node
fn prepare_order(buffer: &mut [u8], l_port : u16, r_port : u16, glb_pos : u16, 
                 num : i32, stream : &mut TcpStream){
    buffer[5..7].copy_from_slice(&l_port.to_le_bytes());
    buffer[7..9].copy_from_slice(&r_port.to_le_bytes());
    buffer[9..11].copy_from_slice(&glb_pos.to_le_bytes());
    buffer[11..15].copy_from_slice(&num.to_le_bytes());
    assert_eq!(stream.write(buffer).expect(&format!("Failed to send data")), 15);
}

// sends out the order to each node consisting its num, algo, partialorder 
// and port numbers of its neighbour nodes
fn send_order(node_data:&mut Vec<Node>, algo:u8, nums:&Vec<i32>, partial_order : u8) {
    let buffer = &mut [0u8; 15];
    buffer[1] = algo - 1;
    buffer[2] = partial_order - 1;
    buffer[3..5].copy_from_slice(&(nums.len() as u16).to_le_bytes());

    prepare_order(buffer, 0u16, node_data[1].port, 1u16, 
             nums[0], &mut node_data[0].stream);

    for i in 1..node_data.len()-1 {
        prepare_order(buffer, node_data[i-1].port, node_data[i+1].port, 
            (i+1) as u16, nums[i], &mut node_data[i].stream);
    }

    let len = node_data.len();
    prepare_order(buffer, node_data[len-2].port, 0u16, 
      len as u16, nums[len-1], &mut node_data[len-1].stream);
}

// sends start command to all nodes, to start sorting
fn send_start(node_data:&mut Vec<Node>) {
    let buffer = [CommFlags::Start as u8];
    for node in node_data { 
        assert_eq!(node.stream.write(&buffer).expect("Failed to send the msg"), 1);
    }
}

// recieves ready message from all connected nodes
fn receive_ready(node_data: &mut Vec<Node>) {
    let mut buffer = [0u8; 1];
    for node in node_data {
        match node.stream.read(&mut buffer) {
            Ok(bytes_read) => {
                assert_eq!(bytes_read, 1);
                assert_eq!(buffer[0], CommFlags::Ready as u8);
            },
            Err(e) =>  panic!("Failed to read :{}", e)
        } 
    }
}

// recieves the final number from each node
fn receive_output(node_data:&mut Vec<Node>, output_nums:&mut Vec<i32>){
    let mut buffer = [0u8; 5];
    for node in node_data {
        match node.stream.read(&mut buffer) {
            Ok(bytes_read) => {
                assert_eq!(bytes_read, 5); 
                assert_eq!(buffer[0], CommFlags::Finish as u8);
                output_nums.push(i32::from_le_bytes(buffer[1..].try_into()
                                    .expect(&format!("Failed to parse {:?} into i32", &buffer[1..]
                                ))));
            },
            Err(e) =>  panic!("Failed to read :{}", e)
        } 
    }
}

fn main() {
    let args = Args::parse();
    let input_nums:Vec<i32>;
    let no_nodes:u16;

    if args.nums.len() == 0 {
        no_nodes = args.test;
        input_nums = gen_random_nums(no_nodes);
        println!("Input nums :\n{:?}", input_nums);
    }

    else {
        input_nums = args.nums
            .split(',')
            .map(|s| parse_nums(s) as i32)
            .collect();
        no_nodes = input_nums.len() as u16;
    }


    let mut output_nums: Vec<i32>  = Vec::new();
    let mut node_data:Vec<Node> = Vec::new();
    let (listener, port) = Utility::create_server();

    println!("Algo          : {:?}\n\
              Partial order : {:?}", args.algo, args.partial_order);

    println!("=> Distributor server running on port : {}", port);
    
    invoke_nodes(port, no_nodes);
    println!("=> Nodes invoked");

    accept_nodes(listener, &mut node_data, no_nodes);
    println!("=> Nodes connected");

    send_order(&mut node_data, args.algo, &input_nums, args.partial_order);
    println!("=> Order sent to the nodes");

    receive_ready(&mut node_data);
    println!("=> All nodes ready");

    send_start(&mut node_data);
    println!("=> Sorting started");

    receive_output(&mut node_data, &mut output_nums);
    println!("Output :\n{:?}", output_nums);

    assert!(verify_results(input_nums, output_nums, args.partial_order));
}