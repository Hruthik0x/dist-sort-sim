
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::panic;
use clap::Parser;
use std::process::{Command, Stdio};
use utility::{CommFlags, Utility};
use std::fs::{File, create_dir_all};
use rand::Rng;
use std::path::Path;
use std::time::Instant;
use std::collections::BTreeMap;

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
        help = "Important note :\n\
                    \tIf you mnetion --sasaki or --odd-even or --alternate in the\n\
                    \tcommand line args when running ./simulate.sh -a will be ignored\n\
                Select your algorithm :     \n\
                \t 1.Odd Even Transposition \n\
                \t 2.Sasaki                 \n\
                \t 3.Alternate",
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
        default_value_t = 50,
        help = "No.of random generated values to be used for testing.\n\
                Recommended to keep it under 2000, depending on the no.of processes\n\
                your system can handle",
    )]
    test : u16,

    #[arg(short, long,
        help = "Run a comparison among algorithms and report it in a table",
        action = clap::ArgAction::SetTrue,
    )]
    comp : bool,
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

fn print_timing_table(timings: &Vec<(u16, u8, u128)>) {
    // Map of no_nums → [algo1_time, algo2_time, algo3_time]
    let mut table: BTreeMap<u16, [Option<u128>; 3]> = BTreeMap::new();

    for (n, a, t) in timings {
        let entry = table.entry(*n).or_insert([None; 3]);
        if *a >= 1 && *a <= 3 {
            entry[(*a as usize) - 1] = Some(*t);
        }
    }

    // Print header
    println!("{:<10} | {:^10} | {:^10} | {:^10}", "Num", "Odd-Even", "Sasaki", "Alternate");
    println!("{}", "-".repeat(47));

    // Print each row
    for (n, times) in table {
        println!("{:<10} | {:^10} | {:^10} | {:^10}",
            n,
            times[0].map_or("-".to_string(), |t| format!("{} ms", t)),
            times[1].map_or("-".to_string(), |t| format!("{} ms", t)),
            times[2].map_or("-".to_string(), |t| format!("{} ms", t)),
        );
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
fn invoke_nodes(distributor_port : u16, no_nodes : u16, algo_string: &str) {
    let node_executable = if cfg!(debug_assertions) {
        "./target/debug/node"
    } else {
        "./target/release/node"
    };

    let log_dir = Path::new("./logs");
    create_dir_all(log_dir).expect("Failed to create logs directory");

    for i in 0..no_nodes {
        let args = vec!["--dist-port".to_string(), distributor_port.to_string()];

        let log_path = log_dir.join(format!("{}_node_{}.log", algo_string, i));
        let log_file = File::create(&log_path).expect("Failed to create log file");

        Command::new(node_executable)
            .args(&args)

            // Forwarding output to log files
            .stdout(Stdio::from(log_file.try_clone().expect("Failed to clone log file")))
            .stderr(Stdio::from(log_file))
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
            Err(e) => println!("Incoming Connection failed: {}", e),
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

    // Comoparision table
    if args.comp {
        let dist_executable = if cfg!(debug_assertions) {
            "./target/debug/distributor"
        } else {
            "./target/release/distributor"
        };
    
        let mut timings: Vec<(u16, u8, u128)> = vec![]; // (no_nums, algo, time in ms)
    
        for no_nums in 1u16..6u16 {
            let nums = gen_random_nums(no_nums * 10)
                .iter()
                .map(|n| n.to_string())
                .collect::<Vec<String>>()
                .join(",");
    
            let mut args = vec!["-n".to_string(), nums, "-a".to_string()];
    
            for algo in 1u8..4u8 {
                args.push(algo.to_string());
    
                // Start timing
                let start = Instant::now();
    
                let status = Command::new(dist_executable)
                    .args(&args)
                    .status()
                    .expect("Failed to run distributor");
    
                let elapsed = start.elapsed().as_millis();
    
                if status.success() {
                    timings.push((no_nums * 10, algo, elapsed));
                } else {
                    eprintln!("Execution failed for algo {} with {} numbers", algo, no_nums * 10);
                }
    
                // Remove the last argument (algo) for next iteration
                args.pop();
            }
        }
        print_timing_table(&timings);
    } 
    else {
        if args.nums.len() == 0 {
            no_nodes = args.test;
            input_nums = gen_random_nums(no_nodes);
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

        //Creating socket server for the nodes to connect to the distributor
        let (listener, port) = Utility::create_server();

        let algo_string  = match args.algo {
            1 => "Odd-Even", 
            2 => "Sasaki",
            3 => "Alternate",
            _ => panic!("Should be between 1 and 3 inclusive"),
        };

        let partial_order_string = match args.partial_order {
            1 => "Less than",
            2 => "Greater than",
            _ => panic!("Should be between 1 and 2 inclusive"),
        };

        println!("Algo          : {:?}", algo_string);
        println!("Partial order : {:?}", partial_order_string);
        println!("Num Count     : {:?}", input_nums.len());

        println!("Input nums    :\n{:?}\n\n", input_nums);

        println!("=> Distributor server running on port : {}", port);
        
        // Invoking the nodes
        invoke_nodes(port, no_nodes, algo_string);
        println!("=> Nodes invoked");

        // Accept connections from the nodes
        accept_nodes(listener, &mut node_data, no_nodes);
        println!("=> All Nodes connected to distributor");

        // Send each node its neighbour port numbers and the number its assigned
        send_order(&mut node_data, args.algo, &input_nums, args.partial_order);
        println!("=> Order sent to the nodes");

        // Receive the numbers fro hte nodes
        receive_output(&mut node_data, &mut output_nums);
        println!("\n\nOutput :\n{:?}", output_nums);

        println!("\nLogs of all the nodes stored at ./logs\n");

        // Locally sort the numbers and verify if the received results from the nodes are correct
        assert!(verify_results(input_nums, output_nums, args.partial_order));

        println!("--------------- End of output ---------------\n\n");
    }
}