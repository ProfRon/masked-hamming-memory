/// # The Unified Decision Optimization Algorithm with the MHD Memory
/// ## The Solution Trait
///
use std::fmt::Debug;  // or {Debug, Display}, if necessary ever again...

use mhd_memory::{ScoreType, ZERO_SCORE};

pub type PriorityType = f32; // that can change at any time, so we give it a name

pub trait Solution: Sized + Clone + Ord + Debug {

    // First, an "associated type"
    // type PriorityType: PartialOrd + Debug + Display;
    // Moved! See above...

    /// Every instance of this struct should have a descriptive name (for tracing, debugging).
    /// Default works, but is very long (override it to make it friendlier).
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

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
    fn size(&self) -> usize;

    /// `priority` is used for sorting (used in turn in the Solver trait):
    /// The definition here is a default; Solvers are free to overrule this.
    fn priority(&self) -> PriorityType;

    /// Setter function for solvers (optional)
    fn set_priority( &mut self, prio : PriorityType );

    /// Return the score stored with this solution.
    /// Note that the score is not _calculated_  -- only a problem instance can do that.
    fn get_score(&self) -> ScoreType; // score is not *calculated*! Just retrieved!

    /// Store a score for this solution.
    fn put_score(&mut self, score: ScoreType);

    /// Get the "upper" bound (which is an upper bound iff this is a maximization problem).
    fn get_best_score(&self) -> ScoreType; // "upper" bound, like score, is *not calculated* (here)!

    /// Store the "upper" bound (which is an upper bound iff this is a maximization problem).
    fn put_best_score(&mut self, best: ScoreType);

    /// This method is needed for calling the Masked Hamming Distnace functions e.g. `distance`.
    fn mask(&self) -> &[u8];
    /// This method is also needed for calling the Masked Hamming Distnace functions e.g. `distance`.
    fn query(&self) -> &[u8];

    /// Return whether this decision has been made; if not, return None,
    /// otherwise, return the decision (true of false)
    fn get_decision(&self, decision_number: usize) -> Option<bool>; // Some(bool) or None

    /// Record a decision which has been made -- unmask it and note whether true or false.
    fn make_decision(&mut self, decision_number: usize, decision: bool); // side effect: set mask bit (etc)

    /// A helper function for printing out solutions in human-readable form
    /// (default implementation provided, should suffice for concrete soutions structs)
    fn readable(&self) -> String {
        let mut result = String::new();
        for dim in 0..self.size() {
            let code = match self.get_decision(dim) {
                Some(decision) => {
                    if decision {
                        '1'
                    } else {
                        '0'
                    }
                }
                None => '?',
            };
            result = format!("{} {},", result, code); // append blank code comma to result
        }
        format!("{} score {}", result, self.get_score())
    }
} // end trait Solution

// This would have been nice but it violates "Object Safety"
// TODO Come back to this someday when I understand Object Safety
// impl std::fmt::Debug for Solution {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "[Sample: score {}, size {}, bits ", self.score, self.bytes)?;
//         for bit in 0..self.size() {
//             let code = match self.get_decision( bit ) {
//                 Ok( decision ) => if ( decision ) { '1' } else { '0' },
//                 None  => '?'
//             };// end match get_decision
//             write!(f,"{} ", code );
//         }; // end for all bits
//         write!(f, "]" );
//     } // end fn fmt
// } // end impl Debug

/// ## A Very Simple but Useful Implementation of the Solution Trait: `MinimalSolution`
///
/// Examples:
/// ```rust
/// use mhd_optimization::optimizer::{ Solution, PriorityType, MinimalSolution };
/// let sol0 = MinimalSolution::new( 8 );
/// let sol1 = MinimalSolution::random( 8 );
///
/// assert_eq!( sol0.name(), "MinimalSolution");
/// assert_eq!( sol0.get_score(), 0 );
/// // assert_eq!( sol0.get_score(), sol1.get_score() );
/// assert_eq!( sol0.get_best_score(), 0 );
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
/// sol2.put_score(      42  );
/// sol2.put_best_score( 88  );
/// assert_eq!( sol2.get_score(),      42  );
/// assert_eq!( sol2.get_best_score(), 88  );
///
/// let mut sol3 = MinimalSolution::new( 4 );
/// sol2.set_priority( 42.00 as PriorityType );
/// sol3.set_priority( 42.42 as PriorityType );
/// assert_eq!( 42.00, sol2.priority() );
/// assert_eq!( 42.42, sol3.priority() );
/// assert!( sol2 < sol3 );
/// assert!( ! (sol2 == sol3) );
/// ```
use rand::prelude::*;
use std::cmp::Ordering;

use mhd_memory::util::*; // pub fn get_bit( bytes: &[u8], bit_index: usize ) -> bool
                         // use std::fmt::Display <-- Already imported, above

