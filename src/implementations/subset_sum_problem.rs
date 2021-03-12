/// # Example Implementations
///
/// ## Example Problem Implementation: Subset Sum 0-1 Knapsack
///
///

extern crate rand_distr;

use rand_distr::{Exp, Bernoulli, Distribution};

use ::mhd_method::sample::{Sample, ScoreType, NUM_BITS, ZERO_SCORE }; // Not used: NUM_BYTES

use ::mhd_optimizer::{ Solution, TwoSampleSolution };
use ::mhd_optimizer::{ Solver, Problem };

#[derive(Debug,Clone)]
pub struct ProblemSubsetSum {
    pub weights   : Vec< ScoreType >,
    pub capacity  : ScoreType,  // The capacity of the Knapsack (not of the weights vector)
} // end struct Sample

// Utility Methods (not part of the Problem trait)
impl ProblemSubsetSum {

    pub fn weights_sum( & self ) -> ScoreType {
        self.weights.iter().sum()
    }

    pub fn first_open_decision( & self, solution : & TwoSampleSolution ) -> Option< usize > {
        // Note to self -- later we can be faster here by doing this byte-wise
        for index in 0..self.problem_size() {
            if ! solution.mask.get_bit( index ) {
                return Some( index );
            };
        } // end for all bits
        return None;
    }

    pub fn make_implicit_decisions( & self,
                                    sol : & mut TwoSampleSolution ) {
        if self.solution_is_legal( &sol )
            && ! self.solution_is_complete( & sol  ) {
            let headroom = self.capacity - sol.get_score();
            for bit in 0..self.problem_size() {
                if None == sol.get_decision( bit )
                    && headroom < self.weights[ bit ] {
                    // found an unmade decision which cannot legally be made
                    sol.make_decision( bit, false );
                } // end if implicit false decision
            } // end for all bits
        } // end if incomplete decision
    }
    pub fn register_one_child( & self,
                               parent : & TwoSampleSolution,
                               solver : & mut impl Solver< TwoSampleSolution >,
                               index : usize, decision : bool,  ) {
        let mut new_solution = parent.clone();
        new_solution.make_decision( index, decision );
        self.make_implicit_decisions( & mut new_solution );
        if self.solution_is_legal( & new_solution ) {
            new_solution.put_score(      self.solution_score(      & new_solution ));
            new_solution.put_best_score( self.solution_best_score( & new_solution ));
            solver.push( new_solution );
        } // else if solution is illegal, do nothing
    }

}

// Problem Trait Methods
impl Problem< TwoSampleSolution > for ProblemSubsetSum {

    fn new( size: usize ) -> Self {
        ProblemSubsetSum {
            weights : vec![ ZERO_SCORE; size ],
            capacity : 0,
        }
    }

    // fn random( size : usize ) -> Self -- take the default implementation

    fn problem_size( & self ) -> usize {
        return self.weights.len();
    }

    fn randomize( &mut self ) {
        let num_bits = self.problem_size( );
        assert!( 1 < num_bits, "Randomize not defined when problem_size = {}", num_bits );
        // self.weights =  (0..self.problem_size()).map( |_| fancy_random_int( ) ).collect();
        let mut rng = rand::thread_rng();
        let expo_distr = Exp::new(3.0/16.0).unwrap();

        self.weights = (0..num_bits)
            .map( |_| (expo_distr.sample( & mut rng ) * 1000.0 + 1.0) as ScoreType )
            .collect();

        ///// The next two lines are optional. Experimentation still going on to see if they help.
        ////  They are not independant: The 2nd makes no sense without the first, so either none,
        ////  just the first or both. See below for experimental results.
        // Sort weights
        self.weights.sort_unstable();
        self.weights.reverse();
        assert!( num_bits == self.problem_size(), "Problem size changed in sort?!?");
        assert!( 0 < self.weights[0] );
        assert!( 0 < self.weights[num_bits-1] );
        assert!( self.weights[num_bits-1] <= self.weights[0] ); // Change if not reversing sort


        // Choose Capacity as the sum of a random selection of the weights
        let berno_distr = Bernoulli::new(0.5).unwrap();
        loop {
            self.capacity = self.weights.iter()
                .map(|w| if berno_distr.sample(&mut rng) { *w } else { ZERO_SCORE })
                .sum();
            if self.is_legal() { return; };
            // else, find another capacity
        }; // loop until self.is_legal();

    }

