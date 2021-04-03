use mhd_method::*;
use mhd_optimizer::{Problem, Solution, Solver};

/// # Example Implementations
///
///
///

pub struct MhdMonteCarloSolver<Sol: Solution, Prob: Problem<Sol = Sol>> {
    pub mhd_memory: MhdMemory,
    pub best_solution: Sol,
    pub problem: Prob,
}

impl<Sol: Solution, Prob: Problem<Sol = Sol>> MhdMonteCarloSolver<Sol, Prob> {
    // a replacement for Self::new( size )
    #[inline]
    pub fn builder(problem: &Prob) -> Self {
        Self {
            mhd_memory: MhdMemory::new(problem.problem_size()),
            best_solution: problem.random_solution(),
            problem: problem.clone(),
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
        let max_solutions : usize = if 32 < self.mhd_memory.width {
            u32::MAX as usize
        } else {
            1 << self.mhd_memory.width()  // 2 ^ width
        };
        // now, return true, finished, exhausted when...
        max_solutions < self.number_of_solutions()
    }

    #[inline]
    fn clear(&mut self) {
        let width = self.mhd_memory.width();
        self.mhd_memory.clear();
        self.best_solution = Sol::new(width);
    }

    #[inline]
    fn push(&mut self, solution: Sol) {
        // we'd like to check for completion, but can't use problem.solution_is_complete( s )
        if self.best_score() < solution.get_score() {
            panic!("Push not implemented!");
        }
    }

    /////// THIS IS WHERE THE MAGIC TAKES PLACE!!! ///////
    fn pop(&mut self) -> Option<Sol> {
        let mut solution = self.problem.starting_solution();
        while !self.problem.solution_is_complete(&solution) {
            let open_decision = self
                .problem
                .first_open_decision(&solution)
                .expect("Should have an open decision");
            // Decide whether to set the next open bit to true or false, 1 or 0
            // First, query the mhd memory
            let decision =
                self.mhd_memory
                    .read_and_decide(solution.mask(), solution.query(), open_decision);
            // now that we've made our decision, modify "solution" until it's complete
            solution.make_decision(open_decision, decision);
            self.problem.apply_rules(&mut solution);
            debug_assert!(self.problem.rules_audit_passed(&solution));
        } // end while solution not complete

        // Done! Solution is complete! Write it into the memory and return it
        self.mhd_memory
            .write_sample(&self.problem.sample_from_solution(&solution));
        Some(solution)
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
    use log::*; // for info, trace, warn, etc.
    use mhd_optimizer::{MinimalSolution, Problem, Solution, Solver};

    #[test]
    fn test_mc_mhd_solver() {
        const NUM_DECISIONS: usize = 8; // for a start
        let problem = ProblemSubsetSum::random(NUM_DECISIONS);
        assert!(problem.is_legal());
        let mut solver =
            MhdMonteCarloSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem);
        assert!(solver.is_empty());

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
        assert!(solver.is_empty());

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
