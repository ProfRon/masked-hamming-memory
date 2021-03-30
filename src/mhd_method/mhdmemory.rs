/// # The MHD Memory Struct
/// Formally, the memory consists of a collection of `samples`, and various `read` and `write` operations.
///
/// `Samples` were defined in the `sample.rs` file. They are critical here.
///
/// Now we define a container to hold samples.
///
/// ```rust
///
/// use mhd_mem::mhd_method::{MhdMemory, Sample, ScoreType };
/// const NUM_BITS : usize = 356; // arbitrary, .... 44.5 bytes
/// let mut test_mem = MhdMemory::<NUM_BITS>::default();
/// assert!( test_mem.is_empty() );
///
/// let row0 = Sample::<NUM_BITS>::new(   3 as ScoreType );
/// let row1 = Sample::<NUM_BITS>::new(  33 as ScoreType );
/// let row2 = Sample::<NUM_BITS>::new( 333 as ScoreType );
///
/// assert_eq!( test_mem.width(), NUM_BITS );
/// assert_eq!( row0.size(), NUM_BITS );
///
/// test_mem.write_sample( &row2 );
/// test_mem.write_sample( &row1 );
/// test_mem.write_sample( &row0 );
///
/// assert!( ! test_mem.is_empty() );
/// assert_eq!( 3, test_mem.num_samples() );
///
/// let target_total : ScoreType = 3 + 33 + 333; // == 369 right?
/// assert_eq!( test_mem.total_score, target_total );
/// assert_eq!( test_mem.min_score, 3 );
/// assert_eq!( test_mem.max_score, 333 );
/// let target_avg : ScoreType = target_total / (3 as ScoreType); // == 123 ?
/// assert_eq!( test_mem.avg_score(), target_avg );
/// ```
use mhd_method::distance_::*;
use mhd_method::sample::*;
// use ::mhd_method::util::*;    // Not needed, according to compiler
// use ::mhd_method::weight_::*; // Not needed, according to compiler

#[derive(Debug, Default, Clone)]
pub struct MhdMemory<const NUM_BITS: usize> {
    pub samples: Vec<Sample<NUM_BITS>>, // initially empty

    pub total_score: ScoreType,
    pub max_score: ScoreType,
    pub min_score: ScoreType,
} // end struct Sample

use log::*;

impl<const NUM_BITS: usize> MhdMemory<NUM_BITS> {
    #[inline]
    pub fn default() -> Self {
        Self {
            samples: vec![], // start with an empty vector of samples
            total_score: ZERO_SCORE,
            max_score: ZERO_SCORE,
            min_score: ZERO_SCORE,
        }
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        NUM_BITS
    }

    #[inline]
    pub fn num_samples(&self) -> usize {
        self.samples.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    #[inline]
    pub fn avg_score(&self) -> ScoreType {
        if self.is_empty() {
            ZERO_SCORE
        } else {
            // if not empty
            self.total_score / self.num_samples() as ScoreType
        }
    }

    #[inline]
    pub fn write_sample(&mut self, new_sample: &Sample<NUM_BITS>) {
        // Here the book-keeping...
        self.total_score += new_sample.score;
        if self.is_empty() {
            self.max_score = new_sample.score;
            self.min_score = new_sample.score;
        } else {
            // if not empty
            // I wanted to use ::std::cmp::max and min here, but...
            // the trait `Ord` is not implemented for `f32`   ?!?
            if self.max_score < new_sample.score {
                self.max_score = new_sample.score
            };
            if new_sample.score < self.min_score {
                self.min_score = new_sample.score
            };
        }

        // Here the real work...
        self.samples.push(new_sample.clone());
    } // end write_sample

    /// Calculate the weighted sum of all the samples in the memory,
    /// where the weight of each sample is the inverse of the squared masked hamming distance to
    /// the query, i.e. 1 / (mhd * mhd)
    /// **This is not a maximum function (yet).**
    pub fn masked_read(&self, mask: &[u8], query: &[u8]) -> ScoreType {
        let (score_sum, weight_sum) = self
            .samples
            .iter()
            .map(|s| {
                // use a closure here to capture query and mask
                let dist = distance(mask, query, &s.bytes);
                let dist_plus_1 = (dist + 1) as f64; // adding one prevents division by zero later
                                                     // let weight = 1.0 / (dist_plus_1 * dist_plus_1);
                let weight = 1.0 / dist_plus_1; // TODO DECIDE! Squared or not!!!
                (weight * s.score as f64, weight) // return score
            })
            .fold((0.0, 0.0), |(s0, w0), (s1, w1)| (s0 + s1, w0 + w1));

        let result = score_sum / weight_sum;
        trace!(
            "sum of scores = {}, sum of weights =  {}, result = {}",
            score_sum,
            weight_sum,
            result
        );
        result as ScoreType
    } // end maked_read

    #[inline]
    pub fn write_random_sample(&mut self) {
        self.write_sample(&Sample::<NUM_BITS>::random());
    } // end write_sample

    #[inline]
    pub fn write_n_random_samples(&mut self, n: usize) {
        for _ in 0..n {
            self.write_random_sample();
        }
    }
} // more coming up below

///////////////////////// TESTS TESTS TESTS TESTS TESTS TESTS /////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_random_write() {
        const NUM_BITS: usize = 256;
        let mut new_test_mem = MhdMemory::<NUM_BITS>::new();

