/// # The Unified Decision Optimization Algorithm with the MHD Memory
/// ## The Solution Trait
///
use std::fmt::Debug;

use mhd_method::{ScoreType, NUM_BITS};

pub trait Solution: Sized + Clone + Ord + Debug {
    // First, an "associated type"
    // Compare <file:///home/ron/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/share/doc/rust/html/book/ch19-03-advanced-traits.html>
    // type ScoreType: PartialOrd + Debug + Display;

    /// every instance of this struct should have a descriptive name (for tracing, debugging)
    /// TODO: Remove this when <https://doc.rust-lang.org/std/any/fn.type_name_of_val.html> stable
    fn name(&self) -> &'static str;

    /// Every instance should have a SHORT description for Debugging,
    /// giving things like a knapsack's weight, etc.
    fn short_description(&self) -> String;

    /// Constructor for a "blank" solution (with no decisions made yet) where
    /// size is the number of decisions to be made (free variables to assign values to).
    fn new(size: usize) -> Self;

    /// Constructor for a complete random solution, where
    /// size is the number of decisions to be made (free variables to assign values to).
    fn random(size: usize) -> Self {
        let mut result = Self::new(size);
        result.randomize();
        result
    }

    /// `randomize` takes a solution and sets all the open decisions at random.
    /// This does NOT mean that the mask is randomized -- it is set to all ones.
    /// Note that this will almost never produce a valid, legal solution to any given problem,
    /// which is why each problem implementation has its own `random_solution` method,
    /// but these usually call Solution::randomize( self ) as a starting step.
    fn randomize(&mut self);

    /// #  Getters and Setters
    /// size, dimension, number of decisions which can be made.
    fn size( &self ) -> usize;

    /// `estimate` is used for sorting (used in turn in the Solver trait):
    /// Note that it works with get_score() and get_best_score() -- and NO problems-specific
    /// information!
    fn estimate(&self) -> ScoreType {
        (self.get_score() + self.get_best_score()) / 2
    }

    /// Return the score stored with this solution.
    /// Note that the score is not _calculated_  -- only a problem instance can do that.
    fn get_score(&self) -> ScoreType; // score is not *calculated*! Just retrieved!

    /// Store a score for this solution.
    fn put_score(&mut self, score: ScoreType);

    /// Get the "upper" bound (which is an upper bound iff this is a maximization problem).
    fn get_best_score(&self) -> ScoreType; // "upper" bound, like score, is *not calculated* (here)!

    /// Store the "upper" bound (which is an upper bound iff this is a maximization problem).
    fn put_best_score(&mut self, best: ScoreType);

    /// Return whether this decision has been made; if not, return None,
    /// otherwise, return the decision (true of false)
    fn get_decision(&self, decision_number: usize) -> Option<bool>; // Some(bool) or None

    /// Record a decision which has been made -- unmask it and note whether true or false.
    fn make_decision(&mut self, decision_number: usize, decision: bool); // side effect: set mask bit (etc)

    /// A helper function for printing out solutions in human-readable form
    /// (default implementation provided, should suffice for concrete soutions structs)
    fn readable( &self ) -> String {
        let mut result = String::new();
        for dim in 0..self.size() {
            let code = match self.get_decision( dim ){
                Some( decision) => if decision { '1' } else { '0' },
                None => '?',
            };
            result = format!( "{} {},", result, code ); // append blank code comma to result
        };
        format!( "{} score {}", result, self.get_score() )
    }
} // end trait Solution

/// ## A Very Simple but Useful Implementation of the Solution Trait: `MinimalSolution`
///
/// Examples:
/// ```rust
/// use mhd_mem::mhd_method::{ ScoreType, ZERO_SCORE };
/// use mhd_mem::mhd_optimizer::{ Solution, MinimalSolution };
/// let sol0 = MinimalSolution::new( 8 );
/// let sol1 = MinimalSolution::random( 8 );
///
/// assert_eq!( sol0.name(), "MinimalSolution");
/// assert_eq!( sol0.get_score(), ZERO_SCORE );
/// // assert_eq!( sol0.get_score(), sol1.get_score() );
/// assert_eq!( sol0.get_best_score(), ZERO_SCORE );
/// // assert_eq!( sol0.get_best_score(), sol1.get_best_score() );
/// assert_eq!( sol0.get_decision( 0 ), None );
///
/// let mut sol2 = MinimalSolution::new( 4 );
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
/// let mut sol3 = MinimalSolution::new( 4 );
/// sol3.put_score(      64 as ScoreType );
/// sol3.put_best_score( 88 as ScoreType );
/// assert!( sol2 < sol3 );
/// assert!( ! (sol2 == sol3) );
/// ```
use rand::prelude::*;
use std::cmp::Ordering;

