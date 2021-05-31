use std::cmp::max;
extern crate structopt;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "knapsacks")]
struct Opt {
    // The number of occurrences of the `v/verbose` flag
    /// Verbose mode (-v or -vv)
    #[structopt(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Number of items (dimensions, choices).
    #[structopt(short, long, default_value = "42")]
    size: usize,

    /// Capacity of Knapsack
    ///
    /// Capacity is interpreted as percentage of sum of weights.
    /// Capacity must be in the range : 0 <= capacity < 100.
    /// A capacity of zero will be replaced by a customized random capacity (this is the default).
    #[structopt(short, long, default_value = "0")]
    capacity: f32,

    /// Time limit in seconds (floating point; defines convergance)
    #[structopt(short, long, default_value = "1.0")]
    time: f32,

    /// Algorithms (solvers) : 1 = depth first, 2 = best first, 4 = MCTS, 8 = MHD, 16 = BF MHD MCTS, 31 = 0x11111 = all of them ...
    #[structopt(short, long, default_value = "31")]
    algorithms: u8,

    /// Number of problems to solve
    ///
    /// If no file is given, num problems will be created with random numbers.
    /// In this case, the default is 1 (and not 1000).
    /// Note that some files have more than one problem.
    /// num_problems specifies the maximum number of problems per file,
    /// if (and only if) at least one file is specified.
    ///
    #[structopt(short, long, default_value = "1000")]
    num_problems: u16,

    /// Files to process
    ///
    /// If no file is given, problems will be created with random numbers.
    ///
    /// Known file formats:
    /// csv (Pisinger format).
    /// dat (rust crate format)
    #[structopt(name = "FILE", parse(from_os_str))]
    files: Vec<PathBuf>,
} // end struct Opt

const DEPTH_FIRST_BIT: u8 = 1;
const BEST_FIRST_BIT: u8 = 2;
const MCTS_BIT: u8 = 4;
const MHD_BIT: u8 = 8;
const BF_MHD_BIT: u8 = 16;

use std::time::{Duration, Instant};

extern crate mhd_memory;
use mhd_memory::ScoreType; // used implicitly (only)

extern crate mhd_optimization;
use mhd_optimization::implementations::BestfirstMhdMonteCarloSolver;
use mhd_optimization::implementations::{BestFirstSolver, DepthFirstSolver};
use mhd_optimization::implementations::{MhdMonteCarloSolver, MonteCarloTreeSolver};
use mhd_optimization::implementations::{Problem01Knapsack, ZeroOneKnapsackSolution};
use mhd_optimization::optimizer::{Problem, Solution, Solver};

fn run_one_problem_one_solver(
    opt: &Opt,
    knapsack: &Problem01Knapsack,
    solver: &mut impl Solver<ZeroOneKnapsackSolution>,
) -> ScoreType {
    if !knapsack.is_legal() {
        println!("Not optimizing ILLEGAL Knapsack! {:?}", knapsack);
        println!(
            "ILLEGAL Knapsack has dim {}, weights {}, capacity {}",
            knapsack.problem_size(),
            knapsack.weights_sum(),
            knapsack.capacity()
        );
        return 99999 as ScoreType;
    };

    let time_limit = Duration::from_secs_f32(opt.time);
    let start_time = Instant::now();

    let the_best = solver
        .find_best_solution(knapsack, time_limit)
        .expect("Optimization fails?!?");

    println!(
        "with {}, found best score {} in knapsack with dim {} after {:?}",
        solver.name(),
        the_best.get_score(),
        knapsack.problem_size(),
        start_time.elapsed()
    );
    info!("                          best is {}", the_best.readable());
    the_best.get_score()
}

const DFS_INDEX: usize = 0;
const BFS_INDEX: usize = 1;
const MCTS_INDEX: usize = 2;
const MCTS_MONTE_INDEX: usize = 3;
const MHD_INDEX: usize = 4;
const MHD_MONTE_INDEX: usize = 5;
const BF_MHD_SCORE: usize = 6;
const NUM_SCORES: usize = 7;

