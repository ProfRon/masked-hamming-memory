/// # The Unified Decision Optimization Algorithm with the MHD Memory
/// ## The Solution Trait
///

pub trait Solution : Sized + Clone + Ord {

    // First, an "associated type"
    // Compare <file:///home/ron/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/book/ch19-03-advanced-traits.html>
    type ScoreType : PartialOrd + Display;

    // every instance of this struct should have a descriptive name (for tracing, debugging)
    // TODO: Remove this when <https://doc.rust-lang.org/std/any/fn.type_name_of_val.html> stable
    fn name ( & self )  -> &'static str;

    // At the moment, we actually have identified no methods we need for Solutions!
    // Still, this is a safe guess...
    // Recall: size is the number of decisions to be made (free variables to assign values to).
    fn new(     size: usize ) -> Self;
    fn random( size : usize ) -> Self {
        let mut result = Self::new( size );
        result.randomize( );
        result
    }

    fn randomize( &mut self );

    // Getters and Setters
    fn get_score( & self  ) -> Self::ScoreType; // score is not *calculated*! Just retrieved!
    fn put_score( & mut self, score : Self::ScoreType );

    fn get_best_score( & self  ) -> Self::ScoreType; // "upper" bound, like score, is *not calculated* (here)!
    fn put_best_score( & mut self, best : Self::ScoreType );

    fn get_decision( & self, decision_number : usize  ) -> Option< bool >; // Some(bool) or None
    fn make_decision( & mut self, decision_number : usize, decision : bool ); // side effect: set mask bit (etc)

} // end trait Solution

/// ## A Very Simple but Useful Implementation of the Solution Trait
///
/// Examples:
/// ```rust
/// use mhd_mem::mhd_method::{ ScoreType, ZERO_SCORE };
/// use mhd_mem::mhd_optimizer::{ Solution, TwoSampleSolution };
/// let sol0 = TwoSampleSolution::new( 8 );
/// let sol1 = TwoSampleSolution::random( 8 );
///
/// assert_eq!( sol0.name(), "TwoSampleSolution");
/// assert_eq!( sol0.get_score(), ZERO_SCORE );
/// // assert_eq!( sol0.get_score(), sol1.get_score() );
/// assert_eq!( sol0.get_best_score(), ZERO_SCORE );
/// // assert_eq!( sol0.get_best_score(), sol1.get_best_score() );
/// assert_eq!( sol0.get_decision( 0 ), None );
///
/// let mut sol2 = TwoSampleSolution::new( 4 );
/// assert_eq!( sol0.get_decision( 0 ), None );
/// assert_eq!( sol0.get_decision( 1 ), None );
/// assert_eq!( sol0.get_decision( 2 ), None );
/// assert_eq!( sol0.get_decision( 3 ), None );
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
/// let mut sol3 = TwoSampleSolution::new( 4 );
/// sol3.put_score(      64 as ScoreType );
/// sol3.put_best_score( 88 as ScoreType );
/// assert!( sol2 < sol3 );
/// assert!( ! (sol2 == sol3) );
/// ```

use ::mhd_method::sample::{Sample, ScoreType, NUM_BITS };
use std::fmt::Display; // Not used: NUM_BYTES
use std::cmp::Ordering;

#[derive(Debug,Clone)]
pub struct TwoSampleSolution {
    pub mask      : Sample,  // we could have used Vec<u8> (twice) here (and saved two scores),
    pub decisions : Sample,  // but we wouldn't have the get_bit and set_bit methods!
}

impl TwoSampleSolution {
    pub fn estimate( & self ) -> <TwoSampleSolution as Solution>::ScoreType {
        (self.get_score() + self.get_best_score()) / 2
    }
}

// Ord requires Eq, which requires PartialEq
impl PartialEq for TwoSampleSolution {
    fn eq(&self, other: &Self) -> bool {
        self.estimate() == other.estimate()
    }
}

impl Eq for TwoSampleSolution {}

impl Ord for TwoSampleSolution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.estimate( ).cmp( & other.estimate( ) )
    }
}

impl PartialOrd for TwoSampleSolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some( self.estimate( ).cmp( & other.estimate( ) ) )
    }
}

impl Solution for TwoSampleSolution {

    type ScoreType = ScoreType;

    fn name ( & self )  -> &'static str { "TwoSampleSolution"  }

    fn new(     size: usize ) -> Self {
        assert!( size <= NUM_BITS );
        Self {
            mask      : Sample::default(), // all zeros == no decision made yet
            decisions : Sample::default(), // all zeros == all decisions are zero/false
        }
    }

    fn randomize( &mut self ) {
        // Do **not** call self.mask.randomize();
        self.decisions.randomize( );
    }

    // Getters and Setters
    fn get_score( & self  ) -> Self::ScoreType {
        // score is not *calculated*! Just retrieved!
        self.decisions.score
    }
    fn put_score( & mut self, score : Self::ScoreType ) {
        self.decisions.score = score;

    }

    fn get_best_score( & self  ) -> Self::ScoreType {
        // best score is not *calculated*! Just retrieved!
        self.mask.score
    }
    fn put_best_score( & mut self, best : Self::ScoreType ) {
        self.mask.score = best;
    }

    fn get_decision( & self, decision_number : usize  ) -> Option< bool > {
        if self.mask.get_bit( decision_number ) {
            Some( self.decisions.get_bit( decision_number ) )
        } else { // if NOT self.mask.get_bit( decision_number )
            None
        }
    }
    fn make_decision( & mut self, decision_number : usize, decision : bool ) {
        self.mask.set_bit(      decision_number, true );
        self.decisions.set_bit( decision_number, decision );
    }

} // end impl Soluton for TwoSampleSolution



