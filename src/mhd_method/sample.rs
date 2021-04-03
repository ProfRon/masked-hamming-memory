// Following two constants might be turned into global variables later...
// pub const NUM_BITS: usize = 1024; // Kilobit, not yet a kilobyte....
// pub const NUM_BYTES: usize = NUM_BITS / 8; // 8 is not really a magic number, is it?

pub type ScoreType = u32; // that can change at any time, so we give it a name

pub const ZERO_SCORE: ScoreType = 0;

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
/// let row0  =  Sample::new( NUM_BITS, the_answer );
/// assert_eq!( row0.bytes[ 4 ], 0 );
/// assert_eq!( row0.bytes[ 44 ], 0 );
/// assert_eq!( row0.score, the_answer);
///
/// assert_eq!( row0.size(), NUM_BITS );
/// assert_eq!( row0.size_in_bytes(), 1 + NUM_BITS/8 );
///
/// let r = Sample::default( );
/// assert_eq!( r.score, ZERO_SCORE );
/// let s = Sample::new( NUM_BITS, the_answer );
/// assert_eq!( s.score, the_answer );
///
/// let rr = Sample::random( NUM_BITS );
/// assert_ne!( r, rr );
///
/// let mut row1 = Sample::new(NUM_BITS, ZERO_SCORE);
/// assert_eq!( row1.get_bit( 42 ), false ); // should be 0
/// row1.set_bit( 42, true );
/// assert!( row1.get_bit( 42 ) );
/// row1.set_bit( 42, false );
/// assert!( ! row1.get_bit( 42 ) );
///
/// let row_ff = Sample::new_ones( NUM_BITS, the_answer );
/// assert_eq!( row_ff.score, the_answer );
/// assert!( row_ff.get_bit(  0 ) ); // should be 1
/// assert!( row_ff.get_bit(  7 ) ); // should be 1
/// assert!( row_ff.get_bit( 32 ) ); // should be 1
/// assert!( row_ff.get_bit( 42 ) ); // should be 1
/// assert!( row_ff.get_bit( 63 ) ); // should be 1
/// ```
///
#[derive(Default, Clone, PartialEq)] // Debug implemented by hand, see below
pub struct Sample {
    // pub bytes:  [u8; NUM_BYTES],
    pub width: usize,
    pub bytes: Vec<u8>,   // initially empty
    pub score: ScoreType, // we will probably change that ...
} // end struct Sample

use rand::prelude::*;

impl Sample {
    // calculate ceil( size_in_bits / 8 ) without floating point cast...
    #[inline]
    fn bits_to_bytes(size_in_bits: usize) -> usize {
        (size_in_bits / 8) + if 0 == (size_in_bits % 8) { 0 } else { 1 }
    }

    #[inline]
    pub fn size_in_bytes(&self) -> usize {
        debug_assert_eq!(self.bytes.len(), Self::bits_to_bytes(self.width));
        self.bytes.len()
    }

    #[inline]
    pub fn size(&self) -> usize {
        debug_assert_eq!(self.bytes.len(), Self::bits_to_bytes(self.width));
        self.width
    }

    #[inline]
    fn size_is_legal(size_in_bits: usize) -> bool {
        let size_in_bytes = Self::bits_to_bytes(size_in_bits);
        (3 < size_in_bits) && (size_in_bytes <= 1024 * 1024) // this is subject to change
    }

    #[inline]
    pub fn default() -> Self {
        const DEFAULT_CAPACITY: usize = 8; // bytes = 64 bits
        Self {
            width: 0,
            // bytes : [0;  Self::NUM_BYTES ],
            bytes: Vec::with_capacity(DEFAULT_CAPACITY),
            score: ZERO_SCORE,
        }
    }

    #[inline]
    pub fn new(size_in_bits: usize, starting_score: ScoreType) -> Self {
        debug_assert!(Self::size_is_legal(size_in_bits));
        Self {
            width: size_in_bits,
            score: starting_score,
            bytes: vec![0x0; Self::bits_to_bytes(size_in_bits)], // start with an empty vector of bytes
        }
    }

    #[inline]
    pub fn new_ones(size_in_bits: usize, starting_score: ScoreType) -> Self {
        debug_assert!(Self::size_is_legal(size_in_bits));
        Self {
            width: size_in_bits,
            score: starting_score,
            bytes: vec![0xFF; Self::bits_to_bytes(size_in_bits)], // start with an empty vector of bytes
        }
    }

