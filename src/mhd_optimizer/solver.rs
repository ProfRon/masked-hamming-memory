/// ## The Solver Trait
///

use ::mhd_optimizer::Solution;

pub trait Solver< Sol : Solution >{

    // First, one "associated type"
    // type Sol = S;

    // Constructors

    fn new(     size: usize ) -> Self;

    // Methods used by the Unified Optimization Algorithm (identified above)

    fn number_of_solutions( & self ) -> usize;
    fn is_empty( & self ) -> bool {
        0 == self.number_of_solutions( )
    }

    fn push( & mut self, solution : Sol );
    fn pop( & mut self ) -> Option< Sol >;

} // end Solver Problem

