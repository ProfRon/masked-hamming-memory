use std::error::Error;
/// ## The Problem Trait
///
use std::time::{Duration, Instant};

use log::*; // for info, trace, warn, etc.
use std::fs::OpenOptions; // and/or File, if we want to overwrite a file...
use std::io::prelude::*; // for writeln! (write_fmt)

use mhd_optimizer::{Problem, Solution, Solver};

static GLOBAL_TIME_LIMIT: Duration = Duration::from_secs(60); // can be changed

/*******************************************************************************/
/// This is the crux of this whole project: The `find_best_solution` method.
/// It does what it says here.
/// Originally outside this (Problem) Trait, but the compiler is making this difficult...
pub fn find_best_solution<Sol: Solution, Solv: Solver<Sol>, Prob: Problem<Sol = Sol>>(
    problem: &Prob,
    solver: &mut Solv,
    time_limit: Duration,
) -> Result<Sol, Box<dyn Error>> {
    let global_start_time = Instant::now();
    let mut start_time = Instant::now();

    // define some solution to be "best-so-far"
    let mut num_visitations: i64 = 0;
    let mut best_solution = problem.random_solution();
    debug_assert!(problem.solution_is_complete(&best_solution));
    debug_assert!(problem.solution_is_legal(&best_solution));
    trace!("Optimizing Problem {}", problem.short_description());
    trace!(
        "First Random Solution (short) {}",
        best_solution.short_description()
    );

    // start at the root of the tree
    debug_assert!(solver.is_empty());
    solver.push(problem.starting_solution());

    let result = loop {
        num_visitations += 1;

        let next_solution = solver
            .pop()
            .expect("solver's queue should not be empty but could not pop");

        trace!(
            "Optimizer pops {} solution {} at depth {}, high score {}",
            if problem.solution_is_complete(&next_solution) {
                "  COMPLETE"
            } else {
                "incomplete"
            },
            next_solution.short_description(),
            problem
                .first_open_decision(&next_solution)
                .unwrap_or(99999999),
            best_solution.get_score()
        );

        debug_assert!( ! problem.solution_is_complete( &next_solution));

        // BOUND
        if problem.can_be_better_than(&next_solution, &best_solution) {
            // BRANCH
            let children = problem.children_of_solution(&next_solution);
            for child in children {
                if !problem.solution_is_complete(&child) {
                    // child is incomplete
                    if problem.can_be_better_than(&child, &best_solution) {
                        solver.push(child ); // clone because rustc says so...
                    }
                } else { // if solution IS complete
                    if problem.better_than(&child, &best_solution) {
                        // record new best solution.
                        best_solution = child;
                        // record new best solution as trace and as a line in trace.csv
                        trace!(
                            "Optimizer finds new BEST score {}! after {} visitations",
                            best_solution.get_score(),
                            num_visitations
                        );
                        // Reset timer!
                        // That means we exit if we go for time_limit without a new best solution!
                        start_time = Instant::now();
                    }; // end solution  better than old best solution
                } // end if complete
            } // end for 0, 1 or 2 children
        }; // end if not bounded

        // Terminate out if loop?
        if solver.is_empty()
            || time_limit < start_time.elapsed()
            || GLOBAL_TIME_LIMIT < global_start_time.elapsed()
        {
            break best_solution;
        }; // end if terminating
    }; // end loop

    let mut macrotrace_file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("macrotrace.csv")
        .expect("Could not open macrotrace.csv");
    writeln!(
        macrotrace_file,
        "\"{}\", \"{}\", \"{}\", {}; {}; {}; {}; {}", // EIGHT fields!
        result.name(),
        solver.name(),
        problem.name(),
        start_time.elapsed().as_nanos(),
        num_visitations,
        solver.number_of_solutions(),
        result.get_score(),
        result.get_best_score(),
    )?;

    trace!("Optimizer find best solution in {:?}", problem);
    trace!("Optimizer converges on soution {:?}", result);

    Ok(result)
} // end default find_best_solution implementation
