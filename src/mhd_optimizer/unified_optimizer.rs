// ## The The Unified Algorithm
///
///
/// ```rust
/// use mhd_mem::mhd_optimizer::{ Solution, TwoSampleSolution };
/// use mhd_mem::mhd_optimizer::{ Solver, FirstDepthFirstSolver };
/// ```

// use core::fmt::Debug;
use log::*; // for info, trace, warn, etc.

use ::mhd_optimizer::{ Solution };
use ::mhd_optimizer::{ Solver };
use ::mhd_optimizer::Problem;

use std::time::{Duration, Instant};

pub fn find_best_solution< Sol  : Solution,
                           Solv : Solver< Sol >,
                           Prob : Problem< Sol, Solv > >
                         ( solver     : & mut Solv,
                           problem    : & mut Prob,
                           time_limit : Duration ) -> Sol {

    let start_time = Instant::now();

    // define some solution to be "best-so-far"
    let mut num_visitations : i64 = 1;
    let mut best_solution = problem.random_solution( );

    // start at the root of the tree
    debug_assert!( solver.is_empty() );
    solver.push( problem.starting_solution( )  );
    debug_assert!( ! solver.is_empty() );

    let result = loop {

        // Terminate out if loop?
        if solver.is_empty()
            || time_limit <= start_time.elapsed( )  {
            println!( "Solver is finished! Unfinished work = {}, visitations = {}, time taken? {:?}",
                      solver.number_of_solutions(), num_visitations, start_time.elapsed( ) );
            break best_solution;
        }; // end if terminating

        let next_solution = solver.pop( ).expect("Failed to pop from non-empty solver queue?!?");
        num_visitations += 1;
        trace!( "Optimizer pops solution with score {}", problem.solution_score( & next_solution ) );

        if problem.solution_is_complete( & next_solution ) {
            if problem.better_than( & next_solution, & best_solution ) {
                best_solution = next_solution.clone();
                trace!( "Optimizer finds new BEST score {}!", problem.solution_score( & best_solution ) );
            }; // end if new solution better than old
        }; // endif next_solution has a score

        // BOUND
        if problem.can_be_better_than( & next_solution, & best_solution ) {
            // BRANCH
            problem.register_children_of( & next_solution, solver );
        }; // end if not bounded

    };// end loop

    return result;

} // end find_best_solution