    #[inline]
    pub fn randomize(&mut self) {
        // First a random score
        const MAX_RANDOM_SCORE: ScoreType = 1000; // seems to work out OK....
        self.score = rand::thread_rng().gen_range(0..=MAX_RANDOM_SCORE);
        // Then some random bytes
        // Note -- length of bytes vector is not changed!
        rand::thread_rng().fill_bytes(&mut self.bytes);
    }

    #[inline]
    pub fn random(size_in_bits: usize) -> Self {
        debug_assert!(Self::size_is_legal(size_in_bits));
        let mut result = Self::new(size_in_bits, ZERO_SCORE);
        result.randomize();
        result
    }

    #[inline]
    pub fn byte_index(bit_index: usize) -> usize {
        bit_index / 8
    }

    #[inline]
    pub fn get_bit(&self, bit_index: usize) -> bool {
        let byte_index = Self::byte_index(bit_index);
        let byte = self.bytes[byte_index];
        let mask_index = bit_index % 8;
        let bit_mask = 128 >> mask_index;
        0 != (byte & bit_mask)
    }

    #[inline]
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

impl std::fmt::Debug for Sample {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Sample: score {}, bytes{:x?}]", self.score, self.bytes)
    }
}

// TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_evcxr() {
        pub struct TestArray<T, const LENGTH: usize> {
            //          ^^^^^^^^^^^^^^^^^^^ Const generic definition.
            pub list: [T; LENGTH], //        ^^^^^^ We use it here.
        }

        let test_array = TestArray::<u8, 42> { list: [0u8; 42] };

        assert_eq!(test_array.list.len(), 42);
    }

    #[test]
    fn test_constructors() {
        const NUM_TEST_BITS: usize = 64;
        let r = Sample::new(NUM_TEST_BITS, ZERO_SCORE);
        assert_eq!(r.bytes[0], 0); // should be 0
        assert_eq!(r.bytes[7], 0); // should be 0
        assert_eq!(r.score, ZERO_SCORE); // should be 0
        assert_eq!(r.size(), NUM_TEST_BITS);

        let s = Sample::new(NUM_TEST_BITS, 42);
        assert_eq!(s.score, 42 as ScoreType); // should NOT be 0
        assert!(r.score != s.score);
        // assert!(r.bytes == s.bytes);
        assert!(r.bytes.eq(&s.bytes));

        let q = Sample::random(NUM_TEST_BITS);
        assert!(r.score != q.score); // with very high probability
        assert_ne!(q, r); // with very high probability

        let t = Sample::new_ones(NUM_TEST_BITS, ZERO_SCORE);
        assert_eq!(t.bytes[0], 0xFF); // should be 0
        assert_eq!(t.bytes[3], 0xff); // should be 0
        assert_eq!(t.bytes[7], 0xFF); // should be 0

        const MORE_TEST_BITS: usize = NUM_TEST_BITS + 4; // = 68
        const MORE_TEST_BYTES: usize = (MORE_TEST_BITS / 8) + 1; // = 9
        let u = Sample::new_ones(MORE_TEST_BITS, ZERO_SCORE);
        assert_eq!(MORE_TEST_BITS, u.size());
        assert_eq!(MORE_TEST_BYTES, u.size_in_bytes());
    } // end test_contructors

    #[test]
    fn test_methods() {
        const NUM_TEST_BITS: usize = 64;
        let mut row1 = Sample::new(NUM_TEST_BITS, ZERO_SCORE);
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
        let starting_point = Sample::new(NUM_TEST_BITS, ZERO_SCORE);
        let mut one_step = starting_point.clone();
        one_step.randomize();
        assert_ne!(starting_point, one_step);

        // Check that re-randomizing changes the sample
        let mut two_steps = one_step.clone();
        two_steps.randomize();
        assert_ne!(one_step, two_steps);

        // Check that calling the `random` constructor
        // gives a fresh sample.
        let three_steps = Sample::random(NUM_TEST_BITS);
        assert_ne!(three_steps, starting_point);
        assert_ne!(two_steps, three_steps);

        // Check that calling the `random` constructor
        // doesn't always return the same result.
        let final_point = Sample::random(NUM_TEST_BITS);
        assert_ne!(three_steps, final_point);
    }
} // end mod tests