        assert!(new_test_mem.is_empty());
        assert_eq!(0, new_test_mem.num_samples());
        assert_eq!(new_test_mem.width(), NUM_BITS);
        assert_eq!(ZERO_SCORE, new_test_mem.avg_score());

        new_test_mem.write_random_sample();

        assert!(!new_test_mem.is_empty());
        assert_eq!(1, new_test_mem.num_samples());
        assert_ne!(ZERO_SCORE, new_test_mem.samples[0].score);
        assert_eq!(new_test_mem.samples[0].size(), NUM_BITS);

        assert_eq!(new_test_mem.min_score, new_test_mem.max_score);
        assert_eq!(new_test_mem.min_score, new_test_mem.total_score);
    }

    #[test]
    fn test_random_writes() {
        const NUM_BITS: usize = 64;
        const NUM_ROWS: usize = 64; // Must be at least four!!!
        const LOG_NUM_ROWS: usize = 6;

        assert!(4 < NUM_ROWS);

        let mut new_test_mem = MhdMemory::<NUM_BITS>::new();

        assert!(new_test_mem.is_empty());
        assert_eq!(new_test_mem.width(), NUM_BITS);

        new_test_mem.write_n_random_samples(NUM_ROWS);

        assert!(!new_test_mem.is_empty());
        assert_eq!(NUM_ROWS, new_test_mem.num_samples());
        assert_eq!(new_test_mem.samples[0].size(), NUM_BITS);

        assert_ne!(new_test_mem.samples[0], new_test_mem.samples[1]);
        assert_ne!(new_test_mem.samples[1], new_test_mem.samples[2]);
        assert_ne!(new_test_mem.samples[2], new_test_mem.samples[3]);
        // ... and so on ... don't test all, too likely to find a false positive (?)
        assert_ne!(
            new_test_mem.samples[NUM_ROWS - 1],
            new_test_mem.samples[NUM_ROWS - 2]
        );

        assert!(new_test_mem.min_score <= new_test_mem.avg_score());
        assert!(new_test_mem.avg_score() <= new_test_mem.max_score);
        assert_ne!(new_test_mem.min_score, new_test_mem.max_score);

        let avg_score = new_test_mem.avg_score();
        trace!(
            "Memory has scores min {} < avg {} < max {} < total {}",
            new_test_mem.min_score,
            new_test_mem.avg_score(),
            new_test_mem.max_score,
            new_test_mem.total_score,
        );

        // Now, test reading!!!

        let zero_mask = &Sample::<NUM_BITS>::new(0);
        let ones_mask = &Sample::<NUM_BITS>::new_ones(0);

        let mut lucky_hits: usize = 0;
        for row in 0..NUM_ROWS {
            let zero_mask_score: ScoreType =
                new_test_mem.masked_read(&zero_mask.bytes, &new_test_mem.samples[row].bytes);
            let ones_mask_score: ScoreType =
                new_test_mem.masked_read(&ones_mask.bytes, &new_test_mem.samples[row].bytes);
            let score_row = new_test_mem.samples[row].score;
            // Zero mask means everything is masked out, so distance is always zero, so we read the avg!
            // Ones mask means everyting is maked in, so distance is often greater than zero,
            // so we expect... a score a little closer to the average.
            trace!(
                "Row {} has score {}; Read with mask 1s -> {}, 0s -> {}",
                row,
                score_row,
                ones_mask_score,
                zero_mask_score,
            );
            if zero_mask_score == ones_mask_score {
                lucky_hits += 1; // improbable but possible, we'll allow 1 or 2 or... see below.
            };
            assert_eq!(zero_mask_score, avg_score);
            if ones_mask_score != avg_score {
                // equality improbable and breaks next lines
                if avg_score < score_row {
                    // assert!(ones_mask_score <= score_row);
                    if score_row < ones_mask_score {
                        warn!(
                            "Warning this should be: avg {} <= 1s read {} <= row {}",
                            avg_score, ones_mask_score, score_row
                        );
                    };
                } else {
                    // if score_row0 < avg_score
                    // assert!(score_row <= ones_mask_score)
                    if ones_mask_score < score_row {
                        warn!(
                            "Warning this should be: row {} <= 1s read {} <= avg {}",
                            score_row, ones_mask_score, avg_score
                        );
                    };
                }; // end if score_row < avg_score
            }; // end if ones hit returns avg exactly
        } // end for all rows
        assert!(lucky_hits <= LOG_NUM_ROWS); // capricious and arbitrary, but gotta be sumthin...
    } // end test random writes
} // end mod tests