    fn is_legal( & self ) -> bool {
        // Note: We're NOT testing whether a solution is legal (we do that below),
        // We're testing if a PROBLEM is non-trivial: if neither the empty knapsack
        // nor the knapsack with ALL items are obviously optimal solutions.
        // Note: By definition, the default knapsack is ILLEGAL since all weights are zero, etc.
        (0 < self.problem_size()) && (0 < self.capacity) && (self.capacity < self.weights_sum())
    }


    // first, methods not defined previously, but which arose while implemeneting the others (see below)
    fn solution_score( & self, solution : & TwoSampleSolution ) -> ScoreType {
        assert!( self.problem_size() <= NUM_BITS );
        let mut result = ZERO_SCORE;
        // Note to self -- later we can be faster here by doing this byte-wise
        for index in 0..self.problem_size() {
            if solution.mask.get_bit( index ) && solution.decisions.get_bit( index ) {
                result += self.weights[ index ];
            };
        } // end for all bits
        return result as ScoreType;
    } // end solution_is_legal

    fn solution_best_score( & self, solution : & TwoSampleSolution ) -> ScoreType {
        assert!( self.problem_size() <= NUM_BITS );
        assert!( self.solution_is_legal( solution ) );
        let mut result = self.solution_score( & solution );
        for index in 0..self.problem_size() {
            if ! solution.mask.get_bit( index ) {
                // open decision! So we COULD put this item in the knapsack...
                result += self.weights[ index ];
                if self.capacity < result {
                    return self.capacity;
                };
            };
        } // end for all bits
        // if we're here, then upper_bound is less than capacity
        assert!( result <= self.capacity );
        assert!( self.solution_score( & solution ) <= result );
        assert!( !self.solution_is_complete( & solution ) || (self.solution_score( & solution ) == result) );
        return result;
    }

    fn solution_is_legal( & self, solution : & TwoSampleSolution ) -> bool {
        // Note for the future:
        // If we ever get rid of the NUM_BITS constant, we'll need to do this:
        // let num_decisions = self.problem_size();
        // if solution.mask.len() < num_decisions      return false;
        // if solution.decisions.len() < num_decisions return false;
        // and then check capacity anyway...
        return ( self.problem_size() <= NUM_BITS )
               && ( self.solution_score( solution ) <= self.capacity );
    } // end solution_is_legal

    fn solution_is_complete( & self, solution : & TwoSampleSolution ) -> bool {
        assert!( self.solution_is_legal( & solution ));
        return None == self.first_open_decision( solution );
    } // end solution_is_complete


    fn random_solution( & self ) -> TwoSampleSolution {

        // We want a complete, final solution -- so all mask bits are one --
        // which has a random selection of things in the knapsack.
        let mut result = TwoSampleSolution { mask      : Sample::new_ones( ZERO_SCORE ),
            decisions : Sample::random( ) };

        // Take items out of knapsack iff necessary, as long as necessary, until light enough.
        if ! self.solution_is_legal( & result ) {
            // while illegal -- i.e. too much in knapsack (?!?)
            assert!( self.problem_size() <= NUM_BITS );
            let mut weight = self.solution_score( & result );
            assert!( self.capacity < weight );
            // Note to self -- later we can be faster here by doing this byte-wise
            for index in 0..self.problem_size() {
                if result.mask.get_bit( index ) && result.decisions.get_bit( index ) {
                    weight -= self.weights[ index ];
                    result.decisions.set_bit( index, false );
                    if weight < self.capacity { break; } // leave for loop.
                };
            }; // end for all bits in solution
            assert!( weight == self.solution_score( & result ) );
            assert!( weight <= self.capacity );
        }; // end if illegal

        // store the solutions's score in the solution
        result.put_score( self.solution_score( & result ) );
        result.put_best_score( self.solution_best_score( & result ) );

        assert!(  self.solution_is_legal( & result ) );

        return result;
    }

