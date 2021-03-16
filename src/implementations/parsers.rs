/// # Example Implementations
///
/// ## Parsers
///
/// A module full of software to read file formats and create problems
/// -- problems in the sense of the problems we want to solve,
/// or more precisely, the ones we've implemented elsewhere in this module ("implementations").

use mhd_method::sample::{ScoreType, NUM_BITS, ZERO_SCORE}; // Not used: NUM_BYTES
use mhd_optimizer::{Problem, Solver};
use mhd_optimizer::{Solution, TwoSampleSolution};
use implementations::Problem01Knapsack;

/////////// Extra File Input Methods
// (Notes to self):
//
// Known file formats : ~/src/clones/knapsack/inst/*.dat
//                      -- one problem per line, many problems per file
//                      -- solutions in ~/src/clones/knapsack/sol/*.dat
//
//                      ~/src/clones/kplib/*/*/*.kp
//                      - one problem per file; many, many files
//                      - no solutions, I believe
//
//                      ~/src/treeless-mctlsolver/Problems/Knapsack/unicauca_mps/*/*.mps
//                      -- Problems in mps format :-(
//                      ~/src/treeless-mctlsolver/Problems/Knapsack/unicauca/*/*
//                      -- Problems in simple text format
//                      -- NO file extension!
//                      -- Optima (values only) in Problems/Knapsack/unicauca/*-optimum/*
//                      -- Best solution in problem file?!?
//                      -- Where are the capacities?!?
//
//                      ~/Documents/Forschung/PraxUndForsch2021/pisinger-knapsacks/*/*.csv
//                      -- Problems in quasi free text format -- NOT COMMA seperated!
//                      -- See Readme.txt in each directory for format! (All 3 readmes identical)
//                      -- More than one problem per file
//                      -- Optima solution supplied in third column (!) ... also time for comparison

pub fn Parse