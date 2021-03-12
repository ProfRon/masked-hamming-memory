extern crate docopt;
use docopt::Docopt;
extern crate serde;
use serde::Deserialize;

const USAGE: &str = "
Usage: knapsacks [--size=<n>] [--capacity=<c>] [--time=<t>] [--algo=<a>] [<problem_filename>]...
       knapsacks --help

Options:
    -s, --size=<n>       Number of items (dimensions, choices) [default: 16 ]
    -c, --capacity=<n>   Capacity of Knapsack [default: 0 ]
    -t, --time=<t>       Time limit in seconds (per optimization run) [default: 2  ]
    -a, --algo=<a>       Algorithm (solver) : 1 = depth first, 2 = best first, 3 = both [default: 3]
    -h, --help           Display this message

A capacity of zero will be replaced by a customized random capacity (this is the default).

";

#[derive(Debug, Deserialize)]
pub struct Args {
    flag_size            : usize,
    flag_capacity        : usize,
    flag_time            : usize,
    flag_algo            : usize,
    arg_problem_filename : Vec< String >
}

// extern crate mhd_mem;
// use mhd_mem::mhd_method::sample::{Sample, ScoreType, NUM_BITS, ZERO_SCORE }; // Not used: NUM_BYTES
//
// use mhd_mem::mhd_optimizer::{ Solution, TwoSampleSolution, Solver, Problem };
//use mhd_mem::mhd_optimizer::{  };

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    assert!( 1 <= args.flag_algo, "Algorithm arguments must be at least 1" );
    assert!( args.flag_algo <= 3, "Algorithm arguments cannot be greater than 3" );

    println!("{:?}", args);
}