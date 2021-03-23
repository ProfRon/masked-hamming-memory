// Following two constants might be turned into global variables later...
// pub const NUM_BITS: usize = 1024; // Kilobit, not yet a kilobyte....
// pub const NUM_BYTES: usize = NUM_BITS / 8; // 8 is not really a magic number, is it?

pub type ScoreType = u32; // that can change at any time, so we give it a name

pub const ZERO_SCORE: ScoreType = 0;

#[derive(Default, Clone, PartialEq)] // Debug implemented by hand, see below
pub struct Sample<const NUM_BITS: usize> {
    // pub bytes:  [u8; NUM_BYTES],
    pub bytes: Vec<u8>,   // initially empty
    pub score: ScoreType, // we will probably change that ...
} // end struct Sample

use rand::prelude::*;

/// # The `Sample` Trait (Generic?)
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
/// use mhd_mem::mhd_method::{Sample, ScoreType, ZERO_SCORE };
/// const NUM_BITS : usize = 356; // arbitrary, .... 44.5 bytes
/// let the_answer = 42 as ScoreType;
/// let row0  =  Sample::<NUM_BITS>::new( the_answer );
/// assert_eq!( row0.bytes[ 4 ], 0 );
/// assert_eq!( row0.bytes[ 44 ], 0 );
/// assert_eq!( row0.score, the_answer);
///
/// assert_eq!( row0.size(), NUM_BITS );
/// assert_eq!( row0.size_in_bytes(), 1 + NUM_BITS/8 );
///
/// let r = Sample::<NUM_BITS>::default( );
/// let s = Sample::<NUM_BITS>::new( the_answer );
/// assert_eq!( r.score, ZERO_SCORE );
/// assert_eq!( s.score, the_answer );
/// assert!( r.bytes.eq( &s.bytes ) );
///
/// let rr = Sample::<NUM_BITS>::random( );
/// assert_ne!( r, rr );
/// // That last test could fail, by dumb luck, but it's nearly impossible...
///
/// let mut row1 = Sample::<NUM_BITS>::default();
/// assert_eq!( row1.get_bit( 42 ), false ); // should be 0
/// row1.set_bit( 42, true );
/// assert!( row1.get_bit( 42 ) );
/// row1.set_bit( 42, false );
/// assert!( ! row1.get_bit( 42 ) );
///
/// let row_ff = Sample::<NUM_BITS>::new_ones( the_answer );
/// assert_eq!( row_ff.score, the_answer );
/// assert!( row_ff.get_bit(  0 ) ); // should be 1
/// assert!( row_ff.get_bit(  7 ) ); // should be 1
/// assert!( row_ff.get_bit( 32 ) ); // should be 1
/// assert!( row_ff.get_bit( 42 ) ); // should be 1
/// assert!( row_ff.get_bit( 63 ) ); // should be 1
/// ```
///
impl<const NUM_BITS: usize> Sample<NUM_BITS> {
    // calculate ceil( Num_bits / 8 ) without floating point cast...
    pub const NUM_BYTES: usize = (NUM_BITS / 8) + if 0 == NUM_BITS % 8 { 0 } else { 1 };

    pub fn size(&self) -> usize {
        NUM_BITS
    }

    pub fn size_in_bytes(&self) -> usize {
        Self::NUM_BYTES
    }

    fn assert_size_is_legal() {
        assert!(3 < NUM_BITS);
        assert!(0 < Self::NUM_BYTES); // zero is illegal
        assert!(Self::NUM_BYTES <= 1024 * 1024); // this will probably change
        debug_assert!(8 * Self::NUM_BYTES - NUM_BITS < 8);
    }

    pub fn default() -> Self {
        Self {
            // bytes : [0;  Self::NUM_BYTES ],
            bytes: vec![0x0; Self::NUM_BYTES], // start with an empty vector of bytes
            score: ZERO_SCORE,
        }
    }

    pub fn new(starting_score: ScoreType) -> Self {
        Self::assert_size_is_legal();
        Self {
            score: starting_score,
            bytes: vec![0x0; Self::NUM_BYTES], // start with an empty vector of bytes
        }
    }

    pub fn new_ones(starting_score: ScoreType) -> Self {
        Self::assert_size_is_legal();
        Self {
            score: starting_score,
            bytes: vec![0xFF; Self::NUM_BYTES], // start with an empty vector of bytes
        }
    }

    pub fn randomize(&mut self) {
        // first a random score
        self.score = rand::thread_rng().gen_range( 0..1_001 ); // TODO -- add constant
        // then some random bytes
        rand::thread_rng().fill_bytes(&mut self.bytes);
    }

