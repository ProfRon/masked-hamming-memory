use implementations::Problem01Knapsack;
/// # Example Implementations
///
/// ## Parsers
///
/// A module full of software to read file formats and create problems
/// -- problems in the sense of the problems we want to solve,
/// or more precisely, the ones we've implemented elsewhere in this module ("implementations").
use mhd_method::sample::ScoreType; // Not used: NUM_BYTES
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

// The file format is as follows:
// >    Format of instance file lines (fields)
// >    0 = ID    [parsed but discarded!]
// >    1 = the number of items
// >    2 =  the knapsack capacity
// >    then sequence of weight-cost pairs, so...
// >    3 = weight 0
// >    4 = cost 0
// >    5 = weight 1
// >    6 = cost 1
// >    ... and so on
pub fn parse_dot_dat_stream<R: io::BufRead>(mut input: R) -> io::Result<Problem01Knapsack> {
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
}

/// This parser reads one problem from a "dot csv" file -- taken to be in "Pisinger format,
/// where each file contains 1ßß knapsack problems --
/// and returns one problem -- or nothing, if no problem could be read.

// The file format is as follows:
// >   The format of each instance is
// >
// >   instance-name            <-- A string
// >   n <int>                  <-- i.e. the char n and a blank and then the dimension
// >   c <int>                  <-- i.e. the char c and a blank and then the capacity
// >   z <int>                  <-- i.e. the char z and a blank and then ?!? (Ziel?!?)
// >   time <float>             <-- i.e. the 4 chars "time" and a blank and then a runtime (secs)
// >   1 p[1] w[1] x[1]         <-- i.e. 4 ints, blank separated,
// >   2 p[2] w[2] x[2]         <-- where p = profit, w = weight and x = solution
// >   :
// >  n p[n] w[n] x[n]
// >  -----                     <-- 5 dashes
// >  (and then a blank line)

// extern crate log;
// use log::*;

pub fn parse_dot_csv_stream<R: io::BufRead>(mut input: R) -> io::Result<Problem01Knapsack> {
    // Line 1 = instance-name
    let mut line = String::new();
    trace!("Parser - ID line 1={}", line);
    let mut tokens: Vec<String> = Vec::new();
    // skip blank lines until we get a non-blank line...
    while tokens.is_empty() {
        line.clear(); // to prevent next  read_line appending on old one...
        // Note that read_line return Ok(0) on EOF
        let num_bytes_read = input.read_line(&mut line)? ;
        if 0 == num_bytes_read {
            return Err(io::Error::new(io::ErrorKind::InvalidData,"End of File? Empty Line".to_string()));
        }; // end if we read zero bytes
        // else we have a string
        tokens = line.split_whitespace().map(|tok| tok.to_owned()).collect();
    }; // end while tokens empty
    assert_eq!(1, tokens.len(), "Expected problem name (only)"); // Discard name if OK

    // line 2 = n <int>
    line.clear();  // make a new line, because...
    input.read_line(&mut line)?; // ...this appends the new line otherwise.
    trace!("Parser - n int line 2 ={}", line);
    tokens = line.split_whitespace().map(|tok| tok.to_owned()).collect();
    assert_eq!(2, tokens.len(), "expected n <int> (only)");
    assert_eq!("n", tokens[0], "expected n <int> (did not find n)");
    let size: usize = tokens[1].parse().expect("Expect dimension");

    // line 3 = c <int>
    let mut line = String::new();
    input.read_line(&mut line)?;
    trace!("Parser - c int line 3 ={}", line);
    tokens = line.split_whitespace().map(|tok| tok.to_owned()).collect();
    assert_eq!(2, tokens.len(), "expected c <int> (only)");
    assert_eq!("c", tokens[0], "expected c <int> (did not find c)");
    let capacity: ScoreType = tokens[1].parse().expect("Expect dimension");

    // line 4 = z <int>
    let mut line = String::new();
    input.read_line(&mut line)?;
    trace!("Parser - z int line 4 ={}", line);
    tokens = line.split_whitespace().map(|tok| tok.to_owned()).collect();
    assert_eq!(2, tokens.len(), "expected c <int> (only)");
    assert_eq!("z", tokens[0], "expected z <int> (did not find z)");
    let goal: ScoreType = tokens[1].parse().expect("Expect Ziel (goal score)");
    println!("                                        GOAL = {}", goal);

    // line 5 = time <int>
    let mut line = String::new();
    input.read_line(&mut line)?;
    trace!("Parser - time int line 5 ={}", line);
    tokens = line.split_whitespace().map(|tok| tok.to_owned()).collect();
    assert_eq!(2, tokens.len(), "expected time <int> (only)");
    assert_eq!("time", tokens[0], "expected time <int> (did not find time)");

    // We can now build the result object...
    let mut result = Problem01Knapsack::new(size);
    result.basis.capacity = capacity;
    let mut reference = vec![255u8; size];

    // lines 6, 7, ...
    for dim in 0..size {
        let mut line = String::new();
        input.read_line(&mut line)?;
        trace!("Parser - data line ={}", line);
        tokens = line.split(',').map(|tok| tok.to_owned()).collect();
        assert_eq!(
            4,
            tokens.len(),
            "expected <index>,<profit>,<weight>,<solution> (only)"
        );

        let index: usize = tokens[0].parse().expect("Expect index");
        assert_eq!(index - 1, dim, "read index =/= dim");
        result.values[dim] = tokens[1].parse().expect("Expect profit");
        result.basis.weights[dim] = tokens[2].parse().expect("Expect weight");
        tokens[3].pop(); // line.split leaves a \n on last token!?!?!
        let dummy: u8 = tokens[3].parse().expect("Expect Solution count (0|1)");
        reference[dim] = dummy;
    } // for 0 <= dim < size

    // Next to last line = "-----\n"
    const DASHES: &str = "-----\n";
    let mut line = String::new();
    input.read_line(&mut line)?;
    assert_eq!(line, DASHES, "expected 5 dashes");

    // Last line should be blank, but that will be skipped above

    trace!(" About to return Knapsack {:?} ", result);
    info!("Reference Solution (score {}) = {:?}", goal, reference);
    return Ok(result);
}
