use std::error::Error;
/// ## The Problem Trait
///
use std::fmt::Debug;
use std::time::Duration;

use mhd_method::ScoreType; // Not used: NUM_BYTES
use mhd_optimizer::Solution;
use mhd_optimizer::Solver;

static GLOBAL_TIME_LIMIT: Duration = Duration::from_secs(60); // can be changed

pub trait Problem: Sized + Clone + Debug {
    // Every Problem will probably need it's own "associated" solution type
    type Sol: Solution;

    /// Every instance of this struct should have a descriptive name (for tracing, debugging).
    /// Default works, but is very long (override it to make it friendlier).
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Every instance should have a SHORT description for Debugging,
    /// giving things like a knapsack's capacity, pehaps more.
    fn short_description(&self) -> String;

    // Constructors

    /// `new` creates a default ("zero") instance of the problem,
    /// where `size` is the number of decisions to be made (free variables to assign values to).
    fn new(size: usize) -> Self;

    /// `random` creates a full-fledged, i.e. complete but random instance of the problem,
    /// where `size` is the number of decisions to be made (free variables to assign values to).
    /// Do not confuse with `random_solution`!
    fn random(size: usize) -> Self {
        let mut result = Self::new(size);
        result.randomize();
        result
    }

    /// The number of decisions to be made (free variables to assign values to)-
    fn problem_size(&self) -> usize;

    /// Given a solution (self), reset all the values at random, while preserving legality.
    fn randomize(&mut self);

    /// is_legal tests whether a problem -- not whether a solution -- is legal
    /// (the Solution trait has its own is_legal method).
    /// For example, are all of the weights of a knapsack greater than zero, is the dimension
    /// greater than zero, is the capacity OK, etc.
    /// In other words, is a valid soution possible (not whether a given solution valid).
    fn is_legal(&self) -> bool;

    /// ## Solution attributes that only the problem can evaluate
    /// What is the score of a given Solution?
    fn solution_score(&self, solution: &Self::Sol) -> ScoreType;

    /// What is the "upper" bound of the score of a given Solution?
    /// Note: If we're maximizing, this is the upper bound,
    /// but if we're minimizing, this is the lower bound.
    fn solution_best_score(&self, solution: &Self::Sol) -> ScoreType;

    /// Helper function to record the score and best score of a given solution
    fn fix_scores(&self, solution: &mut Self::Sol) {
        solution.put_score(self.solution_score(solution));
        solution.put_best_score(self.solution_best_score(solution));
    }

    /// Is a given solution legal *for this problem*?
    fn solution_is_legal(&self, solution: &Self::Sol) -> bool;

    /// Is a given solution complete *for this problem*?
    fn solution_is_complete(&self, solution: &Self::Sol) -> bool;

    /// ## Methods used by the Unified Optimization Algorithm (identified above)
    ///
    /// Create a random complete solution of this problem:
    fn random_solution(&self) -> Self::Sol;

    /// Create a (clone of) the starting solution for this problem,
    /// i.e. the solution with NO decisions made yet.
    fn starting_solution(&self) -> Self::Sol;

    /// Is new_solution better than old_solution?
    /// Note that the default version assumes we're maximizing.
    fn better_than(&self, new_solution: &Self::Sol, old_solution: &Self::Sol) -> bool {
        old_solution.get_best_score() <= new_solution.get_best_score()
    }

    /// Is the "upper bound" of new_solution better than score the old solution?
    /// Note that the default version assumes we're maximizing.
    fn can_be_better_than(&self, new_solution: &Self::Sol, old_solution: &Self::Sol) -> bool {
        self.solution_best_score(old_solution) <= self.solution_best_score(new_solution)
    }

    /// Find the index of the next decision to make (bit to set), if any,
    /// or return None if there are no more open decisions.
    fn first_open_decision(&self, solution: &Self::Sol) -> Option<usize>;

    /// Find the largest index of a closed decision, if any,
    /// or return None if there are no closed decisions
    /// (which defines the starting solution, by the way).
    fn last_closed_decision(&self, solution: &Self::Sol) -> Option<usize>;

