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

#[derive(Debug)]
pub struct MHDMemoryRow {
    bits:  [ u8; 128 ],
    pub value: f32,
 }

impl MHDMemoryRow {

    pub fn new( value: f32 ) -> MHDMemoryRow {
        MHDMemoryRow { bits : [ 0; 128 ], value: value }
    }

    pub fn get_bit(&mut self, index: usize ) -> bool {
        let byte_index = index / 8;
        assert!( byte_index < self.bits.len() );

        let byte = self.bits[byte_index];
        let bit_index = index % 8;
        let bit_mask = 128 >> bit_index;
        return 0 != (byte & bit_mask);
    }

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
    }


} // end impl MHDMemoryRow


// TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS

#[cfg(test)]
mod tests {

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn naive_smoke() {

        let mut row1 = MHDMemoryRow::new( 4.2 );
        assert!( ! row1.get_bit( 42 ) );
        row1.set_bit( 42, true );
        assert!( row1.get_bit( 42 ) );
        row1.set_bit( 42, false );
        assert!( ! row1.get_bit( 42 ) );
    } // end naive_smoke

} // end mod tests
