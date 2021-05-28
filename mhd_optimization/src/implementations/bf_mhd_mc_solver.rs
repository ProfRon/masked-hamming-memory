use log::*;

use mhd_memory::*;
use optimizer::{PriorityType, Problem, Solution, Solver};
use std::collections::BinaryHeap;

/// # Example Implementations
///

/**************************************************************************************/
/// ## Example Solver Implementation: MCTS, Monte Carlo Tree Search
///
pub struct BestfirstMhdMonteCarloSolver<Sol: Solution, Prob: Problem<Sol = Sol>> {
    pub mhd_memory: MhdMemory,
    pub solutions: BinaryHeap<Sol>,
    pub best_solution: Sol,
    pub problem: Prob,
}

/// Extra Methods for the BestfirstMhdMonteCarloSolver type (trait, whatever)
impl<Sol: Solution, Prob: Problem<Sol = Sol>> BestfirstMhdMonteCarloSolver<Sol, Prob> {
    // Never use an empty mhd memory -- when empty, add some rows!
    // Used by builder and clear (and who knows what else meanwhile?)
    fn bootstrap_memory(&mut self) {
        assert!(self.mhd_memory.is_empty());
        // bootstrap the memory with random samples (but legal ones!)

        // Actually what I wahted was
        // "while product.number_of_solutions() < problem.problem_size()"
        // but that can take FOREVER due to duplicate solutions @ small width
        // So instead...
        for _ in 0..self.problem.problem_size() {
            // create a square memory -- with height == width
            let solution = self.problem.random_solution();
            self.mhd_memory
                .write_sample(&self.problem.sample_from_solution(&solution));
        }
    }

    /// a replacement for Self::new( size )
    pub fn builder(problem: &Prob) -> Self {
        // build a memory....
        let mut product = Self {
            mhd_memory: MhdMemory::new(problem.problem_size()),
            solutions: BinaryHeap::new(),
            best_solution: problem.random_solution(),
            problem: problem.clone(),
        };
        // bootstrap the memory with random samples (but legal ones!)
        product.bootstrap_memory();
        // Finished! Return what we've built!
        product
    }
} // end private Methods

/// Here are the public methods needed to implement Solver<Sol>
impl<Sol: Solution, Prob: Problem<Sol = Sol>> Solver<Sol>
    for BestfirstMhdMonteCarloSolver<Sol, Prob>
{
    #[inline]
    fn name(&self) -> &'static str {
        "BestfirstMhdMonteCSolver "
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{}, memory has width {}, {} rows and {} leaves",
            self.name(),
            self.mhd_memory.width(),
            self.mhd_memory.num_samples(),
            self.solutions.len(),
        )
    }

    #[inline]
    fn new(_: usize) -> Self {
        panic!("New(size) not defined for BestfirstMhdMonteCarloSolver!");
    }

    // Methods used by the Unified Optimization Algorithm (identified above)

    #[inline]
    fn number_of_solutions(&self) -> usize {
        self.solutions.len()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.solutions.is_empty()
    }

    // take default fn is_finished(&self) -> bool

    #[inline]
    fn clear(&mut self) {
        self.solutions.clear();
        let width = self.mhd_memory.width();
        self.mhd_memory.clear();
        self.bootstrap_memory();
        self.best_solution = Sol::new(width);
    }

    #[inline]
    fn push(&mut self, solution: Sol) {
        if self.problem.solution_is_complete(&solution) {
            // Done! Solution is complete! Write it into the memory and return it
            self.mhd_memory.write_sample(&self.problem.sample_from_solution(&solution));
        }; // end if complete
        // whether complete or not...
        self.solutions.push(solution);
    }

    #[inline]
    fn pop(&mut self) -> Option<Sol> {
        self.solutions.pop()
    }

    /////// THIS IS WHERE THE MAGIC TAKES PLACE!!! ///////
    fn children_of_solution<ArgProb: Problem>(&mut self, parent: &Sol, _: &ArgProb) -> Vec<Sol> {
        let mut result = Vec::<Sol>::new(); // initially empty...
        if self.problem.solution_is_complete(&parent) {
            // Done! Solution is complete! Write it into the memory and return it
            self.mhd_memory.write_sample(&self.problem.sample_from_solution(&parent));
            // return empty vector (it has no children, so we're done)
            result
        } else {
            // solution is NOT complete (is incomplete) -- it has children
            let open_decision = self
                .problem
                .first_open_decision(&parent)
                .expect("Should have an open decision");
            // Decide whether to set the next open bit to true or false, 1 or 0
            // First, query the mhd memory
            let priorities =
                self.mhd_memory
                    .read_2_priorities(parent.mask(), parent.query(), open_decision);

            trace!(
                "BF MHD BEST FIRST MCTS: depth {}, solution score {} (high score {}) => prios ({},{})",
                open_decision,
                parent.get_score(),
                self.best_solution.get_score(),
                priorities.0, priorities.1,
            );

            // Push both children into result vector
            let mut false_child = parent.clone();
            false_child.make_decision(open_decision, false);
            self.problem.apply_rules(&mut false_child);
            false_child.set_priority(priorities.0 as PriorityType);
            debug_assert!(self.problem.rules_audit_passed(&false_child));
            result.push(false_child);

            let mut true_child = parent.clone();
            true_child.make_decision(open_decision, true);
            self.problem.apply_rules(&mut true_child);
            true_child.set_priority(priorities.1 as PriorityType);
            debug_assert!(self.problem.rules_audit_passed(&true_child));
            result.push(true_child);

            // return
            result
        }
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
} // end imp Solver for BestfirstMhdMonteCarloSolver

