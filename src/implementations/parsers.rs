use implementations::Problem01Knapsack;
/// # Example Implementations
///
/// ## Parsers
///
/// A module full of software to read file formats and create problems
/// -- problems in the sense of the problems we want to solve,
/// or more precisely, the ones we've implemented elsewhere in this module ("implementations").
use mhd_method::sample::{ScoreType, NUM_BITS}; // Not used: NUM_BYTES
use mhd_optimizer::Problem;

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

// use std::error::Error;
use log::*;
use std::io;

/// This parser reads one line from a "dot dat" file -- since each line is a problem --
/// and returns one problem -- or nothing, if no problem could be read.
/// The file format is as follows:
/// ```
/// Format of instance file lines (fields)
/// 0 = ID    [discarded!]
/// 1 = the number of items
/// 2 =  the knapsack capacity
/// then sequence of weight-cost pairs, so...
/// 3 = weight[0]
/// 4 = cost[0]
/// 5 = weight[1]
/// 6 = cost[2]
/// ```
pub fn parse_dot_dat_string<R: io::BufRead>(mut input: R) -> io::Result<Problem01Knapsack> {
    debug!("At start of parse . dat file...");
    loop {
        let mut line = String::new();
        input.read_line(&mut line)?;
        debug!("Parser read line {}", line);
        // Split line into tokens
        let tokens: Vec<String> = line.split_whitespace().map(|tok| tok.to_owned()).collect();
        let num_tokens = tokens.len();
        debug!(
            "Parser split line into {} tokens (empty? {})",
            num_tokens,
            tokens.is_empty()
        );
        if tokens.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "End of File? Empty Line".to_string(),
            ));
        } else {
            // if not empty
            assert!(
                (7 <= num_tokens) && (1 == num_tokens % 2),
                "Illegal number of token"
            );
            let id: usize = tokens[0].parse().expect("Expect id");
            let size: usize = tokens[1].parse().expect("Expect dimension");
            let capacity: ScoreType = tokens[2].parse().expect("Expect capacity");
            assert!(size < NUM_BITS);
            assert!(2 * size + 3 == num_tokens);
            debug!(
                " Parsing Knapsack id {}, size {}, capacity {}:",
                id, size, capacity
            );
            // id now not an unused variable, I hope...
            let mut result = Problem01Knapsack::new(size);
            result.basis.capacity = capacity;
            for dim in 0..size {
                result.basis.weights[dim] = tokens[3 + 2 * dim].parse().expect("Expect weight");
                result.values[dim] = tokens[4 + 2 * dim].parse().expect("Expect cost");
                debug!(
                    " Just parsed dim {}, weight {}, cost {} ",
                    dim, result.basis.weights[dim], result.values[dim]
                );
            } // end loop over weight-cost pairs
            debug!(" About to return Knapsack {:?} ", result);
            return Ok(result);
        } // end if non-empty line
    } // look over multiple lines
}
