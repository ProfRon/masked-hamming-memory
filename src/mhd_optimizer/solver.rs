use std::time::{Duration, Instant};

use log::*; // for info, trace, warn, etc.
use std::error::Error;
use std::fs::OpenOptions; // and/or File, if we want to overwrite a file...
use std::io::prelude::*; // for writeln! (write_fmt)

use mhd_optimizer::{Problem, Solution};

static GLOBAL_TIME_LIMIT: Duration = Duration::from_secs(60); // can be changed

/// ## The Solver Trait
///
pub trait Solver<Sol: Solution> {
    // First, one "associated type"
    // type Sol = S;

    /// every instance of this struct should have a descriptive name (for tracing, debugging)
    /// TO DO: Remove this when <https://doc.rust-lang.org/std/any/fn.type_name_of_val.html> stable
    fn name(&self) -> &'static str;

    /// Every instance should have a SHORT description for Debugging,
    /// giving things like the number of solutions in the container, etc.
    fn short_description(&self) -> String;

    // Constructors

    /// Constructor for a "blank" solution (with no decisions made yet) where
    /// size is the number of decisions to be made (free variables to assign values to).
    fn new(size: usize) -> Self;

    // Methods used by the Unified Optimization Algorithm (identified above)

    /// Number of solutions stored in this container
    fn number_of_solutions(&self) -> usize;

    fn is_empty(&self) -> bool {
        0 == self.number_of_solutions()
    }

    /// Discard any solutions currently stored in container
    fn clear(&mut self); // empty out (if not already empty) like std::vec::clear()

    /// Add one incomplete solution to container -- the main difference between each implementation!
    /// This is a very important difference between the various implementations!
    fn push(&mut self, solution: Sol);

    /// Remove one solution from container (if possible)
    /// This is a very important difference between the various implementations!
    fn pop(&mut self) -> Option<Sol>;

    /// Each Solver should also note the best solution seen so far.
    /// This requirement leads to the next two methods: a getter and a setter -- and a fancier setter.
    ///
    /// The getter is simple: `best_solution` return the best solution see so far.
    fn best_solution(&self) -> &Sol;

    /// Store new best solution. Note, we take caller's word for it. Solution is not (re)tested.
    fn store_best_solution(&mut self, sol: Sol);

    /// The next method looks at a complete solution and, if it is the best, remembers it
    /// (at the very least -- some form of "machine learning" may also take place).
    /// Every complete solution see so far should be sent through this method.
    /// `new_best_solution` returns TRUE iff this was the best soluton see so far.
    fn new_best_solution<Prob: Problem<Sol = Sol>>(
        &mut self,
        problem: &Prob,
        solution: Sol,
    ) -> bool {
        debug_assert!(problem.solution_is_complete(&solution));

        let result = problem.better_than(&solution, self.best_solution());
        if  result { // i.e. if solution is better than best_solution
            // record new best solution.
            self.store_best_solution(solution);

            // record new best solution as trace and as a line in trace.csv
            debug!(
                "Optimizer finds new BEST score {}!",
                self.best_solution().get_score(),
            );
        }; // end solution  better than old best solution
        result // return result!!
    } // end accept_solution

    /*******************************************************************************/
    /// This is the crux of this whole project: The `find_best_solution` method.
    /// It does what it says here.
    /// Originally outside this (Problem) Trait, but the compiler is making this difficult...
    fn find_best_solution<Prob: Problem<Sol = Sol>>(
        &mut self,
        problem: &Prob,
        time_limit: Duration,
    ) -> Result<Sol, Box<dyn Error>> {
        let global_start_time = Instant::now();
        let mut start_time = Instant::now();

        // The best solution is currently defined, and randomized, but wrong.
        // Do it right.
        self.store_best_solution(problem.random_solution());

        // define some solution to be "best-so-far"
        let mut num_visitations: i64 = 0;
        debug_assert!(problem.solution_is_complete(self.best_solution()));
        debug_assert!(problem.solution_is_legal(self.best_solution()));
        info!("Optimizing Problem {}", problem.short_description());
        debug!(
            "First Random Solution (short) {}",
            self.best_solution().short_description()
        );
        trace!("; visits; depth; score; complete; high score;");
        trace!(
            "; {}; {}; {}; {}; {};",
            num_visitations,
            problem.problem_size(),
            self.best_solution().get_score(),
            true, // by definition
            self.best_solution().get_score()
        );

        // start at the root of the tree
        debug_assert!(self.is_empty());
        self.push(problem.starting_solution());

        loop {
            num_visitations += 1;

            let next_solution = self
                .pop()
                .expect("solver's queue should not be empty but could not pop");
            debug_assert!(!problem.solution_is_complete(&next_solution));
            // Compare above: ""; visits; depth; score; complete; high score;"");
            trace!(
                "; {}; {}; {}; {}; {};",
                num_visitations,
                problem.first_open_decision(&next_solution).unwrap(),
                next_solution.get_score(),
                problem.solution_is_complete(&next_solution),
                self.best_solution().get_score()
            );

            debug_assert!(!problem.solution_is_complete(&next_solution));

            // BOUND
            if problem.can_be_better_than(&next_solution, self.best_solution()) {
                // BRANCH
                let children = problem.children_of_solution(&next_solution);
                for child in children {
                    if !problem.solution_is_complete(&child) {
                        // child is incomplete
                        if problem.can_be_better_than(&child, self.best_solution()) {
                            self.push(child); // clone because rustc says so...
                        }
                    } else {
                        // if solution IS complete
                        // Compare above: "; depth; score; complete; high score;");
                        trace!(
                            "; {}; {}; {}; {}; {};",
                            num_visitations,
                            problem.problem_size(),
                            child.get_score(),
                            true, // by definition
                            self.best_solution().get_score()
                        );
                        // Learn the new complete solution, and test if it is the best so far
                        if self.new_best_solution(problem, child) {
                            // Reset timer!
                            // That means we have converted if we go for time_limit without a new best solution!
                            start_time = Instant::now();
                        }
                    } // end if complete
                } // end for 0, 1 or 2 children
            }; // end if not bounded

            // Terminate out if loop?
            if self.is_empty()
                || (time_limit < start_time.elapsed())
                || (GLOBAL_TIME_LIMIT < global_start_time.elapsed())
            {
                break;
            }; // end if terminating
        } // end loop

        // Done. Take a deep breath, print debug print, then return result.

        let result = self.best_solution();

        let mut macrotrace_file = OpenOptions::new()
            .append(true)
            .create(true)
            .open("macrotrace.csv")
            .expect("Could not open macrotrace.csv");
        writeln!(
            macrotrace_file,
            "\"{}\", \"{}\", \"{}\", {}; {}; {}; {}; {}", // EIGHT fields!
            result.name(),
            self.name(),
            problem.name(),
            start_time.elapsed().as_nanos(),
            num_visitations,
            self.number_of_solutions(),
            result.get_score(),
            result.get_best_score(),
        )?;

        debug!("Optimizer find best solution in {:?}", problem);
        debug!("Optimizer converges on soution {:?}", result);
        info!("Optimizer find best score {}", result.get_score());

        Ok(result.clone())
    } // end default find_best_solution implementation
} // end Solver Problem