    fn starting_solution( & self ) -> TwoSampleSolution {

        // We want an "innocent" solution, before any decision as been made,
        // So all mask bits are one. It doesn't matter what the decisions are,
        // but we set them all to false.
        let mut result = TwoSampleSolution { mask      : Sample::new( 0 ),
                                             decisions : Sample::new( 0 ) };

        // store the solutions's score in the solution
        result.put_score( self.solution_score( & result ));
        result.put_best_score( self.capacity );

        assert!(  self.solution_is_legal( & result ) );
        return result;
    }

    // At first, we used the default implementations of:
    // fn better_than(        & self, new_solution : & Sol, old_solution : & Sol ) -> bool   and
    // fn can_be_better_than( & self, new_solution : & Sol, old_solution : & Sol ) -> bool
    // These are faster:
    fn better_than(        & self, new_solution : & TwoSampleSolution,
                                   old_solution : & TwoSampleSolution ) -> bool {
        old_solution.get_score( )  < new_solution.get_score( )
    }
    fn can_be_better_than( & self, new_solution : & TwoSampleSolution,
                                   old_solution : & TwoSampleSolution ) -> bool {
        old_solution.get_best_score(  ) <= new_solution.get_best_score(  )
    }


    fn register_children_of( & self, parent : & TwoSampleSolution, solver : & mut impl Solver< TwoSampleSolution > ) {
        assert!( self.solution_is_legal( parent ));
        match self.first_open_decision( parent ) {
            None          => { },  // do nothing!
            Some( index ) => {
                self.register_one_child( parent, solver, index, false );
                self.register_one_child( parent, solver, index, true );
            } // end if found Some(index) -- an open decision
        } // end match
    } // end register_children

} // end impl ProblemSubsetSum

///////////////////// TESTs for ProblemSubsetSum with  FirstDepthFirstSolver /////////////////////
#[cfg(test)]
mod tests {

    use super::*;
    use ::mhd_optimizer::find_best_solution;
    use ::implementations::DepthFirstSolver;

    #[test]
    fn test_random_weights() {
        const TEST_SIZE: usize  = 16;
        let mut rand_sack_a = ProblemSubsetSum::new(TEST_SIZE);

        assert!( ! rand_sack_a.is_legal( ) );
        assert_eq!(rand_sack_a.problem_size( ), TEST_SIZE);
        assert_eq!(rand_sack_a.weights_sum(), 0 );
        assert_eq!(rand_sack_a.capacity,      0 );

        rand_sack_a.randomize();

        assert!( rand_sack_a.is_legal( ) );
        assert_eq!(rand_sack_a.problem_size( ), TEST_SIZE);

        assert_ne!(rand_sack_a.weights_sum(), 0 );
        assert_ne!(rand_sack_a.capacity,      0 );

        let rand_sack_b = ProblemSubsetSum::random(TEST_SIZE);

        assert!( rand_sack_b.is_legal( ) );
        assert_eq!(rand_sack_b.problem_size( ), TEST_SIZE);
        assert_ne!(rand_sack_b.weights_sum(), 0 );
        assert_ne!(rand_sack_b.capacity,      0 );

        let starter = rand_sack_b.starting_solution( );
        assert!( rand_sack_b.is_legal( ) );
        assert!( rand_sack_b.solution_is_legal( & starter ) );
        assert!( ! rand_sack_b.solution_is_complete( & starter ) );
        assert_eq!( rand_sack_b.solution_score( & starter ), ZERO_SCORE );
        assert_eq!( rand_sack_b.solution_best_score( & starter ),
                    rand_sack_b.capacity );
        assert_eq!( rand_sack_b.solution_score( & starter ),
                    starter.get_score( ) );
        assert_eq!( rand_sack_b.solution_best_score( & starter ),
                    starter.get_best_score( ) );


        let thrown_dart = rand_sack_b.random_solution( );
        assert!( rand_sack_b.is_legal( ) );
        assert!( rand_sack_b.solution_is_legal( & thrown_dart ) );
        assert!( rand_sack_b.solution_is_complete( & thrown_dart ) );
        assert_ne!( rand_sack_b.solution_score( & thrown_dart ), ZERO_SCORE );
        assert_ne!( rand_sack_b.solution_score( & thrown_dart ),
                    rand_sack_b.capacity ); // could be equal by dump luck, but very improbable
        assert_eq!( rand_sack_b.solution_score( & thrown_dart ),
                    thrown_dart.get_score( ) );
        assert_eq!( rand_sack_b.solution_best_score( & thrown_dart ),
                    thrown_dart.get_best_score( ) );
        assert!( thrown_dart.get_score( ) < rand_sack_b.capacity ); // could be equal by dump luck
        assert!( thrown_dart.get_best_score( ) <= rand_sack_b.capacity );

    } // end test_random_weights

