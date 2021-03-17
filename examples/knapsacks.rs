extern crate structopt;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "knapsacks")]
struct Opt {
    /// To Trace or Not to Trace
    #[structopt(short, long)]
    verbose: bool,

    /// Number of items (dimensions, choices)
    #[structopt(short, long, default_value = "256")]
    size: usize,

    /// Capacity of Knapsack
    ///
    /// Capacity is interpreted as percentage of sum of weights.
    /// Capacity must be in the range : 0 <= capacity < 100.
    /// A capacity of zero will be replaced by a customized random capacity (this is the default).
    ///
    #[structopt(short, long, default_value = "0")]
    capacity: f32,

    /// Time limit in seconds (floating point; defines convergance)
    #[structopt(short, long, default_value = "1.0")]
    time: f32,

    /// Algorithms (solvers) : 1 = depth first, 2 = best first, 3 = both
    #[structopt(short, long, default_value = "3", possible_values=&["1","2","3"])]
    algorithms: u8,

    /// Number of problems to solve
    ///
    /// If no file is given, num problems will be created with random numbers.
    ///
    /// Note that some files have more than one problem.
    /// num_problems specifies the maximum number of problems per file,
    /// if (and only if) at least one file is specified.
    ///
    #[structopt(short, long, default_value = "1")]
    num_problems: u16,

    /// Files to process
    ///
    /// If no file is given, problems will be created with random numbers.
    ///
    /// Known file formats:
    /// csv (Pisinger format).
    ///
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
} // end struct Opt

const DEPTH_FIRST_BIT: u8 = 1;
const BEST_FIRST_BIT: u8 = 2;

use std::time::{Duration, Instant};

extern crate mhd_mem;
// use mhd_mem::mhd_method::sample::{ ScoreType }; // used implicitly (only)
use mhd_mem::mhd_optimizer::{ Solution, Solver, Problem};
use mhd_mem::implementations::{ DepthFirstSolver, BestFirstSolver };
use mhd_mem::implementations::{Problem01Knapsack, ZeroOneKnapsackSolution};


fn test_one_problem(
    opt: &Opt,
    knapsack: &mut Problem01Knapsack,
    solver: &mut impl Solver<ZeroOneKnapsackSolution>,
) {
    assert!(0.0 <= opt.capacity, "Capacity cannot be negative");
    assert!(opt.capacity < 100.0, "Capacity cannot be 100% or greater");
    if 0.0 != opt.capacity {
        knapsack.basis.capacity = (knapsack.weights_sum() as f32 * (opt.capacity / 100.0)) as i32;
    }; // else, leave capacity alone remain what the random constructor figured out.

    if !knapsack.is_legal() {
        println!("Not optimizing ILLEGAL Knapsack!");
        return;
    };

    let time_limit = Duration::from_secs_f32(opt.time);
    let start_time = Instant::now();

    let the_best = knapsack
        .find_best_solution(solver, time_limit)
        .expect("Optimization fails?!?");

    println!(
        "with {}, found best solution = {} in knapsack {} after {:?}",
        solver.name(),
        the_best.short_description(),
        knapsack.short_description(),
        start_time.elapsed()
    );
}

extern crate log;
extern crate simplelog;
use log::*;
use simplelog::*;
use std::fs::File;

fn main() {
    let opt = Opt::from_args();
    println!("{:?}\n", opt);

    if opt.verbose {
        CombinedLogger::init(vec![
            TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed),
            WriteLogger::new(
                LevelFilter::Trace,
                Config::default(),
                File::create("trace.log").unwrap(),
            ),
        ])
        .unwrap();
    }; // end if verbose

    if opt.files.is_empty() {
        // FIRST USE CASE : No files, random data

        for prob_num in 0..opt.num_problems {
            let mut knapsack = Problem01Knapsack::random(opt.size);
            if 0 != (opt.algorithms & DEPTH_FIRST_BIT) {
                print!("Knapsack {}: ", prob_num + 1);
                test_one_problem(&opt, &mut knapsack, &mut DepthFirstSolver::new(opt.size));
            }; // endif depth first
            if 0 != (opt.algorithms & BEST_FIRST_BIT) {
                print!("Knapsack {}: ", prob_num + 1);
                test_one_problem(&opt, &mut knapsack, &mut BestFirstSolver::new(opt.size));
            }; // end if best first
        } // for 0 <= prob_num < num_problems
    } else {
        // if opt.files NOT empty

        // SECOND USE CASE : No files, random data

        for file_name in opt.files.iter() {
            println!("Processing Filename: {:?}", file_name);
        }
    }
}
