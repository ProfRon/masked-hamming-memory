use log::*;

use mhd_memory::*;
use optimizer::{Problem, Solution, Solver};

/// # Example Implementations
///
///
///

pub struct MhdMonteCarloSolver<Sol: Solution, Prob: Problem<Sol = Sol>> {
    pub mhd_memory: MhdMemory,
    pub best_solution: Sol,
    pub problem: Prob,
    pub full_monte: bool,
}

impl<Sol: Solution, Prob: Problem<Sol = Sol>> MhdMonteCarloSolver<Sol, Prob> {
    // Never use an empty mhd memory -- when empty, add some rows!
    fn bootstrap_memory(&mut self) {
        assert!(self.mhd_memory.is_empty());
        // bootstrap the memory with random samples (but legal ones!)

        // Actually what I wanted was
        // "while product.number_of_solutions() < problem.problem_size()"
        // but that can take FOREVER due to duplicate solutions @ small width
        // So instead...
        // Version 0 (until 30 May 2021)
        // let starting_row_count = self.problem.problem_size().next_power_of_two();
        // for _ in 0..starting_row_count {
        //     // create a square memory -- with height == width
        //     let solution = self.problem.random_solution();
        //     self.mhd_memory
        //         .write_sample(&self.problem.sample_from_solution(&solution));
        // };
        // Version 1: (since 30 May 2021)
        let target = if self.problem.problem_size() < 16 { 4 } else { 16 };
        while self.mhd_memory.num_samples() < target {
            let solution = self.problem.random_solution();
            self.mhd_memory
                .write_sample(&self.problem.sample_from_solution(&solution));
        };
        debug!("MHD MCSolver Builder -- Size goal was {}, build size {}",
                 target, self.mhd_memory.num_samples() );
    }

    // a replacement for Self::new( size )
    #[inline]
    pub fn builder(problem: &Prob) -> Self {
        // build a memory....
        let mut product = Self {
            mhd_memory: MhdMemory::new(problem.problem_size()),
            best_solution: problem.random_solution(),
            problem: problem.clone(),
            full_monte: false, // until overwritten with true
        };
        // bootstrap the memory with random samples (but legal ones!)
        product.bootstrap_memory();
        // Finished! Return what we've built!
        product
    }

    /// **The whole magic is _here!_**
    ///
    /// `find_new_solution()` is a recursive utility function that uses MCTS
    /// using an MHD Memory instead of a tree to find a new solution-- and
    /// knows how to react should it find a solution which is already in the memory.
    fn find_new_solution(&mut self, solution: &Sol) -> Option<Sol> {
        if self.problem.solution_is_complete(solution) {
            if self
                .mhd_memory
                .write_sample(&self.problem.sample_from_solution(solution))
            {
                trace!("find_new_solution, returning new solution!");
                Some(solution.clone())
            } else {
                trace!("find_new_solution, returning NONE = No Solution!");
                None
            }
        } else {
            // if sol is NOT complete (is incomplete)
            let open_decision = self
                .problem
                .first_open_decision(solution)
                .expect("Should have an open decision");
            // Decide whether to set the next open bit to true or false, 1 or 0
            // First, query the mhd memory
            let decision = self.mhd_memory.read_and_decide(
                solution.mask(),
                solution.query(),
                open_decision,
                self.full_monte,
            );

            // Now, try this solution and see if it's usable...
            let mut child = solution.clone();
            child.make_decision(open_decision, decision);
            self.problem.apply_rules(&mut child);
            debug_assert!(self.problem.rules_audit_passed(&child));

            trace!(
                "find_new_solution, depth = {}, first try = {}",
                open_decision,
                decision
            );

            let first_try = self.find_new_solution(&child);
            if first_try.is_some() {
                trace!(
                    "find_new_solution({}), first try was a hit!!",
                    open_decision
                );
                first_try
            } else {
                // if first is none, then we change our mind about decision (!)
                child = solution.clone(); // go back one step
                let not_decision = !decision;
                trace!(
                    "find_new_solution, depth = {}, second try = {}",
                    open_decision,
                    not_decision
                );
                child.make_decision(open_decision, not_decision);
                self.problem.apply_rules(&mut child);
                debug_assert!(self.problem.rules_audit_passed(&child));
                // try this new child and return the result, even if none
                let second_try = self.find_new_solution(&child);
                trace!(
                    "find_new_solution({}) 2nd try was a hit? {}!",
                    open_decision,
                    second_try.is_some()
                );
                second_try
            }
        }
    }
} // end private Methods

