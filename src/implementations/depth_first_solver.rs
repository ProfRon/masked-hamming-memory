/// # Example Implementations
///
///

use ::mhd_optimizer::Solver;
use ::mhd_optimizer::TwoSampleSolution;

/// ## Sample Solver Implementation: Depth First Search
///
///
/// ```rust
/// use mhd_mem::mhd_method::sample::{Sample, ScoreType, NUM_BITS, ZERO_SCORE }; // Not used: NUM_BYTES
/// use mhd_mem::mhd_optimizer::{ Solution, TwoSampleSolution };
/// use mhd_mem::mhd_optimizer::Solver;
/// use mhd_mem::implementations::DepthFirstSolver;
///
/// let mut my_solver = DepthFirstSolver::new( 8 );
///
/// assert_eq!( my_solver.number_of_solutions(), 0 );
/// assert!( my_solver.is_empty() );
///
/// let sol0 = TwoSampleSolution::new( 8 );
/// let sol1 = TwoSampleSolution::random( 8 );
///
/// assert_eq!( sol0.get_score(), ZERO_SCORE );
/// assert_eq!( sol0.get_best_score(), ZERO_SCORE );
/// assert_eq!( sol0.get_decision( 0 ), None );
///
/// let mut sol2 = TwoSampleSolution::new( 4 );
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

#[derive(Debug,Clone)]
pub struct DepthFirstSolver {
    pub solutions: Vec< TwoSampleSolution >
}

impl Solver<TwoSampleSolution> for DepthFirstSolver {

    // type Sol = TwoSampleSolution;

    fn new(  _: usize ) -> Self {
        Self {
            solutions : Vec::new( )
        }
    }

    // Methods used by the Unified Optimization Algorithm (identified above)

    fn number_of_solutions( & self ) -> usize {
        self.solutions.len()
    }
    fn is_empty( & self ) -> bool {
        self.solutions.is_empty( )
    }

    fn push( & mut self, solution : TwoSampleSolution ) {
        self.solutions.push( solution );
    }
    fn pop( & mut self ) -> Option< TwoSampleSolution > {
        self.solutions.pop( )
    }

}
