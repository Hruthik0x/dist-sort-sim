use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, 
    about = "Distributed sorting simulator",
    long_about = "This program simulates multiple distributed sorting algos using\n\
                  sockets and seperate processes",
    author = "Hruthik <hruthikchalamareddy.c22@iiits.in"
)]

struct Args {

    #[arg(short, long, 
        default_value_t = 2,
        value_parser = clap::value_parser!(u8).range(1..=3),
        help = "Select your algorithm :     \n\
                \t 1.Odd Even Transposition \n\
                \t 2.Sasaki                 \n\
                \t 3.Triplet",
    )]
    algo: u8,

    #[arg(short, long, 
        help = "Comma seperated numbers to sort e.g. `--nums 5,3,8,1`"
    )]
    nums: String,
}

fn parse_nums(inp_str:&str) -> isize{
    inp_str.trim()
           .parse::<isize>()
           .expect(&format!("Failed to parse '{}'", inp_str))
}

fn main() {
    let args = Args::parse();

    let nums: Vec<isize> = args.nums
    .split(',')
    .map(|s| parse_nums(s))
    .collect();

    println!("Args : {} {:#?}", args.algo, nums);
}