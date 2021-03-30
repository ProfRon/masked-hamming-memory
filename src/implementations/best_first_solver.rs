/// # Example Implementations
///
///
///
use mhd_optimizer::{Solution, Solver};

/// ## Example Solver Implementation: Best First Search
///
///
use mhd_method::ZERO_SCORE; // ScoreType not needed (?!?)
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct BestFirstSolver<Sol: Solution> {
    pub solutions: BinaryHeap<Sol>,
    best_solution: Sol,
}

impl<Sol: Solution> Solver<Sol> for BestFirstSolver<Sol> {
    #[inline]
    fn name(&self) -> &'static str {
        "BestFirstSolver "
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{} holding {} solutions, best score {}",
            self.name(),
            self.number_of_solutions(),
            match self.solutions.peek() {
                None => ZERO_SCORE,
                Some(sol) => sol.get_score(),
            }
        )
    }

    #[inline]
    fn new(size: usize) -> Self {
        Self {
            solutions: BinaryHeap::new(),
            best_solution: Sol::new(size),
        }
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

    #[inline]
    fn clear(&mut self) {
        self.solutions.clear();
        let size = self.best_solution.size();
        self.best_solution = Sol::new( size );
    }

    #[inline]
    fn push(&mut self, solution: Sol) {
        self.solutions.push(solution);
    }

    #[inline]
    fn pop(&mut self) -> Option<Sol> {
        self.solutions.pop()
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
    }
} // end imp Solver for BestFirstSolver

///////////////////// TESTs for ProblemSubsetSum with  BestFirstSolver /////////////////////
#[cfg(test)]
mod more_tests {
    use super::*;
    use implementations::ProblemSubsetSum;
    use mhd_optimizer::{MinimalSolution, Problem, Solution, Solver};

    #[test]
    fn test_best_first_solver_solver() {
        const NUM_DECISIONS: usize = 64; // for a start
        let mut solver = BestFirstSolver::<MinimalSolution>::new(NUM_DECISIONS);
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
    fn test_find_best_first_solution() {
        const FEW_DECISIONS: usize = 4; // so we can be sure to find THE optimum!
        let knapsack = ProblemSubsetSum::random(FEW_DECISIONS);
        let mut second_solver = BestFirstSolver::<MinimalSolution>::new(FEW_DECISIONS);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        assert!(knapsack.is_legal());

        let the_best = second_solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find best solution");

        assert!(knapsack.solution_is_legal(&the_best));
        assert!(knapsack.solution_is_complete(&the_best));
        assert_eq!(knapsack.solution_score(&the_best), knapsack.capacity);
        assert_eq!(the_best.get_score(), knapsack.capacity);
    }
}
