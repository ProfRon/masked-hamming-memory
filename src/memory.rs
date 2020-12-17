///  The MHD Memory (see literature list)
///  is an associative memory, i.e. it is content adressable, and maps content,
///  in the form of a binary vectors, onto an abritrary "right hand side" (RHS).
///  In the original literature, the RHS was usually a category ID, and the memory
///  was used for pattern recognition.
///
///  This experimental version maps binary vectors onto a *score*, which is simply a
///  floating point (real) numbers.  More on this below.
///
///  Conclusion: The memory consists of a number of rows, and a `read` and a `write` operation.
///  A *row* is a 2-tuple consisting of a binary vector (a vector of bits) and a RHS, i.e. a float
///  The 'write' operation takes as an argument a new row (which in the simplest case is added to
///  the memory -- any complications above and beyond that are invisible to the user.
///  The `read` operation takes as an argument a binary vector, and returns a calculated RHS.


/// First -- the row type, which contains bits (the left hand side) and a float value (RHS).
/// Further, since bits are not easy to address individually, a getter and a setter are provided.
/// ```rust
/// let mut row1 = mhd_mem::MHDMemoryRow::new( 4.2 );
///  assert!( ! row1.get_bit( 42 ) );
///  row1.set_bit( 42, true );
///  assert!( row1.get_bit( 42 ) );
///  row1.set_bit( 42, false );
///  assert!( ! row1.get_bit( 42 ) );
///  assert_eq!( row1.value, 4.2 )
/// ```
/// There is also a `distance_for` method for calling the masked hamming distance function
/// ```rust
/// use mhd_mem::MHDMemoryRow;
/// let mut row1 = MHDMemoryRow::new( 4.2 );
/// let mut row2 = MHDMemoryRow::new( 2.4 );
/// row1.set_bit( 42, true );
/// row1.set_bit( 24, true );
/// let mask_vector = [0xFF as u8; MHDMemoryRow::NUM_BYTES ];
/// assert_eq!( 2, row1.distance_from( &mask_vector, &row2 ) );
/// assert_eq!( 2, row2.distance_from( &mask_vector, &row1 ) );
/// ```

#[derive(Debug)]
pub struct MHDMemoryRow {
    bits:  [ u8; MHDMemoryRow::NUM_BYTES ],
    pub value: f32,
 }

use ::distance;

impl MHDMemoryRow {

    pub const NUM_BITS  : usize = 1024;
    pub const NUM_BYTES : usize = MHDMemoryRow::NUM_BITS / 8;

    pub fn new( value: f32 ) -> MHDMemoryRow {
        MHDMemoryRow { bits : [ 0; MHDMemoryRow::NUM_BYTES ], value: value }
    }

    pub fn get_bit(&self, index: usize ) -> bool {
        let byte_index = index / 8;
        assert!( byte_index < self.bits.len() );

        let byte = self.bits[byte_index];
        let bit_index = index % 8;
        let bit_mask = 0b1000_0000 >> bit_index;
        return 0 != (byte & bit_mask);
    } // end get_bit

    pub fn set_bit(&mut self, index: usize, bit_value: bool ) {
        let byte_index = index / 8;
        assert!( byte_index < self.bits.len() );

        let bit_index = index % 8;
        let bit_mask = 128 >> bit_index;

        if bit_value {
            self.bits[byte_index] |= bit_mask;
        } else {
            self.bits[byte_index] &= !bit_mask;
        };
    } // end set_bit

    pub fn distance_from( &self, mask: &[u8; MHDMemoryRow::NUM_BYTES ],  other : &MHDMemoryRow ) -> u64 {
        return distance( mask, &self.bits, &other.bits );
    } // end distance_from( other )

} // end impl MHDMemoryRow


// TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS

#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn naive_smoke() {
        let mut row1 = MHDMemoryRow::new( 4.2 );
        assert_eq!( row1.value, 4.2 );
        let mut row2 = MHDMemoryRow::new( 2.4 );
        assert_eq!( row2.value, 2.4 );
        let mut mask_vector = [0xFF as u8; MHDMemoryRow::NUM_BYTES ];
        assert_eq!( 0, row1.distance_from( &mask_vector, &row2 ) );
        row1.set_bit( 24, true );
        row2.set_bit( 42, true );
        row2.set_bit( 24, true );
        assert_eq!( 1, row2.distance_from( &mask_vector, &row1 ) );
        mask_vector[ 42 / 8 ] = 0;
        assert_eq!( 0, row2.distance_from( &mask_vector, &row1 ) );
    } // end naive_smoke

} // end mod tests

