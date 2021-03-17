/// # Example Implementations
///
///
///
use mhd_optimizer::{Solution, Solver};

/// ## Example Problem Implementation: Depth First Search
///
///
use mhd_method::ZERO_SCORE; // ScoreType not needed (?!?)
use std::collections::BinaryHeap;

#[derive(Debug, Clone)]
pub struct BestFirstSolver<Sol: Solution> {
    pub solutions: BinaryHeap<Sol>,
}

impl<Sol: Solution> Solver<Sol> for BestFirstSolver<Sol> {
    fn name(&self) -> &'static str {
        "BestFirstSolver "
    }

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

    fn new(_: usize) -> Self {
        Self {
            solutions: BinaryHeap::new(),
        }
    }

    // Methods used by the Unified Optimization Algorithm (identified above)

    fn number_of_solutions(&self) -> usize {
        self.solutions.len()
    }
    fn is_empty(&self) -> bool {
        self.solutions.is_empty()
    }
    fn clear(&mut self) {
        self.solutions.clear()
    }

    fn push(&mut self, solution: Sol) {
        self.solutions.push(solution);
    }
    fn pop(&mut self) -> Option<Sol> {
        self.solutions.pop()
    }
} // end imp Solver for BestFirstSolver

///////////////////// TESTs for ProblemSubsetSum with  BestFirstSolver /////////////////////
#[cfg(test)]
mod more_tests {
    use super::*;
    use implementations::ProblemSubsetSum;
    use mhd_optimizer::{MinimalSolution, Solution};
    use mhd_optimizer::{Problem, Solver};


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

        let the_best = knapsack
            .find_best_solution(&mut second_solver, time_limit)
            .expect("could not find best solution");

        assert!( knapsack.solution_is_legal( &the_best) );
        assert!( knapsack.solution_is_complete( &the_best ));
        assert_eq!(
            knapsack.solution_score(&the_best),
            knapsack.capacity
        );
        assert_eq!(the_best.get_score(), knapsack.capacity);
    }
}
