fn naive(mask:&[u8], x: &[u8], y: &[u8]) -> u64 {
    assert_eq!(mask.len(), x.len());
    assert_eq!(x.len(),    y.len());
    mask.iter().zip( x.iter().zip(y) ).fold(0, |a, (m, (b, c)) | a + (*m & (*b ^ *c)).count_ones() as u64)
}

#[derive(Debug, PartialEq, Eq, Ord, PartialOrd, Hash, Clone)]
pub struct DistanceError {
    _x: ()
}

/// Computes the bitwise [Hamming
/// distance](https://en.wikipedia.org/wiki/Hamming_distance) between
/// `x` and `y`, that is, the number of bits where `x` and `y` differ,
/// or, the number of set bits in the xor of `x` and `y`...
/// ... or rather, the  _masked_ hamming distance, i.e. counting 
///     only the differing bits that are also 1 in the mask (!).
///
/// The text below ignores the use of the mask. Keep it in mind.
/// 
/// This is a highly optimised version of the following naive version:
///
/// ```rust
/// fn naive(mask: &[u8], x: &[u8], y: &[u8]) -> u64 {
///   mask.iter().zip( x.iter().zip(y) )
///       .fold(0, |a, (m, (b, c)) | a + (*m & (*b ^ *c)).count_ones() as u64)
/// }
/// ```
///
/// This function requires that `x` and `y` have the same 8-byte
/// alignment. If not, `Err` is returned. If sub-optimal performance
/// can be tolerated, consider using `distance` which incorporates a
/// fallback to a slower but less restrictive algorithm.
///
/// It is essentially guaranteed that `x` and `y` will have the same
/// 8-byte alignment if they are both just `Vec<u8>`s of non-trivial
/// length (e.g. larger than 8) as in the example below.
///
/// This is implemented using the same tree-merging approach as
/// `weight`, see there for details.
///
/// # Panics
///
/// `x` and `y` must have the same length, or else `distance_fast` panics.
///
/// # Performance Comparison
///
/// | length | `naive` (ns) | `distance_fast` (ns) | `naive`/`distance_fast` |
/// |--:|--:|--:|--:|
/// | 1 | 5  | 6  | 0.83 |
/// | 10 | 44  | 45  | 0.97 |
/// | 100 | 461  | 473  | 0.97 |
/// | 1,000 | 4,510  | 397  | 11 |
/// | 10,000 | 46,700  | 2,740  | 17 |
/// | 100,000 | 45,600  | 20,400  | 22 |
/// | 1,000,000 | 4,590,000  | 196,000  | 23 |
///
/// # Examples
///
/// ```rust
/// let m = vec![0xFF; 1000];
/// let x = vec![0xFF; 1000];
/// let y = vec![0; 1000];
/// assert_eq!(mhd_am::distance_fast(&m, &x, &y), Ok(8 * 1000));
///
/// // same alignment, but moderately complicated
/// assert_eq!(mhd_am::distance_fast(&m[1..1000 - 8], &x[1..1000 - 8], &y[8 + 1..]), Ok(8 * (1000 - 8 - 1)));
///
/// // differing alignments
/// assert!(mhd_am::distance_fast(&m[1..], &x[1..],   &y[..999]).is_err());
/// assert!(mhd_am::distance_fast(&m[1..], &x[..999], &y[..999]).is_err());
/// ```
pub fn distance_fast(mask: &[u8], x: &[u8], y: &[u8]) -> Result<u64, DistanceError> {
    assert_eq!(x.len(),    y.len());
    assert_eq!(mask.len(), y.len());

    const M1: u64 = 0x5555555555555555;
    const M2: u64 = 0x3333333333333333;
    const M4: u64 = 0x0F0F0F0F0F0F0F0F;
    const M8: u64 = 0x00FF00FF00FF00FF;

    type T30 = [u64; 30];

    // can't fit a single T30 in
    let (head0, thirty0, tail0) = unsafe {
        ::util::align_to::<_, T30>(mask)
    };
   let (head1, thirty1, tail1) = unsafe {
        ::util::align_to::<_, T30>(x)
    };
    let (head2, thirty2, tail2) = unsafe {
        ::util::align_to::<_, T30>(y)
    };

    if (head1.len() != head2.len()) || (head0.len() != head1.len()) {
        // The arrays required different shift amounts, so we can't
        // use aligned loads for both slices.
        return Err(DistanceError { _x: () });
    }

    debug_assert_eq!(thirty1.len(), thirty2.len());
    debug_assert_eq!(thirty0.len(), thirty1.len());

    // do the nonaligned stuff at the head and tail the hard way...
    let mut count = naive(head0, head1, head2) + naive(tail0, tail1, tail2);
    
    // now do the aligned stuff in the middle...
    for ( mask_array, 
         (array1, array2) ) in thirty0.iter()
                                      .zip( thirty1.iter()
                                                   .zip(thirty2) ) {
        let mut acc = 0;
        for j_ in 0..10 {
            let j = j_ * 3;
            // Next 3 lines were all we had to modify for masking!
            let mut count1 = mask_array[j]   & (array1[j]   ^ array2[j]  );
            let mut count2 = mask_array[j+1] & (array1[j+1] ^ array2[j+1]);
            let mut half1  = mask_array[j+1] & (array1[j+2] ^ array2[j+2]);
            let mut half2 = half1;
            half1 &= M1;
            half2 = (half2 >> 1) & M1;
            count1 -= (count1 >> 1) & M1;
            count2 -= (count2 >> 1) & M1;
            count1 += half1;
            count2 += half2;
            count1 = (count1 & M2) + ((count1 >> 2) & M2);
            count1 += (count2 & M2) + ((count2 >> 2) & M2);
            acc += (count1 & M4) + ((count1 >> 4) & M4);
        }
        acc = (acc & M8) + ((acc >> 8) & M8);
        acc =  acc       +  (acc >> 16);
        acc =  acc       +  (acc >> 32);
        count += acc & 0xFFFF;
    }
    Ok(count)
}