use mhd_method::util::*; // pub fn get_bit( bytes: &[u8], bit_index: usize ) -> bool
                         // use std::fmt::Display <-- Already imported, above

#[derive(Debug, Clone)]
pub struct MinimalSolution {
    pub size: usize,
    pub mask: Vec<u8>, // we could have used Vec<u8> (twice) here (and saved two scores),
    pub decisions: Vec<u8>, // but we wouldn't have the get_bit and set_bit methods!
    pub score: ScoreType,
    pub best_score: ScoreType, // best score possible
}

impl Solution for MinimalSolution {
    // type ScoreType = ScoreType; // mhd_method::ScoreType;

    fn name(&self) -> &'static str {
        "MinimalSolution"
    }

    fn short_description(&self) -> String {
        format!(
            "{}: score {}, best score {}",
            self.name(),
            self.get_score(),
            self.get_best_score()
        )
    }

    fn new(size: usize) -> Self {
        assert!(size <= NUM_BITS);
        let num_bytes = (size as f32 / 8.0).ceil() as usize;
        Self {
            size: size,
            mask: vec![0x0; num_bytes],      // all zeros == no decision made yet
            decisions: vec![0x0; num_bytes], // all zeros == all decisions are false (zero)
            score: 0 as ScoreType,
            best_score: 0 as ScoreType,
        }
    }

    fn randomize(&mut self) {
        const TOP_SCORE: ScoreType = 1000;
        let mut generator = thread_rng();
        self.mask = vec![0xFF; self.mask.len()];
        generator.fill_bytes(&mut self.decisions);
        self.score = generator.gen_range(1..=TOP_SCORE); //  as ScoreType;
        self.best_score = self.score + generator.gen_range(1..=TOP_SCORE); // as ScoreType
    }

    // Getters and Setters
    fn size(&self) -> usize { self.size } // times 8 bits per byte

    fn get_score(&self) -> ScoreType {
        self.score
    }

    fn put_score(&mut self, score: ScoreType) {
        self.score = score;
    }

    fn get_best_score(&self) -> ScoreType {
        self.best_score
    }

    fn put_best_score(&mut self, best: ScoreType) {
        self.best_score = best;
    }

    fn get_decision(&self, decision_number: usize) -> Option<bool> {
        if !get_bit(&self.mask, decision_number) {
            None
        } else {
            // if bit is masked ==> Decision is made
            Some(get_bit(&self.decisions, decision_number))
        }
    }

    fn make_decision(&mut self, decision_number: usize, decision: bool) {
        put_bit(&mut self.mask, decision_number, true);
        put_bit(&mut self.decisions, decision_number, decision);
    }
} // end impl Soluton for MinimalSolution

/// ## Default Sorting Implementations (hopefully allowed)
use std::cmp::*;

// Ord requires Eq, which requires PartialEq
impl PartialEq for MinimalSolution {
    fn eq(&self, other: &Self) -> bool {
        self.get_score() == other.get_score()
    }
}

impl Eq for MinimalSolution {}

impl Ord for MinimalSolution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.get_score().cmp(&other.get_score())
    }
}

impl PartialOrd for MinimalSolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.get_score().cmp(&other.get_score()))
    }
}

///////////////////// TESTs for MinimalSolution /////////////////////
#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn test_minimal_solution() {
        let sol8 = MinimalSolution::new( 8 );
        assert_eq!( 8, sol8.size() );
        let sol9 = MinimalSolution::new( 9 );
        assert_eq!( 9, sol9.size() );
        let sol23 = MinimalSolution::new( 23 );
        assert_eq!( 23, sol23.size() );

        assert_eq!( "MinimalSolution", sol23.name() );
        assert_eq!( "MinimalSolution: score 0, best score 0", sol23.short_description() );

        let mut sol = MinimalSolution::new( 42 );
        sol.make_decision( 17, true );
        assert_eq!( Some(true), sol.get_decision( 17 ));
        assert_eq!( None, sol.get_decision( 41 ));

        sol.put_score( 42 );
        sol.put_best_score( 4242 );
        assert_eq!( 42, sol.get_score() );
        assert_eq!( 4242, sol.get_best_score() );
    }
}