/**************************************************************************************/
/// ## Example Solver Implementation: MCTS, Monte Carlo Tree Search
///
/// Here are the public methods needed to implement Solver<Sol>
impl<Sol: Solution, Prob: Problem<Sol = Sol>> Solver<Sol> for MhdMonteCarloSolver<Sol, Prob> {
    #[inline]
    fn name(&self) -> &'static str {
        "MhdMonteCarloSolver "
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{}, memory has width {} and {} rows",
            self.name(),
            self.mhd_memory.width(),
            self.mhd_memory.num_samples()
        )
    }

    #[inline]
    fn new(_: usize) -> Self {
        panic!("New(size) not define for MhdMonteCarloSolver!");
    }

    // Methods used by the Unified Optimization Algorithm (identified above)

    #[inline]
    fn number_of_solutions(&self) -> usize {
        self.mhd_memory.num_samples()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.mhd_memory.is_empty()
    }

    #[inline]
    fn is_finished(&self) -> bool {
        // Max size should be about a gigabyte.
        // Problem: Small problems lead to many, many solutions (max)....
        // Alternative = 2 ^width. So, where is the threshold?
        // width      width in bytes     max_solutions in 1 GB  2^width
        // 256        32                 32 Mega = 2^25   <     2^256
        // 64         8                 128 Mega = 2^27   <     2^64
        // 32         4                 256 Mega = 2^28   <     2^32
        // 28         4 (!)             256 Mega = 2^28   =     2^28
        // 24         3                 341 Mega = 2^28ish >  2^24
        // 16         2                 512 Mega = 2^29    >  2^16
        let max_solutions: usize = if self.mhd_memory.width <= 28 {
            1 << self.mhd_memory.width() // 2 ^ width
        } else {
            // if 28 < self.mhd_memory.width
            const MAX_MEMORY: usize = 1 << 30;
            let width_in_bytes = (self.mhd_memory.width() + 7) / 8;
            MAX_MEMORY / width_in_bytes
        };
        // now, return true, finished, exhausted when...
        // ( self.mhd_memory.width() <= self.repitition_count ) || ...
        max_solutions < self.number_of_solutions()
    }

    #[inline]
    fn clear(&mut self) {
        let width = self.mhd_memory.width();
        self.mhd_memory.clear();
        self.bootstrap_memory();
        self.best_solution = Sol::new(width);
        // Leave full_monte as it is (?!?)
    }

    #[inline]
    fn push(&mut self, solution: Sol) {
        // we'd like to check for completion, but can't use problem.solution_is_complete( s )
        if self.best_score() < solution.get_score() {
            panic!("Push not implemented!");
        }
    }

    fn pop(&mut self) -> Option<Sol> {
        match self.find_new_solution(&self.problem.starting_solution()) {
            None => {
                debug!("MHD MCTS POP Returns NONE!!");
                None
            }
            Some(solution) => {
                assert!(self.problem.solution_is_complete(&solution));
                // assert_ne!(solution, self.best_solution); Unlikely but not illegal
                debug!(
                    "MHD MCTS POP: Returns solution with score {}",
                    solution.get_score()
                );
                Some(solution)
            } // end case Some(solution)
        } // end match
    }

    #[inline]
    fn best_solution(&self) -> &Sol {
        &self.best_solution
    }

    #[inline]
    fn store_best_solution(&mut self, solution: Sol) {
        // we'd like to check for completion, but can't use proble.solution_is_complete( s )
        debug_assert_eq!(solution.get_score(), solution.get_best_score());
        // Occasionally, the following condition IS allowed (to be false)
        // debug_assert!(self.best_score() <= solution.get_score());
        self.best_solution = solution;
    } //end store_best_solution
} // end imp Solver for MhdMonteCarloSolver