/**************************************************************************************/
//////////////// TESTs for ProblemSubsetSum with  MonteCarloTreeSolver /////////////////
#[cfg(test)]
mod more_tests {

    use super::*;
    use implementations::*;
    use optimizer::{MinimalSolution, Problem, Solution, Solver};

    #[test]
    fn test_bf_mc_mhd_solver() {
        const NUM_DECISIONS: usize = 8; // for a start
        let problem = ProblemSubsetSum::random(NUM_DECISIONS);
        assert!(problem.is_legal());
        let mut solver =
            BestfirstMhdMonteCarloSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem);

        assert!(solver.is_empty());
        let solution = MinimalSolution::random(NUM_DECISIONS);
        solver.push(solution);
        assert!(!solver.is_empty());
        assert_eq!(solver.number_of_solutions(), 1);
        let solution = MinimalSolution::random(NUM_DECISIONS);
        solver.push(solution);
        assert_eq!(solver.number_of_solutions(), 2);

        let _ = solver.pop();
        assert_eq!(solver.number_of_solutions(), 1);
        let _ = solver.pop();
        assert!(solver.is_empty());

        // Try again, to test clear
        let solution = MinimalSolution::random(NUM_DECISIONS);
        solver.push(solution);
        let solution = MinimalSolution::random(NUM_DECISIONS);
        solver.push(solution);
        assert_eq!(solver.number_of_solutions(), 2);
        solver.clear();
        assert!(solver.is_empty());
    }

    #[test]
    fn test_bf_mcts_find_solution() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!

        let knapsack = ProblemSubsetSum::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            BestfirstMhdMonteCarloSolver::<MinimalSolution, ProblemSubsetSum>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!(
            "Start of test_bf_mcts_find_solution, knapsack = {:?}",
            knapsack
        );

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
    fn test_bf_mcts_find_01knapsack_solution() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!

        let knapsack = Problem01Knapsack::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            BestfirstMhdMonteCarloSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(
                &knapsack,
            );

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!(
            "Start of test_bf_mcts_find_01knapsack_solution, knapsack = {:?}",
            knapsack
        );

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
    fn test_bf_mcts_solve_mutliple_knapsacks() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!

        let knapsack = Problem01Knapsack::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            BestfirstMhdMonteCarloSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(
                &knapsack,
            );

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!(
            "Start of test_bf_mcts_solve_mutliple_knapsacks, knapsack = {:?}",
            knapsack
        );

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