    /// Apply this problem's only logic to check if any decisions are implicitly already decided.
    /// Example: if some items are heavier than a knapsack's remainng capacity, we don't have
    /// to consider putting them into the knapsack.
    fn make_implicit_decisions(&self, sol: &mut Self::Sol);

    /// Create a child of a parent solution, given a decision and a decision to make
    /// (i.e. decision index and boolean value) -- and iff the solution is good, add it to the
    /// solver (otherwise discard).
    fn register_one_child(
        &self,
        parent: &Self::Sol,
        solver: &mut impl Solver<Self::Sol>,
        index: usize,
        decision: bool,
    ) {
        let mut new_solution = parent.clone();
        new_solution.make_decision(index, decision);
        self.make_implicit_decisions(&mut new_solution);
        self.fix_scores(&mut new_solution);
        if self.solution_is_legal(&new_solution) {
            solver.push(new_solution);
        } // else if solution is illegal, do nothing
    }

    /// Create and then register all (usually both) children of a parent solution;
    /// compare `register_one_child` method.
    fn register_children_of(&self, parent: &Self::Sol, solver: &mut impl Solver<Self::Sol>) {
        debug_assert!(self.solution_is_legal(parent));
        match self.first_open_decision(parent) {
            None => {} // do nothing!
            Some(index) => {
                self.register_one_child(parent, solver, index, false);
                self.register_one_child(parent, solver, index, true);
            } // end if found Some(index) -- an open decision
        } // end match
    } // end register_children

    /*******************************************************************************/
    /// This is the crux of this whole project: The `find_best_solution` method.
    /// It does what it says here.
    /// Originally outside this (Problem) Trait, but the compiler is making this difficult...
    fn find_best_solution<Solv: Solver<Self::Sol>>(
        &self,
        solver: &mut Solv,
        time_limit: Duration,
    ) -> Result<Self::Sol, Box<dyn Error>> {
        use log::*; // for info, trace, warn, etc.
        use std::fs::OpenOptions; // and/or File, if we want to overwrite a file...
        use std::io::prelude::*; // for writeln! (write_fmt)
        use std::time::Instant;

        let global_start_time = Instant::now();
        let mut start_time = Instant::now();

        // define some solution to be "best-so-far"
        let mut num_visitations: i64 = 0;
        let mut best_solution = self.random_solution();
        debug_assert!(self.solution_is_complete(&best_solution));
        debug_assert!(self.solution_is_legal(&best_solution));
        trace!("Optimizing Problem (short) {}", self.short_description());
        trace!(
            "First Random Solution (short) {}",
            best_solution.short_description()
        );

        // start at the root of the tree
        debug_assert!(solver.is_empty());
        solver.push(self.starting_solution());

        let result = loop {
            num_visitations += 1;

            let next_solution = solver
                .pop()
                .expect("solver's queue should not be empty but could not pop");
            trace!(
                "Optimizer pops {} solution {} at depth {}, high score {}",
                if self.solution_is_complete(&next_solution) {
                    "  COMPLETE"
                } else {
                    "incomplete"
                },
                next_solution.short_description(),
                self.first_open_decision(&next_solution).unwrap_or(99999999),
                best_solution.get_score()
            );

            if self.solution_is_complete(&next_solution)
                && self.better_than(&next_solution, &best_solution)
            {
                // record new best solution.
                best_solution = next_solution.clone();
                // record new best solution as trace and as a line in trace.csv
                trace!(
                    "Optimizer finds new BEST score {}! after {} visitations",
                    best_solution.get_score(),
                    num_visitations
                );
                // Reset timer!
                // That means we exit if we go for time_limit without a new best solution!
                start_time = Instant::now();
            }; // end solution complete and better than old best solution

            // BOUND
            if self.can_be_better_than(&next_solution, &best_solution) {
                // BRANCH
                self.register_children_of(&next_solution, solver);
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
            self.name(),
            start_time.elapsed().as_nanos(),
            num_visitations,
            solver.number_of_solutions(),
            result.get_score(),
            result.get_best_score(),
        )?;

        trace!("Optimizer find best solution in {:?}", self);
        trace!("Optimizer converges on soution {:?}", result);

        Ok(result)
    } // end default find_best_solution implementation
} // end trait Problem
