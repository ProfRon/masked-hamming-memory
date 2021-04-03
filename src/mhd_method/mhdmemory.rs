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
/// let mut test_mem = MhdMemory::new( NUM_BITS);
/// assert!( test_mem.is_empty() );
/// assert_eq!( test_mem.width(), NUM_BITS );
///
/// let row0 = Sample::new(NUM_BITS,   3 as ScoreType );
/// let row1 = Sample::new(NUM_BITS,  33 as ScoreType );
/// let row2 = Sample::new(NUM_BITS, 333 as ScoreType );
///
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
pub struct MhdMemory {
    pub width : usize,
    pub total_score: ScoreType,
    pub max_score: ScoreType,
    pub min_score: ScoreType,
    pub samples: Vec<Sample>, // initially empty
} // end struct Sample

use log::*;

impl MhdMemory {
    #[inline]
    pub fn default() -> Self {
        Self {
            width : 0,
            total_score: ZERO_SCORE,
            max_score: ZERO_SCORE,
            min_score: ZERO_SCORE,
            samples: vec![], // start with an empty vector of samples
        }
    }

    #[inline]
    pub fn new( width : usize ) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    #[inline]
    pub fn width(&self) -> usize { self.width }

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
    pub fn clear(&mut self) {
        let old_width = self.width;
        self.samples.clear();
        *self = Self::new( old_width );
    }

    #[inline]
    pub fn write_sample(&mut self, new_sample: &Sample ) {
        // Here the book-keeping...
        assert_eq!( self.width, new_sample.size() );
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
        assert!( self.width <= 8 * mask.len() );
        assert!( self.width <= 8 * query.len() );
        let (score_sum, weight_sum) = self
            .samples
            .iter()
            .map(|s| {
                // use a closure here to capture query and mask
                let dist = distance(mask, query, &s.bytes);
                let dist_plus_1 = (dist + 1) as f64; // adding one prevents division by zero later
                                                     // let weight = 1.0 / (dist_plus_1 * dist_plus_1);
                let weight = 1.0 / dist_plus_1; // TODO DECIDE! Squared or not!!!
                let floating_avg = self.avg_score() as f64;
                let delta_score = s.score as f64 - floating_avg;
                let weighted_delta = delta_score * weight;
                let weighted_score = floating_avg + weighted_delta;
                (weighted_score, weight) // return score
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

    pub fn read_for_decision(
        &self,
        mask: &[u8],
        query: &[u8],
        index: usize,
    ) -> (Option<ScoreType>, Option<ScoreType>) {
        assert!( self.width <= 8 * mask.len() );
        assert!( self.width <= 8 * query.len() );
        let mut hit_on_0 = false; // until proven true
        let mut hit_on_1 = false; // until proven true
        let (score_false, score_true, weight_sum) = self
            .samples
            .iter()
            .map(|s| {
                // use a closure here to capture query and mask
                let dist = distance(mask, query, &s.bytes);
                let dist_plus_1 = (dist + 1) as f64; // adding one prevents division by zero later
                                                     // let weight = 1.0 / (dist_plus_1 * dist_plus_1);
                let weight = 1.0 / dist_plus_1; // TODO DECIDE! Squared or not!!!
                let s_at_index = s.get_bit(index);
                if s_at_index {
                    hit_on_1 = true;
                    (0.0f64, weight * s.score as f64, weight) // return score
                } else {
                    // if NOT s_at_index
                    hit_on_0 = true;
                    (weight * s.score as f64, 0.0f64, weight) // return score
                }
            })
            .fold((0.0, 0.0, 0.0), |(s0f, s0t, w0), (s1f, s1t, w1)| {
                (s0f + s1f, s0t + s1t, w0 + w1)
            });

        let false_score = if hit_on_0 {
            Some((score_false / weight_sum) as ScoreType)
        } else {
            None
        };
        let true_score = if hit_on_1 {
            Some((score_true / weight_sum) as ScoreType)
        } else {
            None
        };
        let result = (false_score, true_score);
        trace!(
            "hits = ({},{}), scores = ({}, {}), weight sum =  {}, result = {:?}",
            hit_on_0,
            hit_on_1,
            score_false,
            score_true,
            weight_sum,
            result
        );
        // Return...
        result
    } // end maked_read

    #[inline]
    pub fn write_random_sample(&mut self) {
        self.write_sample(&Sample::random( self.width ));
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
        let mut new_test_mem = MhdMemory::new( NUM_BITS );

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

        let mut new_test_mem = MhdMemory::new( NUM_BITS );

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

        let zero_mask = &Sample::new(NUM_BITS, ZERO_SCORE );
        let ones_mask = &Sample::new_ones( NUM_BITS, ZERO_SCORE );

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


    #[test]
    fn test_read_for_decision() {
        const NUM_BITS: usize = 64;
        const NUM_ROWS: usize = 64; // Must be at least four!!!
        const LOG_NUM_ROWS: usize = 6;

        assert!(4 < NUM_ROWS);

        let mut new_test_mem = MhdMemory::new( NUM_BITS );

        new_test_mem.write_n_random_samples(NUM_ROWS);

        assert!(new_test_mem.min_score <= new_test_mem.avg_score());
        assert!(new_test_mem.avg_score() <= new_test_mem.max_score);
        assert_ne!(new_test_mem.min_score, new_test_mem.max_score);

        trace!(
            "Memory has scores min {} < avg {} < max {} < total {}",
            new_test_mem.min_score,
            new_test_mem.avg_score(),
            new_test_mem.max_score,
            new_test_mem.total_score,
        );

        // Now, test reading!!!
        let mut false_misses = 0;
        let mut true_misses = 0;
        let mut index = 0;
        let mut sums = ( ZERO_SCORE, ZERO_SCORE );
        for row in 0..NUM_ROWS {
            let random_mask = &Sample::random( NUM_BITS );
            index = (index +1) % NUM_BITS;
            let ( false_result, true_result) = new_test_mem.read_for_decision(
                &random_mask.bytes, &new_test_mem.samples[row].bytes, index );
            match false_result {
                None => false_misses += 1,
                Some( score ) => sums = ( sums.0 + score, sums.1 ),
            };
            match true_result {
                None => true_misses += 1,
                Some( score ) => sums = ( sums.0, sums.1 + score ),
            };
            trace!(
                "Row {} has decision result ( {:?}, {:?} )",
                row,
                false_result, true_result
            );
        } // end for all rows
        trace!( "At end of read_for_decision test, num misses on false = {}, on true = {}, sums = {:?}",
            false_misses, true_misses, sums );
        assert!(false_misses <= LOG_NUM_ROWS); // capricious and arbitrary, but gotta be sumthin...
        assert!(true_misses <= LOG_NUM_ROWS); // capricious and arbitrary, but gotta be sumthin...
    } // end test read_for_decsions

} // end mod tests