/// Computes the bitwise [Hamming
/// distance](https://en.wikipedia.org/wiki/Hamming_distance) between
/// `x` and `y`, that is, the number of bits where `x` and `y` differ,
/// or, the number of set bits in the xor of `x` and `y`
///  -- whereby only the bits which are set in the mask are counted!
///
/// When `mask`, `x` and `y` have the same 8-byte alignment, this uses
/// `distance_fast`, a highly optimised version of the following naive
/// version:
///
/// ```rust
/// fn naive(mask: &[u8], x: &[u8], y: &[u8]) -> u64 {
///  mask.iter().zip( x.iter().zip(y) )
///      .fold(0, |a, (m, (b, c)) | a + (*m & (*b ^ *c)).count_ones() as u64)
/// }
/// ```
///
/// If alignments differ, a slower but less restrictive algorithm is
/// used.
///
/// It is essentially guaranteed that `mask`, `x` and `y` will have the same
/// 8-byte alignment if they are all just `Vec<u8>`s of non-trivial
/// length (e.g. larger than 8) as in the example below.
///
/// # Panics
///
/// Arguments must have the same length, or else `distance` panics.
///
/// # Performance Comparison (old. obsolete...)
///
/// | length | `naive` (ns) | `distance` (ns) | `naive`/`distance` |
/// |--:|--:|--:|--:|
/// | 1 | 5  | 6  | 0.83 |
/// | 10 | 44  | 45  | 0.97 |
/// | 100 | 461  | 473  | 0.97 |
/// | 1,000 | 4,510  | 397  | 11 |
/// | 10,000 | 46,700  | 2,740  | 17 |
/// | 100,000 | 45,600  | 20,400  | 22 |
/// | 1,000,000 | 4,590,000  | 196,000  | 23 |
///
/// The benchmarks ensured that `x` and `y` had the same alignment.
///
/// # Examples
///
/// ```rust
/// let mask = vec![0xF0; 1000];
/// let x    = vec![0xFF; 1000];
/// let y    = vec![0; 1000];
/// assert_eq!(mhd_am::distance(&mask, &x, &y), 4 * 1000);
/// ```
pub fn distance(mask: &[u8], x: &[u8], y: &[u8]) -> u64 {
    distance_fast(mask, x, y)
        .ok()
        .unwrap_or_else(|| naive(mask, x, y))
}

