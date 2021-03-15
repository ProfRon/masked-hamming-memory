/// # The MHD Memory Struct
/// Formally, the memory consists of a collection of `samples`, and various `read` and `write` operations.
///
/// `Samples` were defined in the `sample.rs` file. They are critical here.
///
/// Now we define a container to hold samples.
///
/// ```rust
///
/// use mhd_mem::mhd_method::{MHDMemory, Sample, ScoreType, NUM_BYTES };
/// let mut test_mem = MHDMemory::default();
/// assert!( test_mem.is_empty() );
///
/// let row0 = Sample { bytes : vec![0xFF;  NUM_BYTES ], score :   3 as ScoreType };
/// let row1 = Sample { bytes : vec![0xFF;  NUM_BYTES ], score :  33 as ScoreType };
/// let row2 = Sample { bytes : vec![0xF0;  NUM_BYTES ], score : 333 as ScoreType };
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
pub struct MHDMemory {
    pub samples: Vec<Sample>, // initially empty

    pub total_score: ScoreType,
    pub max_score: ScoreType,
    pub min_score: ScoreType,
} // end struct Sample

impl MHDMemory {
    pub fn default() -> Self {
        MHDMemory {
            samples: vec![], // start with an empty vector of samples
            total_score: ZERO_SCORE,
            max_score: ZERO_SCORE,
            min_score: ZERO_SCORE,
        }
    }

    pub fn new() -> Self {
        MHDMemory {
            ..Default::default()
        }
    }

    pub fn num_samples(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn avg_score(&self) -> ScoreType {
        if self.is_empty() {
            ZERO_SCORE
        } else {
            // if not empty
            self.total_score / self.num_samples() as ScoreType
        }
    }

    pub fn write_sample(&mut self, new_sample: &Sample) {
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

    pub fn masked_read(&self, mask: &[u8], query: &[u8]) -> ScoreType {
        // read only idempotent method

        let (score_sum, weight_sum) = self
            .samples
            .iter()
            .map(|s| {
                // use a closure here to capture query and mask
                let dist = distance(mask, query, &s.bytes);
                let dist_plus_1 = (dist + 1) as f64; // adding one prevents division by zero later
                let weight = 1.0 / (dist_plus_1 * dist_plus_1);
                (weight * s.score as f64, weight) // return score
            })
            .fold((0.0, 0.0), |(s0, w0), (s1, w1)| (s0 + s1, w0 + w1));

        let result = score_sum / weight_sum;
        println!(
            "sum of scores = {}, sum of weights =  {}, result = {}",
            score_sum, weight_sum, result
        );
        result as ScoreType
    } // end maked_read

    pub fn write_random_sample(&mut self) {
        self.write_sample(&Sample::random());
    } // end write_sample

    pub fn write_n_random_samples(&mut self, n: usize) {
        for _ in 0..n {
            self.write_random_sample();
        }
    }
} // more coming up below

// TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS TESTS

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_random_write() {
        let mut new_test_mem = MHDMemory::new();

        new_test_mem.write_random_sample();

        assert!(!new_test_mem.is_empty());
        assert_eq!(1, new_test_mem.num_samples());
        assert_ne!(ZERO_SCORE, new_test_mem.samples[0].score);
    }

    #[test]
    fn test_random_writes() {
        let mut new_test_mem = MHDMemory::new();

        new_test_mem.write_random_sample();
        new_test_mem.write_n_random_samples(2);
        new_test_mem.write_random_sample();

        assert!(!new_test_mem.is_empty());
        assert_eq!(4, new_test_mem.num_samples());
        assert_ne!(new_test_mem.samples[0], new_test_mem.samples[1]);
        assert_ne!(new_test_mem.samples[1], new_test_mem.samples[2]);
        assert_ne!(new_test_mem.samples[2], new_test_mem.samples[3]);
    }
}