/**************************************************************************************/
//////////////// TESTs for ProblemSubsetSum with  MonteCarloTreeSolver /////////////////
#[cfg(test)]
mod more_tests {

    use super::*;
    use implementations::*;
    use optimizer::{MinimalSolution, Problem, Solution, Solver};

    #[test]
    fn test_mc_mhd_solver() {
        const NUM_DECISIONS: usize = 8; // for a start
        let problem = ProblemSubsetSum::random(NUM_DECISIONS);
        assert!(problem.is_legal());
        let mut solver =
            MhdMonteCarloSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem);

        assert!(!solver.is_empty()); // bootstraping!
        assert_eq!(solver.width(), NUM_DECISIONS);
        assert!(solver.number_of_solutions() <= NUM_DECISIONS);

        debug!("Start of test_mc_mhd_solver, knapsack = {:?}", problem);

        let solution1 = solver.pop().expect("pop() should return Some(sol)");

        assert!(!solver.is_empty());
        assert!(problem.rules_audit_passed(&solution1));

        if problem.solution_is_complete(&solution1) {
            solver.new_best_solution(&problem, solution1); // Warning: solution1 moved!
        } else {
            warn!(
                "First Solution returned is not complete? S1 = {:?}",
                solution1
            );
            warn!(
                "                      current best solution = {:?}",
                solver.best_solution()
            );
        };

        let solution2 = solver.pop().expect("pop() should return Some(sol)");
        assert!(!solver.is_empty());
        assert!(solver.problem.rules_audit_passed(&solution2));

        if problem.solution_is_complete(&solution2) {
            solver.new_best_solution(&problem, solution2); // Warning: solution1 moved!
        } else {
            warn!(
                "Second Solution returned is not complete? S1 = {:?}",
                solution2
            );
            warn!(
                "                      current best solution = {:?}",
                solver.best_solution()
            );
        };
    }

    #[test]
    fn test_mcts_find_solution() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!

        let knapsack = ProblemSubsetSum::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            MhdMonteCarloSolver::<MinimalSolution, ProblemSubsetSum>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!("Start of test find_solution, knapsack = {:?}", knapsack);

        let the_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find best solution");

        assert!(solver.problem.rules_audit_passed(&the_best));
        assert!(solver.problem.solution_is_complete(&the_best));
        assert_eq!(
            solver.problem.solution_score(&the_best),
            the_best.get_score()
        );
        assert!(the_best.get_score() <= solver.problem.capacity);
    }

    #[test]
    fn test_mcts_find_01knapsack_solution() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!

        let knapsack = Problem01Knapsack::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            MhdMonteCarloSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!("Start of test find_solution, knapsack = {:?}", knapsack);

        let the_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find best solution");

        assert!(solver.problem.rules_audit_passed(&the_best));
        assert!(solver.problem.solution_is_complete(&the_best));
        assert_eq!(
            solver.problem.solution_score(&the_best),
            the_best.get_score()
        );
    }

    #[test]
    fn test_mcts_solve_mutliple_knapsacks() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!

        let knapsack = Problem01Knapsack::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            MhdMonteCarloSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!("Start of test find_solution, knapsack = {:?}", knapsack);

        let the_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find best solution");

        assert!(solver.problem.rules_audit_passed(&the_best));
        assert!(solver.problem.solution_is_complete(&the_best));
        assert_eq!(
            solver.problem.solution_score(&the_best),
            the_best.get_score()
        );

        // Now test solver.clear()!!!
        solver.clear();
        assert!(!solver.is_empty()); // Bootstrapping, again!
        solver.full_monte = true; // two birds with one stone...  sozusagen...

        let second_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find 2nd best solution");

        assert!(solver.problem.rules_audit_passed(&second_best));
        assert!(solver.problem.solution_is_complete(&second_best));
        assert_eq!(
            solver.problem.solution_score(&second_best),
            second_best.get_score()
        );
    }
}
