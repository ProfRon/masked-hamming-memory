/// # Example Implementations
///
///
use mhd_optimizer::{Solution, Solver};

/// ## Example Solver Implementation: Depth First Search
///
///
/// ```rust
/// use mhd_mem::mhd_method::sample::{ ScoreType, ZERO_SCORE }; // Not used: NUM_BYTES
/// use mhd_mem::mhd_optimizer::{ Solution, MinimalSolution, Solver };
/// use mhd_mem::implementations::DepthFirstSolver;
///
/// let mut my_solver = DepthFirstSolver::< MinimalSolution >::new( 8 );
///
/// assert_eq!( my_solver.number_of_solutions(), 0 );
/// assert!( my_solver.is_empty() );
///
/// let sol0 = MinimalSolution::new( 8 );
/// let sol1 = MinimalSolution::random( 8 );
///
/// assert_eq!( sol0.get_score(), ZERO_SCORE );
/// assert_eq!( sol0.get_best_score(), ZERO_SCORE );
/// assert_eq!( sol0.get_decision( 0 ), None );
///
/// let mut sol2 = MinimalSolution::new( 4 );
/// sol2.make_decision( 0, true );
/// sol2.make_decision( 1, false );
/// sol2.make_decision( 2, true );
/// assert!(   sol2.get_decision( 0 ).unwrap( ) );
/// assert!( ! sol2.get_decision( 1 ).unwrap( ) );
/// assert!(   sol2.get_decision( 2 ).unwrap( ) );
/// assert_eq!( sol0.get_decision( 3 ), None );
///
/// sol2.put_score(      42 as ScoreType );
/// sol2.put_best_score( 88 as ScoreType );
/// assert_eq!( sol2.get_score(),      42 as ScoreType );
/// assert_eq!( sol2.get_best_score(), 88 as ScoreType );
///
/// my_solver.push( sol0 );
/// my_solver.push( sol1 );
/// my_solver.push( sol2 );
///
/// assert_eq!( my_solver.number_of_solutions(), 3 );
///
/// let popped = my_solver.pop( ).unwrap();
/// assert!( ! popped.get_decision( 1 ).unwrap( ) );
///
///
/// ```

#[derive(Debug, Clone)]
pub struct DepthFirstSolver<Sol: Solution> {
    pub solutions: Vec<Sol>,
    best_solution: Sol,
}

impl<Sol: Solution> Solver<Sol> for DepthFirstSolver<Sol> {
    // type Sol = TwoSampleSolution;

    #[inline]
    fn name(&self) -> &'static str {
        "DepthFirstSolver"
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{} holding {} solutions, best score is {}",
            self.name(),
            self.number_of_solutions(),
            self.best_solution().get_best_score(),
        )
    }

    #[inline]
    fn new(size: usize) -> Self {
        Self {
            solutions: Vec::new(),
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
        self.solutions.clear()
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
        debug_assert!(solution.get_score() == solution.get_best_score());
        debug_assert!(self.best_solution.get_score() <= solution.get_score());
        self.best_solution = solution;
    }

    // take default new_best_soluiton() method
}

///////////////////// TESTs for DepthFirstSolver /////////////////////
#[cfg(test)]
mod more_tests {
    use super::*;
    use mhd_optimizer::{MinimalSolution, Solution};

    const NUM_DECISIONS: usize = 64; // for a start

    #[test]
    fn test_depth_first_solver_solver() {
        let mut solver = DepthFirstSolver::<MinimalSolution>::new(NUM_DECISIONS);
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
}