    pub fn random() -> Self {
        let mut result = Self::default();
        result.randomize();
        result
    }

    pub fn byte_index(bit_index: usize) -> usize {
        let byte_index = bit_index / 8;
        debug_assert!(byte_index < Self::NUM_BYTES);
        byte_index
    }

    pub fn get_bit(&self, bit_index: usize) -> bool {
        let byte_index = Self::byte_index(bit_index);

        let byte = self.bytes[byte_index];
        let mask_index = bit_index % 8;
        let bit_mask = 128 >> mask_index;
        0 != (byte & bit_mask)
    }

    pub fn set_bit(&mut self, bit_index: usize, bit_value: bool) {
        let byte_index = Self::byte_index(bit_index);

        let mask_index = bit_index % 8;
        let bit_mask = 128 >> mask_index;

        if bit_value {
            self.bytes[byte_index] |= bit_mask;
        } else {
            self.bytes[byte_index] &= !bit_mask;
        };
    }
} // end impl Sample

impl<const NUM_BITS: usize> std::fmt::Debug for Sample<NUM_BITS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Sample: score {}, bytes{:x?}]", self.score, self.bytes)
    }
}

// TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constructors() {
        Sample::<4>::assert_size_is_legal();
        Sample::<8>::assert_size_is_legal();
        Sample::<42>::assert_size_is_legal();
        Sample::<64>::assert_size_is_legal();
        Sample::<100>::assert_size_is_legal();
        Sample::<800>::assert_size_is_legal();
        Sample::<64_000>::assert_size_is_legal();
        Sample::<{ 1024 * 1024 }>::assert_size_is_legal(); // suddenly rustc WANTS brackes?!?

        const NUM_TEST_BITS: usize = 64;
        let r = Sample::<NUM_TEST_BITS>::default();
        assert_eq!(r.bytes[0], 0); // should be 0
        assert_eq!(r.bytes[7], 0); // should be 0
        assert_eq!(r.score, ZERO_SCORE); // should be 0
        assert_eq!(r.size(), NUM_TEST_BITS);

        let s = Sample::<NUM_TEST_BITS>::new(42);
        assert_eq!(s.score, 42 as ScoreType); // should NOT be 0
        assert!(r.score != s.score);
        // assert!(r.bytes == s.bytes);
        assert!(r.bytes.eq(&s.bytes));

        let q = Sample::<NUM_TEST_BITS>::random();
        assert!(r.score != q.score); // with very high probability
        assert_ne!(q, r); // with very high probability

        let t = Sample::<NUM_TEST_BITS>::new_ones(0);
        assert_eq!(t.bytes[0], 0xFF); // should be 0
        assert_eq!(t.bytes[3], 0xff); // should be 0
        assert_eq!(t.bytes[7], 0xFF); // should be 0

        const MORE_TEST_BITS: usize = NUM_TEST_BITS + 4; // = 68
        const MORE_TEST_BYTES: usize = (MORE_TEST_BITS / 8) + 1; // = 9
        let u = Sample::<MORE_TEST_BITS>::new_ones(0);
        assert_eq!(MORE_TEST_BITS, u.size());
        assert_eq!(MORE_TEST_BYTES, u.size_in_bytes());
    } // end test_contructors

    #[test]
    fn test_methods() {
        const NUM_TEST_BITS: usize = 64;
        let mut row1 = Sample::<NUM_TEST_BITS>::default();
        assert_eq!(row1.get_bit(62), false); // should be 0
        row1.set_bit(62, true);
        assert!(row1.get_bit(62));
        row1.set_bit(62, false);
        assert!(!row1.get_bit(62));
    } // end test_methods

    #[test]
    fn test_randomization() {
        // Note: This test could fail due to dumb luck.
        // That's very improbable, but _could_ happen.

        // Check that randomize() changes its argument
        const NUM_TEST_BITS: usize = 1000; // not always a power of two...
        let starting_point = Sample::<NUM_TEST_BITS>::default();
        let mut one_step = starting_point.clone();
        one_step.randomize();
        assert_ne!(starting_point, one_step);

        // Check that re-randomizing changes the sample
        let mut two_steps = one_step.clone();
        two_steps.randomize();
        assert_ne!(one_step, two_steps);

        // Check that calling the `random` constructor
        // gives a fresh sample.
        let three_steps = Sample::<NUM_TEST_BITS>::random();
        assert_ne!(three_steps, starting_point);
        assert_ne!(two_steps, three_steps);

        // Check that calling the `random` constructor
        // doesn't always return the same result.
        let final_point = Sample::<NUM_TEST_BITS>::random();
        assert_ne!(three_steps, final_point);
    }
} // end mod tests