    #[test]
    fn test_random_knapsacks() {
        for size in [ 2, 3, 4, 5, 6, 7, 8, 16, 32, 64, 128, 256 ].iter() {
            let sack = ProblemSubsetSum::random( *size );
            assert!( sack.is_legal(), "illegal random sack with size {}?!?", *size );
        }
    }

    #[test]
    fn test_children_regstration() {
        const NUM_BITS: usize = 32; // big, to make special cases below REALLY improbable

        // Test register_children_of( )
        let problem = ProblemSubsetSum::random(NUM_BITS); // a lot smaller
        assert!( problem.is_legal() );

        let mut solver  = DepthFirstSolver::new(NUM_BITS);

        solver.push( problem.starting_solution( ) );
        assert!( ! solver.is_empty() );

        let root = solver.pop( ).expect( "Solver should let us pop SOMETHING #1");
        assert!( solver.is_empty() );
        assert!( problem.solution_is_legal( & root ));
        assert!( ! problem.solution_is_complete( & root ));

        problem.register_children_of( & root, & mut solver );
        assert!( ! solver.is_empty() );
        assert!( solver.number_of_solutions() <= 2 );

        let node_a = solver.pop( ).expect( "Solver should let us pop SOMETHING #2");
        assert!( ! solver.is_empty() );
        assert!( solver.number_of_solutions() <= 1 );
        assert!( problem.solution_is_legal( & node_a ));
        assert!( ! problem.solution_is_complete( & node_a ));

        problem.register_children_of( & node_a, & mut solver );
        assert!( ! solver.is_empty() );
        assert!( solver.number_of_solutions() <= 3 );

        let node_b = solver.pop( ).expect( "Solver should let us pop SOMETHING #3");
        assert!( ! solver.is_empty() );
        assert!( solver.number_of_solutions() <= 2 );
        assert!( problem.solution_is_legal( & node_b ));
        assert!( ! problem.solution_is_complete( & node_b ));

        problem.register_children_of( & node_b, & mut solver );
        assert!( ! solver.is_empty() );
        assert!( solver.number_of_solutions() <= 4 );

        // Before we go...
        assert!( problem.is_legal() );

    } // end test_children_regstration


    #[test]
    fn test_find_depth_first_solution() {
        const NUM_DECISIONS: usize = 4; // for a start

        let mut little_knapsack = ProblemSubsetSum::random(NUM_DECISIONS);
        let mut first_solver   = DepthFirstSolver::new(NUM_DECISIONS);

        use std::time::{Duration};
        let time_limit = Duration::new( 1, 0); // 1 second

        assert!( little_knapsack.is_legal());

        let the_best = find_best_solution( & mut first_solver, & mut little_knapsack, time_limit )
                                       .expect("could not find best solution");

        assert!( little_knapsack.solution_is_legal( & the_best ));
        assert!( little_knapsack.solution_is_complete( & the_best ));

        assert_eq!( little_knapsack.solution_score( & the_best ), little_knapsack.capacity );
        assert_eq!( the_best.get_score(),      little_knapsack.capacity );
        assert_eq!( the_best.get_best_score(), little_knapsack.capacity );

    }


} // end mod tests
