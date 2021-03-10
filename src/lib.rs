//! A crate to count ones and xor bytes, fast (aka popcount, hamming
//! weight and hamming distance).
//!
//! # Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! mhd_mem = "0.1"
//! ```
//!
//! # Examples
//!
//! ```rust
//! use mhd_mem::mhd_method::*;
//! assert_eq!( weight(&[1, 0xFF, 1, 0xFF]), 1 + 8 + 1 + 8);
//! assert_eq!( distance(&[0xFF, 0xFF], &[1, 0xFF], &[0xFF, 1]), 7 + 7);
//! assert_eq!( Sample::default().score, ZERO_SCORE );
//! assert_eq!( Sample::new( 42 as ScoreType ).get_bit( 7 ), false );
//! ```

//   #![deny(warnings)]
// #![cfg_attr(not(test), no_std)]

extern crate hamming;
extern crate core;
extern crate rand;
extern crate rand_distr;
extern crate log;

#[cfg(test)] extern crate quickcheck;

pub mod mhd_method {

    pub mod util;

    pub mod weight_;
    pub use self::weight_::weight;

    pub mod distance_;
    pub use self::distance_::{ distance, distance_fast, truncated_distance };

    pub mod sample;
    pub use self::sample::{ NUM_BITS, NUM_BYTES, ZERO_SCORE, ScoreType, Sample };

    pub mod mhdmemory;
    pub use self::mhdmemory::{ MHDMemory };
}

pub mod mhd_optimizer {
    pub mod solution;
    pub use self::solution::{ Solution, TwoSampleSolution };

    pub mod solver;
    pub use self::solver::{ Solver };

    pub mod problem;
    pub use self::problem::Problem;

    mod examples;
    pub use self::examples::{ ProblemSubsetSum, FirstDepthFirstSolver, BestFirstSolver };

    pub mod unified_optimizer;
    pub use self::unified_optimizer::find_best_solution;

}


