//! A crate to apply the MHD-Memory to Optimization (NP-Hard) Problems.
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mhd_optimization = "0.0.1"
//! ```
//!
//! # Examples
//!
//! ```rust
//! use mhd_optimization::optimizer::*;
//! use mhd_optimization::implementations::*;
//!
//! const NUM_DECISIONS: usize = 4; // for a start
//!
//! let knapsack = Problem01Knapsack::random(NUM_DECISIONS); // or ProblemSubsetSum, or ... ??
//! let mut solver = BestFirstSolver::new(NUM_DECISIONS); // or DepthFirstSolver, or ... ??
//!
//! use std::time::Duration;
//! let time_limit = Duration::new(2, 0); // 2 seconds convergence time
//
//! assert!(knapsack.is_legal());
//
//! let the_best = solver.find_best_solution(&knapsack, time_limit)
//!               .expect("could not find best solution");
//! assert!(knapsack.solution_is_legal(&the_best));
//! assert!(knapsack.solution_is_complete(&the_best));
//
//! let best_score = the_best.get_score();
//! assert!(0  < best_score);
//! assert_eq!(best_score, knapsack.solution_score(&the_best));
//! ```

//   #![deny(warnings)]
// #![cfg_attr(not(test), no_std)]

extern crate core;
extern crate hamming;
extern crate log;
extern crate rand;
extern crate rand_distr;
extern crate simplelog;
extern crate structopt;

extern crate mhd_memory;

pub mod optimizer {
    pub mod solution;
    pub use self::solution::{MinimalSolution, Solution, PriorityType, };

    pub mod solver;
    pub use self::solver::Solver;

    pub mod problem;
    pub use self::problem::Problem;
}

pub mod implementations {
    pub mod subset_sum_problem;
    pub use self::subset_sum_problem::ProblemSubsetSum;

    pub mod zero_one_knapsack_problem;
    pub use self::zero_one_knapsack_problem::{Problem01Knapsack, ZeroOneKnapsackSolution};

    pub mod depth_first_solver;
    pub use self::depth_first_solver::DepthFirstSolver;

    pub mod best_first_solver;
    pub use self::best_first_solver::BestFirstSolver;

    pub mod mcts_solver;
    pub use self::mcts_solver::MonteCarloTreeSolver;

    pub mod mhd_mc_solver;
    pub use self::mhd_mc_solver::*;

    pub mod bf_mhd_mc_solver;
    pub use self::bf_mhd_mc_solver::*;

    pub mod parsers;
    pub use self::parsers::{parse_dot_csv_stream, parse_dot_dat_stream};
}