#[cfg(test)]
mod tests {
    use quickcheck as qc;
    use rand;
    #[test]
    fn naive_smoke() {
        let tests: &[(&[u8], &[u8], &[u8], u64)] = &[
            (&[],       &[],        &[],        0),
            (&[0],      &[0],       &[0],       0),
            (&[0x0F],   &[0],       &[0xFF],    4),
            (&[0b11111111], &[0b10101010], &[0b01010101], 8),
            (&[0b11111111], &[0b11111010], &[0b11110101], 4),
            (&[0b00001111], &[0b11111010], &[0b11110101], 4),
            (&[0; 10],  &[0; 10],   &[0; 10],   0),
            (&[0xFF; 10], &[0xFF; 10], &[0x0F; 10], 4 * 10),
            (&[0x0F; 10], &[0xFF; 10], &[0x0F; 10], 0),
            (&[0xFF; 10000], &[0x3B; 10000], &[0x3B; 10000], 0),
            (&[0xFF; 10000], &[0x77; 10000], &[0x3B; 10000], 3 * 10000),
            (&[0x00; 10000], &[0x77; 10000], &[0x3B; 10000], 0),
            ];
        for &(mask, x, y, expected) in tests {
            assert_eq!(super::naive(mask, x, y), expected);
            assert_eq!(super::distance(mask, x, y), expected);
        }
    }
    #[test]
    fn distance_fast_qc() {
        fn prop(m_vec: Vec<u8>, x_vec: Vec<u8>, y_vec: Vec<u8>, misalign: u8) -> qc::TestResult {
            let l = ::std::cmp::min(m_vec.len(), ::std::cmp::min(x_vec.len(), y_vec.len()));
            if l < misalign as usize {
                return qc::TestResult::discard()
            }

            let mask = &m_vec[misalign as usize..l];
            let x    = &x_vec[misalign as usize..l];
            let y    = &y_vec[misalign as usize..l];
            qc::TestResult::from_bool(super::distance_fast(mask, x, y).unwrap() == super::naive(mask, x, y))
        }
        qc::QuickCheck::new()
            .gen(qc::StdGen::new(rand::thread_rng(), 300 ))  // was originally 10_000, 480 or 600 or 900 all take forever?!?
            .quickcheck(prop as fn(Vec<u8>,Vec<u8>,Vec<u8>,u8) -> qc::TestResult)
    }
    #[test]
    fn distance_fast_smoke_huge() {
        
        let m = vec![0b1111_1111; 10234567];
        let v = vec![0b1001_1101; 10234567];
        let w = vec![0b1111_1111; v.len()];

        assert_eq!(super::distance_fast(&m, &v, &v).unwrap(), 0);
        assert_eq!(super::distance_fast(&m, &v, &w).unwrap(), 3 * w.len() as u64);
    }
    #[test]
    fn distance_smoke() {
        let m = vec![0xFF; 10000];
        let v = vec![0; m.len()];
        let w = vec![0xFF; v.len()];
        for len_ in 0..99 {
            let len = len_ * 10;
            for i in 0..8 {
                for j in 0..8 {
                    assert_eq!(super::distance(&m[i..i+len], &v[i..i+len], &w[j..j+len]),
                               len as u64 * 8)
                }
            }
        }
    }
}