fn run_one_problem(opt: &Opt, knapsack: &mut Problem01Knapsack, prob_num: u16) {
    if 0.0 != opt.capacity {
        knapsack.basis.capacity =
            (knapsack.weights_sum() as f32 * (opt.capacity / 100.0)) as ScoreType;
    }; // else, leave capacity alone remain what the random constructor figured out.

    let mut scores = [0 as ScoreType; NUM_SCORES];

    println!(" "); // blank line seperator -> output
    if 0 != (opt.algorithms & DEPTH_FIRST_BIT) {
        print!("Knapsack {}: ", prob_num + 1);
        scores[DFS_INDEX] =
            run_one_problem_one_solver(&opt, &knapsack, &mut DepthFirstSolver::new(opt.size));
    }; // endif depth first

    if 0 != (opt.algorithms & BEST_FIRST_BIT) {
        print!("Knapsack {}: ", prob_num + 1);
        scores[BFS_INDEX] =
            run_one_problem_one_solver(&opt, &knapsack, &mut BestFirstSolver::new(opt.size));
    }; // end if best first

    if 0 != (opt.algorithms & MCTS_BIT) {
        print!("Knapsack {}: ", prob_num + 1);
        let mut solver = MonteCarloTreeSolver::builder(knapsack);
        scores[MCTS_INDEX] = run_one_problem_one_solver(&opt, &knapsack, &mut solver);

        // Do it again, but full monte
        solver.clear();
        solver.full_monte = true;
        print!("FullMonte{}: ", prob_num + 1);
        scores[MCTS_MONTE_INDEX] = run_one_problem_one_solver(&opt, &knapsack, &mut solver);
    }; // end if best first

    if 0 != (opt.algorithms & MHD_BIT) {
        let mut solver = MhdMonteCarloSolver::builder(knapsack);
        let start_size = solver.mhd_memory.num_samples();
        print!("Knapsack {}: ", prob_num + 1);
        scores[MHD_INDEX] = run_one_problem_one_solver(&opt, &knapsack, &mut solver);
        println!("SIZES: MHD had {}, has {} samples", start_size, solver.mhd_memory.num_samples());
        // Do it again, but full monte
        solver.clear();
        solver.full_monte = true;
        let start_size = solver.mhd_memory.num_samples();
        print!("MHDMonte {}: ", prob_num + 1);
        scores[MHD_MONTE_INDEX] = run_one_problem_one_solver(&opt, &knapsack, &mut solver);
        println!("SIZES: MHD had {}, has {} samples", start_size, solver.mhd_memory.num_samples());
    }; // end if best first

    if 0 != (opt.algorithms & BF_MHD_BIT) {
        print!("Knapsack {}: ", prob_num + 1);
        let mut solver = BestfirstMhdMonteCarloSolver::builder(knapsack);
        let start_size = solver.mhd_memory.num_samples();
        scores[BF_MHD_SCORE] = run_one_problem_one_solver(&opt, &knapsack, &mut solver);
        println!("SIZES: MHD had {}, has {} samples", start_size, solver.mhd_memory.num_samples());
    }; // end if best first

    let best_score = scores.iter().fold(ZERO_SCORE, |s0, s1| max(s0, *s1));
    assert_ne!(best_score, 0);
    print!("Dim {}, Best score {}, ", knapsack.problem_size(), best_score);
    let fbest: f32 = best_score as f32;
    let ratios: Vec<f32> = scores.iter().map(|s| *s as f32 / fbest).collect();
    println!("ratios {:?}", ratios);
} // end run_one_problem