#[derive(Debug, Clone)]
pub struct MinimalSolution {
    pub size: usize,
    pub mask: Vec<u8>, // we could have used Vec<u8> (twice) here (and saved two scores),
    pub decisions: Vec<u8>, // but we wouldn't have the get_bit and set_bit methods!
    pub score: ScoreType,
    pub best_score: ScoreType, // best score possible
    pub priority : PriorityType,
}

impl Solution for MinimalSolution {
    // type ScoreType = ScoreType; // mhd_memory::ScoreType;
    // type PriorityType = f32;

    // Take default .. or use this shorter version
    #[inline]
    fn name(&self) -> &'static str {
        "MinimalSolution"
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{}: score {}, best score {}",
            self.name(),
            self.get_score(),
            self.get_best_score()
        )
    }

    #[inline]
    fn new(size: usize) -> Self {
        let num_bytes = (size as f32 / 8.0).ceil() as usize;
        Self {
            size,                            // idiomatic rust for "size: size"
            mask: vec![0x0; num_bytes],      // all zeros == no decision made yet
            decisions: vec![0x0; num_bytes], // all zeros == all decisions are false (zero)
            score: ZERO_SCORE,
            best_score: ZERO_SCORE,
            priority : 0.0,
        }
    }

    #[inline]
    fn randomize(&mut self) {
        const TOP_SCORE: ScoreType = 1000;
        let mut generator = thread_rng();
        self.mask = vec![0xFF; self.mask.len()];
        generator.fill_bytes(&mut self.decisions);
        self.score = generator.gen_range(1..=TOP_SCORE); //  as ScoreType;
        self.best_score = self.score + generator.gen_range(1..=TOP_SCORE); // as ScoreType
    }

    // Getters and Setters
    #[inline]
    fn size(&self) -> usize {
        self.size
    } // times 8 bits per byte

    #[inline]
    fn priority(&self) -> PriorityType { self.priority }

    #[inline]
    fn set_priority( &mut self, prio : PriorityType ) { self.priority = prio }

    #[inline]
    fn get_score(&self) -> ScoreType {
        self.score
    }

    #[inline]
    fn put_score(&mut self, score: ScoreType) {
        self.score = score;
    }

    #[inline]
    fn get_best_score(&self) -> ScoreType {
        self.best_score
    }

    #[inline]
    fn put_best_score(&mut self, best: ScoreType) {
        self.best_score = best;
    }

    #[inline]
    fn mask(&self) -> &[u8] {
        &self.mask
    }

    #[inline]
    fn query(&self) -> &[u8] {
        &self.decisions
    }

    #[inline]
    fn get_decision(&self, decision_number: usize) -> Option<bool> {
        if !get_bit(&self.mask, decision_number) {
            None
        } else {
            // if bit is masked ==> Decision is made
            Some(get_bit(&self.decisions, decision_number))
        }
    }

    #[inline]
    fn make_decision(&mut self, decision_number: usize, decision: bool) {
        put_bit(&mut self.mask, decision_number, true);
        put_bit(&mut self.decisions, decision_number, decision);
    }
} // end impl Soluton for MinimalSolution

/// ## Default Sorting Implementations
use std::cmp::*;

// Ord requires Eq, which requires PartialEq
impl PartialEq for MinimalSolution {
    fn eq(&self, other: &Self) -> bool {
        self.priority() == other.priority()
    }
}

impl Eq for MinimalSolution {}

impl Ord for MinimalSolution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().partial_cmp(&other.priority()).expect("Ordering")
    }
}

impl PartialOrd for MinimalSolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority().partial_cmp(&other.priority())
    }
}

///////////////////// TESTs for MinimalSolution /////////////////////
#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn test_minimal_solution() {
        let sol8 = MinimalSolution::new(8);
        assert_eq!(8, sol8.size());
        assert_eq!("MinimalSolution", sol8.name());
        let sol9 = MinimalSolution::new(9);
        assert_eq!(9, sol9.size());
        let sol23 = MinimalSolution::new(23);
        assert_eq!(23, sol23.size());

        assert_eq!("MinimalSolution", sol23.name());
        assert_eq!(
            "MinimalSolution: score 0, best score 0",
            sol23.short_description()
        );

        let mut sol = MinimalSolution::new(42);
        sol.make_decision(17, true);
        assert_eq!(Some(true), sol.get_decision(17));
        assert_eq!(None, sol.get_decision(41));

        sol.put_score(42);
        sol.put_best_score(4242);
        assert_eq!(42, sol.get_score());
        assert_eq!(4242, sol.get_best_score());

        assert_eq!( 0.0, sol.priority() );
        sol.set_priority( 42.42  );
        assert_eq!( 42.42, sol.priority() );
    }
}
