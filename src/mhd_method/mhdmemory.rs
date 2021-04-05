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
/// let mut row0 = Sample::random(NUM_BITS );
/// row0.score =  3 as ScoreType;
/// let mut row1 = Sample::random(NUM_BITS );
/// row1.score =  33 as ScoreType;
/// let mut row2 = Sample::random(NUM_BITS );
/// row2.score = 333 as ScoreType;
///
/// assert_eq!( row0.size(), NUM_BITS );
///
/// test_mem.write_sample( &row2 );
/// test_mem.write_sample( &row1 );
/// test_mem.write_sample( &row0 );
/// test_mem.write_sample( &row2 ); // already there, so does nothing! Not added!
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
use rand::Rng;
use mhd_method::distance_::*;
use mhd_method::sample::*;
// use ::mhd_method::util::*;    // Not needed, according to compiler
// use ::mhd_method::weight_::*; // Not needed, according to compiler

#[derive(Debug, Default, Clone)]
pub struct MhdMemory {
    pub width: usize,
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
            width: 0,
            total_score: ZERO_SCORE,
            max_score: ZERO_SCORE,
            min_score: ZERO_SCORE,
            samples: vec![], // start with an empty vector of samples
        }
    }

    #[inline]
    pub fn new(width: usize) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width
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
    pub fn clear(&mut self) {
        let old_width = self.width;
        self.samples.clear();
        *self = Self::new(old_width);
    }

    // search for a sample with a patter -- return true iff the query is already stored
    #[inline]
    pub fn search(&self, query: &Sample) -> Option<&Sample> {
        self.samples
            .iter()
            .find(|s_in_mem| s_in_mem.bytes == query.bytes)
    } // end sample_present

    #[inline]
    pub fn write_sample(&mut self, new_sample: &Sample) {
        assert_eq!(self.width, new_sample.size());

        // First take care of the scores
        if self.is_empty() {
            self.total_score += new_sample.score;
            self.max_score = new_sample.score;
            self.min_score = new_sample.score;
            self.samples.push(new_sample.clone());
        } else {
            match self.search(&new_sample) {
                Some(elder_sample) => {
                    // Check that the scores match TOO, which they must...
                    assert_eq!(elder_sample.score, new_sample.score);
                    // But otherwise do nothing!
                }
                None => {
                    // if not empty, and query not found in memory:
                    // I wanted to use ::std::cmp::max and min here, but...
                    // the trait `Ord` is not implemented for `f32`
                    if self.max_score < new_sample.score {
                        self.max_score = new_sample.score
                    };
                    if new_sample.score < self.min_score {
                        self.min_score = new_sample.score
                    };
                    self.total_score += new_sample.score;
                    self.samples.push(new_sample.clone());
                }
            }
        };

        // Then take care of the bytes and actually adding the new sample to the memory
    } // end write_sample

    /// Calculate the weighted sum of all the samples in the memory,
    /// where the weight of each sample is the inverse of the squared masked hamming distance to
    /// the query, i.e. 1 / (mhd * mhd)
    /// **This is not a maximum function (yet).**
    pub fn masked_read(&self, mask: &[u8], query: &[u8]) -> ScoreType {
        assert!(self.width <= 8 * mask.len());
        assert!(self.width <= 8 * query.len());
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

    pub fn read_and_decide(&self, mask: &[u8], query: &[u8], index: usize) -> bool {
        assert!(self.width <= 8 * mask.len());
        assert!(self.width <= 8 * query.len());
        // let threshold = std::cmp::max( 8,std::cmp::min( 4, mask.iter().count_ones() ) );
        const THRESHOLD: u64 = 4; // TODO : Optimize threshold!
        const UCB_CONSTANT : f64 = 5.65685425; // == 4* 2.sqrt()
        let mut hits_on_0: usize = 0;
        let mut hits_on_1: usize = 0;
        let (score_false, score_true, weight_false, weight_true) = self
            .samples
            .iter()
            .map(|s| {
                // use a closure here to capture query and mask
                let dist = distance(mask, query, &s.bytes);
                if THRESHOLD < dist {
                    (0.0f64, 0.0f64, 0.0f64, 0.0f64)
                } else { // if dist <= THRESHOLD
                    let dist_plus_1 = (dist + 1) as f64; // adding one prevents division by zero later
                    // let weight = 1.0 / (dist_plus_1 * dist_plus_1);
                    let weight = 1.0 / dist_plus_1; // TODO DECIDE! Squared or not!!!
                    let s_at_index = s.get_bit(index);
                    if s_at_index {
                        if 0 == dist { hits_on_1 += 1 };
                        (0.0f64, weight * s.score as f64, 0.0f64, weight) // return score
                    } else {
                    // if dist <= threshold AND NOT s_at_index
                        if 0 == dist { hits_on_0 += 1 };
                        (weight * s.score as f64, 0.0f64, weight, 0.0f64) // return score
                    }
                } // endif dist <= THRESHOLD
            })
            .fold(
                (0.0, 0.0, 0.0, 0.0),
                |(s0f, s0t, w0f, w0t), (s1f, s1t, w1f, w1t)| {
                    (s0f + s1f, s0t + s1t, w0f + w1f, w0t + w1t)
                },
            );

        // We now know if there were any hits on 0, or on 1, and if so, with what scores
        let result = if 0 == hits_on_0 {
            if 0 == hits_on_1 {
                // if 0 == hit_on_0 == hit_on_1... flip a coin!
                rand::thread_rng().gen::<bool>()
            } else {
                // if 0 == hits_on_0 BUT 0 < hits_on_1, return...
                false
            }
        } else if 0 == hits_on_1 {
            // if NOT hits_on_1 BUT hits_on_0, return...
            true
        } else {
            // if 0 < hits_on_1 AND 0 < hits_on_0
            // Exploitation: true_score / best_score - false_score / best_score = true- false /best
            let denominator = self.max_score as f64;
            let true_exploitation = (score_true / weight_true) / denominator;
            let false_exploitation = (score_false / weight_false) / denominator;

            // exploration -- trickier...
            let total_hits : f64 = (hits_on_0 + hits_on_1) as f64;
            let ln_total_hits = total_hits.ln();
            let true_exploration =  (ln_total_hits / hits_on_1 as f64).sqrt() * UCB_CONSTANT;
            let false_exploration = (ln_total_hits / hits_on_0 as f64).sqrt() * UCB_CONSTANT;

            // UCB Formula, kinda...
            let true_sum = true_exploitation + true_exploration;
            if true_sum <= 0.0 {
                error!("True Sum = {} = exploration = {} + exploitation {} <= 0.0",
                        true_sum, true_exploration, true_exploitation );
            };
            // assert!(0.0 < true_sum);
            // assert!(true_sum <= UCB_CONSTANT+1.0);
            let false_sum = false_exploitation + false_exploration;
            if false_sum <= 0.0 {
                error!("False Sum = {} = exploration = {} + exploitation {} <= 0.0",
                       true_sum, false_exploration, false_exploitation );
            };
            // assert!(0.0 < false_sum);
            // assert!(false_sum <= UCB_CONSTANT+1.0);

            // DECIDE!
            // Are deterministic decisions a bad idea because they repeat?!?
/*******    if false_sum < true_sum {
                true
            } else if true_sum < false_sum {
                false
            } else {
                // true_sum == false_sum  --- flip a coin!
                rand::thread_rng().gen()
            }
********/
            // Or are probablistic decisions even worse? Because ... flaky?
            let probability = true_sum / (true_sum + false_sum);
            rand::thread_rng().gen_bool( probability )
        };

        trace!(
            "MHD MEM: hits = ({},{}), scores = ({}, {}), weights =  ({}, {}), result = {}",
            hits_on_0,
            hits_on_1,
            score_false,
            score_true,
            weight_false,
            weight_true,
            result
        );

        // Return...
        result
    } // end maked_read

    #[inline]
    pub fn write_random_sample(&mut self) {
        self.write_sample(&Sample::random(self.width));
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
    // use rand::prelude::*;

    #[test]
    fn test_one_random_write() {
        const NUM_BITS: usize = 256;
        let mut memory = MhdMemory::new(NUM_BITS);

        assert!(memory.is_empty());
        assert_eq!(0, memory.num_samples());
        assert_eq!(memory.width(), NUM_BITS);
        assert_eq!(ZERO_SCORE, memory.avg_score());

        memory.write_random_sample();

        assert!(!memory.is_empty());
        assert_eq!(1, memory.num_samples());
        assert_ne!(ZERO_SCORE, memory.samples[0].score);
        assert_eq!(memory.samples[0].size(), NUM_BITS);

        assert_eq!(memory.min_score, memory.max_score);
        assert_eq!(memory.min_score, memory.total_score);
    }

    #[test]
    fn test_random_writes() {
        const NUM_BITS: usize = 64;
        const NUM_ROWS: usize = 64; // Must be at least four!!!
        const LOG_NUM_ROWS: usize = 6;

        assert!(4 < NUM_ROWS);

        let mut memory = MhdMemory::new(NUM_BITS);

        assert!(memory.is_empty());
        assert_eq!(memory.width(), NUM_BITS);

        memory.write_n_random_samples(NUM_ROWS);

        assert!(!memory.is_empty());
        assert_eq!(NUM_ROWS, memory.num_samples());
        assert_eq!(memory.samples[0].size(), NUM_BITS);

        assert_ne!(memory.samples[0], memory.samples[1]);
        assert_ne!(memory.samples[1], memory.samples[2]);
        assert_ne!(memory.samples[2], memory.samples[3]);
        // ... and so on ... don't test all, too likely to find a false positive (?)
        assert_ne!(memory.samples[NUM_ROWS - 1], memory.samples[NUM_ROWS - 2]);

        assert!(memory.min_score <= memory.avg_score());
        assert!(memory.avg_score() <= memory.max_score);
        assert_ne!(memory.min_score, memory.max_score);

        let avg_score = memory.avg_score();
        trace!(
            "Memory has scores min {} < avg {} < max {} < total {}",
            memory.min_score,
            memory.avg_score(),
            memory.max_score,
            memory.total_score,
        );

        // Now, test reading!!!

        let zero_mask = &Sample::new(NUM_BITS, ZERO_SCORE);
        let ones_mask = &Sample::new_ones(NUM_BITS, ZERO_SCORE);

        let mut lucky_hits: usize = 0;
        for row in 0..NUM_ROWS {
            let zero_mask_score: ScoreType =
                memory.masked_read(&zero_mask.bytes, &memory.samples[row].bytes);
            let ones_mask_score: ScoreType =
                memory.masked_read(&ones_mask.bytes, &memory.samples[row].bytes);
            let score_row = memory.samples[row].score;
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
    fn test_identical_writes() {
        const NUM_BITS: usize = 64;
        const NUM_ROWS: usize = 64; // Must be at least four!!!

        assert!(8 <= NUM_ROWS);

        let mut memory = MhdMemory::new(NUM_BITS);

        memory.write_n_random_samples(NUM_ROWS);

        assert!(!memory.is_empty());
        assert_eq!(NUM_ROWS, memory.num_samples());

        let redundant_a = memory.samples[4].clone();
        let redundant_b = memory.samples[6].clone();

        memory.write_sample(&redundant_a);
        assert_eq!(NUM_ROWS, memory.num_samples());

        memory.write_sample(&redundant_b);
        assert_eq!(NUM_ROWS, memory.num_samples());

        memory.write_n_random_samples(NUM_ROWS);
        assert_eq!(2 * NUM_ROWS, memory.num_samples());
    }

    #[test]
    fn test_read_for_decision() {
        const NUM_BITS: usize = 16;
        const NUM_ROWS: usize = 32; // Must be at least four!!!

        assert!(4 < NUM_ROWS);

        let mut memory = MhdMemory::new(NUM_BITS);

        memory.write_n_random_samples(NUM_ROWS);

        assert!(memory.min_score <= memory.avg_score());
        assert!(memory.avg_score() <= memory.max_score);
        assert_ne!(memory.min_score, memory.max_score);

        trace!(
            "Memory has scores min {} < avg {} < max {} < total {}",
            memory.min_score,
            memory.avg_score(),
            memory.max_score,
            memory.total_score,
        );

        // Now, test reading!!!
        let mut true_decisions = 0;
        let mut false_decisions = 0;
        let mut index: usize = 0;
        for row in 0..NUM_ROWS {
            let random_mask = &Sample::random(NUM_BITS);
            index = (index + 1) % NUM_BITS;
            let decision =
                memory.read_and_decide(&random_mask.bytes, &memory.samples[row].bytes, index);
            if decision {
                true_decisions += 1
            } else {
                false_decisions += 1
            };
            trace!(
                "Row {} has decision result {} - counts = (f {}, t {})",
                row,
                decision,
                false_decisions,
                true_decisions
            );
        } // end for all rows
        trace!(
            "At end of read_for_decision test, falses = {}, trues = {}",
            false_decisions,
            true_decisions
        );
        // remainder is capricious and arbitrary, but we gotta do sumthin...
        assert!(0 < true_decisions);
        assert!(0 < false_decisions);
        assert!(true_decisions < NUM_ROWS);
        assert!(false_decisions < NUM_ROWS);
    } // end test read_for_decsions
} // end mod tests