fn run_one_file(opt: &Opt, file_name: &PathBuf) -> usize {
    println!("\nProcessing Filename: {:?}", file_name);
    let mut counter: usize = 0;
    let file = std::fs::File::open(file_name).unwrap();
    let mut input = io::BufReader::new(file);
    match file_name
        .extension()
        .expect("Need a file name with an extension")
        .to_str()
        .expect("Need a simple file name extension")
    {
        "dat" => {
            for prob_num in 0..opt.num_problems {
                // or end of file
                match parse_dot_dat_stream(&mut input) {
                    Err(_) => break,
                    Ok(mut knapsack) => run_one_problem(&opt, &mut knapsack, prob_num),
                }; // end match parse_dot_dat
                counter += 1;
            } // end for  problems in file
        } // end for all problems
        "csv" => {
            for prob_num in 0..opt.num_problems {
                // or end of file
                match parse_dot_csv_stream(&mut input) {
                    Err(_) => break,
                    Ok(mut knapsack) => run_one_problem(&opt, &mut knapsack, prob_num),
                }; // end match parse_dot_dat
                counter += 1;
            } // end for  problems in file
        } // end for all problems
        _ => warn!("Unknown file extension (not dat, not csv"),
    }; // end match file name extension
       // Done!
    counter
} // end run_one_file

fn run_one_directory(opt: &Opt, path: &PathBuf) -> usize {
    let mut num_tests: usize = 0;
    for entry_result in path.read_dir().expect("read_dir call failed") {
        match entry_result {
            Ok(dir_entry) => {
                num_tests += run_one_file(opt, &dir_entry.path());
            }
            Err(e) => warn!("Error {:?} in directory {:?}", e, path),
        };
    } // end for all entries in directory
      // Done!
    num_tests
} // end run_one_file

extern crate log;
extern crate simplelog;
use log::*;
use simplelog::*;
use std::fs::File;
use std::io;
// use mhd_mem::mhd_memory::ScoreType; -- already imported above
use mhd_memory::ZERO_SCORE;
use mhd_optimization::implementations::{parse_dot_csv_stream, parse_dot_dat_stream};

/********************************* MAIN *****************************/
fn main() {
    let mut opt = Opt::from_args();
    println!("{:?}\n", opt);

    assert!(0.0 <= opt.capacity, "Capacity cannot be negative");
    assert!(opt.capacity < 100.0, "Capacity cannot be 100% or greater");
    assert!(opt.verbose < 4, "Too verbose: Maximum verbosity is vvv");
    assert!(
        opt.algorithms < 32,
        "Illegal algorithm (code 32 or more not allowed)"
    );

    if 0 < opt.verbose {
        let term_level = match opt.verbose {
            1 => LevelFilter::Info,
            2 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };
        let file_level = match opt.verbose {
            1 => LevelFilter::Warn,
            2 => LevelFilter::Warn,
            _ => LevelFilter::Trace,
        };
        CombinedLogger::init(vec![
            TermLogger::new(
                term_level,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(
                file_level,
                Config::default(),
                File::create("trace.log").unwrap(),
            ),
        ])
        .unwrap();
    }; // end if verbose

    let mut num_tests: usize = 0;
    if opt.files.is_empty() {
        // FIRST USE CASE : No files, random data

        if 1000 <= opt.num_problems {
            opt.num_problems = 1;
        }
        for prob_num in 0..opt.num_problems {
            let mut knapsack = Problem01Knapsack::random(opt.size);
            run_one_problem(&opt, &mut knapsack, prob_num);
        } // for 0 <= prob_num < num_problems
        num_tests = opt.num_problems as usize;
    } else {
        // if opt.files NOT empty

        // SECOND USE CASE : No files, random data
        for file_name in opt.files.iter() {
            if file_name.is_file() {
                num_tests += run_one_file(&opt, file_name);
            } else if file_name.is_dir() {
                num_tests += run_one_directory(&opt, file_name);
            } else if !file_name.exists() {
                warn!("file name {:?} does not exist.", file_name);
            } else {
                // if file exists, but is neither file nor dir
                warn!(
                    "file name {:?} exists, but is neither a file nor a directory",
                    file_name
                );
            };
        } // end for all files
    }; // end if there are files
    println!("Finished with {} tests (total)", num_tests);
    // let geo_mean = (ratio as f64).powf(1.0 / (num_tests as f64));
    // println!(
    //     "At the end, ratio = {}, n = {}, geo mean = {}",
    //     ratio, num_tests, geo_mean
    // );
}
