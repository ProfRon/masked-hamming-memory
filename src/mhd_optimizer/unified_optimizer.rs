// ## The The Unified Algorithm
///
///


// use core::fmt::Debug;
use log::*; // for info, trace, warn, etc.

use ::mhd_optimizer::{ Solution };
use ::mhd_optimizer::{ Solver };
use ::mhd_optimizer::Problem;

use std::time::{Duration, Instant};
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;

pub fn find_best_solution< Sol  : Solution,
                           Solv : Solver< Sol >,
                           Prob : Problem< Sol > >
                         ( solver     : & mut Solv,
                           problem    : & Prob,
                           time_limit : Duration ) -> Result< Sol, Box<dyn Error> >{

    let filename = "trace.csv";
    let mut csv_file = File::create( filename )?;

    writeln!( csv_file, "time; visitations; queue size; score; upper bound" )?; // FIVE fields!

    let start_time = Instant::now();

    // define some solution to be "best-so-far"
    let mut num_visitations : i64 = 1;
    let mut best_solution = problem.random_solution( );
    trace!( "Optimizer initializes BEST score {}! after {} visitations",
            best_solution.get_score(), num_visitations );
    writeln!( csv_file, "{}; {}; {}; {}; {}" , // FIVE fields!
                          start_time.elapsed().as_nanos(),
                          num_visitations,
                          solver.number_of_solutions(),
                          best_solution.get_score(),
                          best_solution.get_best_score() )?;

    // start at the root of the tree
    debug_assert!( solver.is_empty() );
    solver.push( problem.starting_solution( )  );

    let result = loop {

        num_visitations += 1;

        let next_solution = solver.pop( ).expect( "solver's queue should not be empty but could not pop");
        trace!( "Optimizer pops solution with score {} after {} visitations",
                next_solution.get_score(), num_visitations );

        if problem.solution_is_complete( & next_solution ) {
            if problem.better_than( & next_solution, & best_solution ) {
                best_solution = next_solution.clone();
                // record new best solution as trace and as a line in trace.csv
                trace!( "Optimizer finds new BEST score {}! after {} visitations",
                        best_solution.get_score(), num_visitations );
                writeln!( csv_file, "{}; {}; {}; {}; {}" , // FIVE fields!
                          start_time.elapsed().as_nanos(),
                          num_visitations,
                          solver.number_of_solutions(),
                          best_solution.get_score(),
                          best_solution.get_best_score() )?;
            }; // end if new solution better than old
        }; // endif next_solution has a score

        // BOUND
        if problem.can_be_better_than( & next_solution, & best_solution ) {
            // BRANCH
            problem.register_children_of( & next_solution, solver );
        }; // end if not bounded

        // Terminate out if loop?
        if solver.is_empty()
            || time_limit <= start_time.elapsed( )  {
            info!( "Solver is finished! Unfinished work = {}, visitations = {}, time taken? {:?}",
                   solver.number_of_solutions(), num_visitations, start_time.elapsed( ) );
            writeln!( csv_file, "{}; {}; {}; {}; {}" , // FIVE fields!
                      start_time.elapsed().as_nanos(),
                      num_visitations,
                      solver.number_of_solutions(),
                      best_solution.get_score(),
                      best_solution.get_best_score(),
                    )?;
            break best_solution;
        }; // end if terminating

    };// end loop

    return Ok( result );

} // end find_best_solution
