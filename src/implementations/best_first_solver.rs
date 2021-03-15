/// # Example Implementations
///
///
///

use ::mhd_optimizer::Solver;
use ::mhd_optimizer::TwoSampleSolution;

/// ## Example Problem Implementation: Depth First Search
///
///
use std::collections::BinaryHeap;

#[derive(Debug,Clone)]
pub struct BestFirstSolver {
    pub solutions: BinaryHeap< TwoSampleSolution >
}

impl Solver<TwoSampleSolution> for BestFirstSolver {

    fn name ( & self )  -> &'static str { "BestFirstSolver "  }

    fn new(  _: usize ) -> Self {
        Self {
            solutions : BinaryHeap::new( )
        }
    }

    // Methods used by the Unified Optimization Algorithm (identified above)

    fn number_of_solutions( & self ) -> usize {
        self.solutions.len()
    }
    fn is_empty( & self ) -> bool {
        self.solutions.is_empty( )
    }
    fn clear( & mut self ) { self.solutions.clear() }

    fn push( & mut self, solution : TwoSampleSolution ) {
        self.solutions.push( solution );
    }
    fn pop( & mut self ) -> Option< TwoSampleSolution > {
        self.solutions.pop( )
    }

} // end imp Solver for BestFirstSolver

///////////////////// TESTs for ProblemSubsetSum with  BestFirstSolver /////////////////////
#[cfg(test)]
mod more_tests {
    use super::*;
    use ::mhd_optimizer::{ Problem, Solution, find_best_solution };
    use ::implementations::ProblemSubsetSum;

    const NUM_DECISIONS: usize = 64; // for a start

    #[test]
    fn test_best_first_solver_solver() {

        let mut solver = BestFirstSolver::new(NUM_DECISIONS);
        assert!( solver.is_empty());
        let solution = TwoSampleSolution::random( NUM_DECISIONS );
        solver.push( solution );
        assert!( ! solver.is_empty());
        assert_eq!( solver.number_of_solutions(), 1 );
        let solution = TwoSampleSolution::random( NUM_DECISIONS );
        solver.push( solution );
        assert_eq!( solver.number_of_solutions(), 2 );

        let _ = solver.pop(  );
        assert_eq!( solver.number_of_solutions(), 1 );
        let _ = solver.pop(  );
        assert!( solver.is_empty());

        // Try again, to test clear
        let solution = TwoSampleSolution::random( NUM_DECISIONS );
        solver.push( solution );
        let solution = TwoSampleSolution::random( NUM_DECISIONS );
        solver.push( solution );
        assert_eq!( solver.number_of_solutions(), 2 );
        solver.clear( );
        assert!( solver.is_empty());

    }

    #[test]
    fn test_find_best_first_solution() {

        let mut little_knapsack = ProblemSubsetSum::random(NUM_DECISIONS);
        let mut second_solver = BestFirstSolver::new(NUM_DECISIONS);

        use std::time::{Duration};
        let time_limit = Duration::new(1, 0); // 1 second

        assert!(little_knapsack.is_legal());

        let the_best = find_best_solution(&mut second_solver, &mut little_knapsack, time_limit)
            .expect("could not find best solution");

        assert_eq!(little_knapsack.solution_score(&the_best), little_knapsack.capacity);
        assert_eq!(the_best.get_score(), little_knapsack.capacity);
        assert_eq!(the_best.get_best_score(), little_knapsack.capacity);
    }

}
