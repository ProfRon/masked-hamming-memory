/// # The `Sample` Data Type
///
/// The Sample data type is used to build the MHD Memory, a
/// and also used to define problems and algorithms to solve those problems.
///
/// ## Definition
/// The Sample is simply an ordered pair of a bit vector and a floating point value.
/// We call the real numbers on the "right hand side" the *score*.
/// The goal is to find the bit vector with best score.
/// Without loss of generality, we take the best score to be the largest score:
/// If you are actually trying to find the minimal score, multiply the "real" scores by -1.
///
/// ## Note on Terminology:
/// It is perhaps more customary to call the `samples` _rows_.
/// However, we will later define types for algorithms and for problems,
/// and we want to reuse the `sample` datatype there -- and there,
/// `samples` makes more sense than _rows_.
///
/// ## Note to self:
/// Once we get many, many rows, should RAM become a problem, we might want to consider faking the floats.
/// For example, we could perhaps get by with only 8 bits (i.e. taking an unsigned byte divided by 256, or a signed byte divided by 128.
///
/// ## Examples:
/// ```rust
/// use mhd_mem::{ScoreType, ZERO_SCORE };
/// let the_answer = 42 as ScoreType;
/// let row0  =  mhd_mem::Sample { bytes : vec![0x0;  mhd_mem::NUM_BYTES ], score : the_answer };
/// assert_eq!( row0.bytes[ 4 ], 0 );
/// assert_eq!( row0.score, the_answer);
///
/// let r = mhd_mem::Sample::default( );
/// let s = mhd_mem::Sample::new( the_answer );
/// assert_eq!( r.score, ZERO_SCORE );
/// assert_eq!( s.score, the_answer );
/// assert!( r.bytes.eq( &s.bytes ) );
///
/// let rr = mhd_mem::Sample::random( );
/// assert_ne!( r, rr );
/// // That last test could fail, by dumb luck, but it's nearly impossible...
///
/// let mut row1 = mhd_mem::Sample::default();
/// assert_eq!( row1.get_bit( 42 ), false ); // should be 0
/// row1.set_bit( 42, true );
/// assert!( row1.get_bit( 42 ) );
/// row1.set_bit( 42, false );
/// assert!( ! row1.get_bit( 42 ) );
/// ```
///

// Following two constants might be turned into global variables later...
pub const NUM_BITS:  usize = 256; // enough for testing, but not too many...
pub const NUM_BYTES: usize = NUM_BITS / 8; // 8 is not really a magic number, is it?

pub type ScoreType = i32; // that can change at any time, so we give it a name

pub const ZERO_SCORE : ScoreType = 0;

#[derive(Debug,Default,Clone,PartialEq)]
pub struct Sample {
    // pub bytes:  [u8; NUM_BYTES],
    pub bytes : Vec< u8 >, // initially empty
    pub score : ScoreType // we will probably change that ...
} // end struct Sample

use rand::prelude::*;

impl Sample {

    pub fn default() -> Self {
        Sample {
            // bytes : [0;  NUM_BYTES ],
            bytes : vec![ 0x0; NUM_BYTES ], // start with an empty vector of bytes
            score : ZERO_SCORE,
        }
    }

    pub fn new( starting_score: ScoreType ) -> Self {
        Sample {
            score : starting_score,
            bytes : vec![ 0x0; NUM_BYTES ], // start with an empty vector of bytes
        }
    }

    pub fn randomize( &mut self ) {
        let random_byte : i8 = rand::thread_rng().gen(); // can be negative, so -128 <= rb < 128
        self.score = random_byte as ScoreType;
        rand::thread_rng().fill_bytes( &mut self.bytes  );
    }

    pub fn random( ) -> Self {
        let mut result = Sample::default();
        result.randomize();
        result
    }

    pub fn byte_index( bit_index: usize ) -> usize {
        let byte_index = bit_index / 8;
        assert!( byte_index <  NUM_BYTES );
        return byte_index;
    }

    pub fn get_bit( &mut self, bit_index: usize ) -> bool {

        let byte_index = Sample::byte_index( bit_index );

        let byte = self.bytes[ byte_index ];
        let mask_index = bit_index % 8;
        let bit_mask = 128 >> mask_index;
        return 0 != (byte & bit_mask);
    }

    pub fn set_bit(&mut self, bit_index: usize, bit_value: bool ) {

        let byte_index = Sample::byte_index( bit_index );

        let mask_index = bit_index % 8;
        let bit_mask = 128 >> mask_index;

        if bit_value {
            self.bytes[  byte_index ] |=  bit_mask;
        } else {
            self.bytes[  byte_index ] &= !bit_mask;
        };
    }

} // end impl Sample

// TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        let r = Sample::default();
        assert_eq!(r.bytes[0], 0); // should be 0
        assert_eq!(r.bytes[7], 0); // should be 0
        assert_eq!(r.score, ZERO_SCORE ); // should be 0

        let s = Sample::new(42 );
        assert_eq!(s.score, 42 as ScoreType ); // should NOT be 0
        assert!(r.score != s.score);
        // assert!(r.bytes == s.bytes);
        assert!( r.bytes.eq( &s.bytes ) );

        let q = Sample::random( );
        assert!(r.score != q.score);  // with very high probability
        assert_ne!( q, r );           // with very high probability

    } // end test_contructors

    #[test]
    fn test_methods() {

        let mut row1 = Sample::default();
        assert_eq!(row1.get_bit(62), false); // should be 0
        row1.set_bit(62, true);
        assert!( row1.get_bit(62));
        row1.set_bit(62, false);
        assert!(! row1.get_bit(62) );

    } // end test_methods

    #[test]
    fn test_randomization() {
        // Note: This test could fail due to dumb luck.
        // That's very improbable, but _could_ happen.
        
        // Check that randomize() changes its argument
        let starting_point = Sample::default();
        let mut one_step = starting_point.clone();
        one_step.randomize();
        assert_ne!( starting_point, one_step );
        
        // Check that re-randomizing changes the sample
        let mut two_steps = one_step.clone();
        two_steps.randomize();
        assert_ne!( one_step, two_steps );
        
        // Check that calling the `random` constructor 
        // gives a fresh sample.
        let three_steps = Sample::random();
        assert_ne!( three_steps, starting_point );
        assert_ne!( two_steps,   three_steps );

		// Check that calling the `random` constructor 
		// doesn't always return the same result.
        let final_point = Sample::random();
        assert_ne!( three_steps, final_point );
    }
} // end mod tests



