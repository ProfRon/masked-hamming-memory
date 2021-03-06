/// # Example Implementations
///
/// ## Example -- the Subset Sum 0-1 Knapsack
///
/// This is a version of the knapsack problem without values:
/// Every item which can go into the knapsack has a weight. Period.
/// The goal is to get the weight of the (filled) knapsack as close to its capacity
/// as possible.
/// This is a subset of the "zero/one" knapsack problem, since we do not consider the
/// possibility of putting two or three or _n_ instances of an item in the sack --
/// every item is either in the sack or out of it.
///
/// Note: This problem does not need its own, customized associated solution type;
/// it ("just") uses the MinimalSolution struct from the mhd_optimization module.
use log::*;

extern crate rand_distr;

use rand_distr::{Bernoulli, Distribution, Gamma}; // formerly used: Exp

use mhd_memory::{ScoreType, ZERO_SCORE}; // Not used: NUM_BYTES
use optimizer::{MinimalSolution, Problem, Solution};

#[derive(Debug, Clone)]
pub struct ProblemSubsetSum {
    pub weights: Vec<ScoreType>,
    pub capacity: ScoreType, // The capacity of the Knapsack (not of the weights vector)
} // end struct Sample

// Utility Methods (not part of the Problem trait)
impl ProblemSubsetSum {
    pub fn weights_sum(&self) -> ScoreType {
        self.weights.iter().sum()
    }
}

// Problem Trait Methods
impl Problem for ProblemSubsetSum {
    type Sol = MinimalSolution; // !!!!

    #[inline]
    fn name(&self) -> &'static str {
        "ProblemSubsetSum"
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{}: capacity {} <= weight sum {}",
            self.name(),
            self.capacity,
            self.weights_sum()
        )
    }

    #[inline]
    fn new(size: usize) -> Self {
        ProblemSubsetSum {
            weights: vec![ZERO_SCORE; size],
            capacity: 0,
        }
    }

    // fn random( size : usize ) -> Self -- take the default implementation

    #[inline]
    fn problem_size(&self) -> usize {
        self.weights.len()
    }

    fn randomize(&mut self) {
        let num_bits = self.problem_size();
        debug_assert!(
            2 < num_bits,
            "Randomize not defined when problem_size = {}",
            num_bits
        );
        // self.weights =  (0..self.problem_size()).map( |_| fancy_random_int( ) ).collect();
        let mut rng = rand::thread_rng();
        // The parameters shape=2.0 and scale=1000.0 were arrived at by playing around in a
        // Jupyter Notebook but remain failry arbitrary
        let distr = Gamma::new(2.0, 1000.0).unwrap();

        self.weights = (0..num_bits)
            .map(|_| (distr.sample(&mut rng) + 1.0) as ScoreType)
            .collect();

        ///// The next two lines are optional. Experimentation still going on to see if they help.
        ////  They are not independant: The 2nd makes no sense without the first, so either none,
        ////  just the first or both. See below for experimental results.
        // Sort weights
        self.weights.sort_unstable();
        self.weights.reverse();
        debug_assert!(
            num_bits == self.problem_size(),
            "Problem size changed in sort?!?"
        );
        debug_assert!(0 < self.weights[0]);
        debug_assert!(0 < self.weights[num_bits - 1]);
        debug_assert!(self.weights[num_bits - 1] <= self.weights[0]); // Change if not reversing sort

        // Choose Capacity as the sum of a random selection of the weights
        let berno_distr = Bernoulli::new(0.5).unwrap();
        loop {
            self.capacity = self
                .weights
                .iter()
                .map(|w| {
                    if berno_distr.sample(&mut rng) {
                        *w
                    } else {
                        ZERO_SCORE
                    }
                })
                .sum();
            if self.is_legal() {
                return;
            };
            // else, find another capacity
        } // loop until self.is_legal();
    }

    #[inline]
    fn is_legal(&self) -> bool {
        // Note: We're NOT testing whether a solution is legal (we do that below),
        // We're testing if a PROBLEM is non-trivial: if neither the empty knapsack
        // nor the knapsack with ALL items are obviously optimal solutions.
        // Note: By definition, the default knapsack is ILLEGAL since all weights are zero, etc.
        //
        // Revision: We're going to allow overly large capacity after all...
        let legal = (0 < self.problem_size()) && (0 < self.capacity);
        if !legal || (self.weights_sum() <= self.capacity) {
            warn!(
                "Funky Subset Sum Proble: dim {}, weight sum {} <= capacity {}",
                self.problem_size(),
                self.weights_sum(),
                self.capacity
            );
        };
        legal
    }

    // first, methods not defined previously, but which arose while implemeneting the others (see below)
    #[inline]
    fn solution_score(&self, solution: &Self::Sol) -> ScoreType {
        let mut result = ZERO_SCORE;
        // Note to self -- later we can be faster here by doing this byte-wise
        for index in 0..self.problem_size() {
            if let Some(decision) = solution.get_decision(index) {
                if decision {
                    result += self.weights[index];
                }
            }
        } // end for all bits
        result as ScoreType
    } // end solution_is_legal

    fn solution_best_score(&self, solution: &Self::Sol) -> ScoreType {
        // add up all weights which are either open or not set to zero,
        // stopping if we get past capacity
        let mut result = ZERO_SCORE;
        for index in 0..self.problem_size() {
            match solution.get_decision(index) {
                // open decision! So we COULD put this item in the knapsack...
                None => result += self.weights[index],
                Some(decision) => {
                    if decision {
                        result += self.weights[index]
                    }
                }
            }; // end match
            if self.capacity < result {
                return self.capacity;
            }; // end if over capacity
        } // end for all bits
          // if we're here, then upper_bound is less than capacity
        debug_assert!(result <= self.capacity);
        debug_assert!(self.solution_score(&solution) <= result);
        // next assert fails if solution is complete and best_score != score
        debug_assert!(
            !self.solution_is_complete(&solution) || (self.solution_score(&solution) == result)
        );
        result
    }

    #[inline]
    fn solution_is_legal(&self, solution: &Self::Sol) -> bool {
        debug_assert!(self.problem_size() <= solution.size());
        self.solution_score(solution) <= self.capacity
    } // end solution_is_legal

    #[inline]
    fn solution_is_complete(&self, solution: &Self::Sol) -> bool {
        // assert!(self.solution_is_legal(&solution)); NOT NECESSARY!
        None == self.first_open_decision(solution)
    } // end solution_is_complete

    fn random_solution(&self) -> Self::Sol {
        // We want a complete, final solution -- so all mask bits are one --
        // which has a random selection of things in the knapsack.
        let mut result = Self::Sol::random(self.problem_size());
        debug_assert!(self.solution_is_complete(&result));
        // Take items out of knapsack iff necessary, as long as necessary, until light enough.
        if !self.solution_is_legal(&result) {
            // while illegal -- i.e. too much in knapsack (?!?)
            let mut weight = self.solution_score(&result);
            debug_assert!(self.capacity < weight);
            // Note to self -- later we can be faster here by doing this byte-wise
            for index in 0..self.problem_size() {
                if let Some(decision) = result.get_decision(index) {
                    if decision {
                        result.make_decision(index, false);
                        weight -= self.weights[index];
                        if weight < self.capacity {
                            break;
                        } // leave for loop!!
                    }
                }
            } // end for all bits in solution
            debug_assert_eq!(weight, self.solution_score(&result));
            debug_assert!(weight <= self.capacity);
        }; // end if illegal

        // store the solutions's score in the solution
        debug_assert!(self.solution_is_complete(&result));
        self.apply_rules(&mut result);
        debug_assert!(self.rules_audit_passed(&result)); // actually clear since complete

        result
    }

    #[inline]
    fn starting_solution(&self) -> Self::Sol {
        // We want an "innocent" solution, before any decision as been made,
        // So all mask bits are one. It doesn't matter what the decisions are,
        // but we set them all to false.
        let mut result = Self::Sol::new(self.problem_size());

        self.fix_scores(&mut result);
        // WAIT -- that's not quite right -- some weight might be greater than the capacity!
        self.apply_rules(&mut result);
        debug_assert!(self.rules_audit_passed(&result));
        // store the solutions's score in the solution
        // result.put_score(self.solution_score(&result));
        debug_assert_eq!(ZERO_SCORE, result.get_score());
        // result.put_best_score(self.capacity);

        debug_assert!(self.solution_is_legal(&result));
        result
    }

    //Use the default implementation of better_than()
    //Use the default implementation of can_be_better_than()

    fn first_open_decision(&self, solution: &Self::Sol) -> Option<usize> {
        // Note to self -- later we can be faster here by doing this byte-wise
        for index in 0..self.problem_size() {
            if solution.get_decision(index).is_none() {
                return Some(index);
            };
        } // end for all bits
          // if we get here, return....
        None
    }

    fn last_closed_decision(&self, solution: &Self::Sol) -> Option<usize> {
        // Note to self -- later we can be faster here by doing this byte-wise
        for index in self.problem_size()..0 {
            if None != solution.get_decision(index) {
                return Some(index);
            };
        } // end for all bits
          // if we get here, return....
        None
    }

    fn apply_rules(&self, sol: &mut Self::Sol) {
        debug_assert!(self.solution_is_legal(&sol));
        // First pass: Calculate and store weight sum.
        let mut weight = self.solution_score(sol);
        sol.put_score(weight);

        // Second pass: See if any items can no longer be put in knapsack
        // (and adjust best_score while we're at it).
        let headroom = self.capacity - weight;
        // debug_assert!( ZERO_SCORE < headroom );
        for bit in 0..self.problem_size() {
            if sol.get_decision(bit).is_none() {
                // found an open decision
                if headroom < self.weights[bit] {
                    // item cannot be put in knapsack legally
                    sol.make_decision(bit, false);
                } else {
                    weight += self.weights[bit]; // we're now calculating best possible weight
                }; // legitimately open decision
            };
        } // end for all bits
        if self.capacity < weight {
            weight = self.capacity
        };
        sol.put_best_score(weight);

        debug_assert!(self.rules_audit_passed(sol));
    }

    fn rules_audit_passed(&self, sol: &Self::Sol) -> bool {
        assert!(self.solution_is_legal(&sol));
        assert_eq!(sol.get_score(), self.solution_score(sol));
        assert_eq!(sol.get_best_score(), self.solution_best_score(sol));
        // if solution is incomplete
        let mut weight = self.solution_score(sol);
        // assert_eq!( sol.get_best_score( ), weight ); already asserted, above.
        let headroom = self.capacity - weight;
        // assert!( ZERO_SCORE < headroom );
        for bit in 0..self.problem_size() {
            // if oepn, but too heavy, return false
            if sol.get_decision(bit).is_none() {
                if headroom < self.weights[bit] {
                    error!( "Found unset implicit decision, bit {}, weight {}, headroom {}, capatcity {}!",
                            bit, self.weights[bit], headroom, self.capacity );
                    return false;
                } else {
                    // if open and not too nhavy
                    weight += self.weights[bit]; // we're now calculating best possible weight
                };
            }; // end decision is open and too heavy
        } // end for all bits
        if self.capacity < weight {
            weight = self.capacity
        };
        assert_eq!(weight, sol.get_best_score());
        true
    }

    // take the default register_one_child()

    // take the default register_children_of
} // end impl ProblemSubsetSum

///////////////////// TESTs for ProblemSubsetSum with  FirstDepthFirstSolver /////////////////////
#[cfg(test)]
mod tests {

    use super::*;

    use implementations::DepthFirstSolver;
    use optimizer::Solver;

    #[test]
    fn test_random_weights() {
        const TEST_SIZE: usize = 16;
        let mut rand_sack_a = ProblemSubsetSum::new(TEST_SIZE);

        assert_eq!(rand_sack_a.name(), "ProblemSubsetSum");

        assert!(!rand_sack_a.is_legal());
        assert_eq!(rand_sack_a.problem_size(), TEST_SIZE);
        assert_eq!(rand_sack_a.weights_sum(), 0);
        assert_eq!(rand_sack_a.capacity, 0);

        rand_sack_a.randomize();

        assert!(rand_sack_a.is_legal());
        assert_eq!(rand_sack_a.problem_size(), TEST_SIZE);

        assert_ne!(rand_sack_a.weights_sum(), 0);
        assert_ne!(rand_sack_a.capacity, 0);

        let rand_sack_b = ProblemSubsetSum::random(TEST_SIZE);

        assert!(rand_sack_b.is_legal());
        assert_eq!(rand_sack_b.problem_size(), TEST_SIZE);
        assert_ne!(rand_sack_b.weights_sum(), 0);
        assert_ne!(rand_sack_b.capacity, 0);

        let starter = rand_sack_b.starting_solution();
        assert!(rand_sack_b.is_legal());
        assert!(rand_sack_b.solution_is_legal(&starter));
        assert!(!rand_sack_b.solution_is_complete(&starter));
        assert_eq!(rand_sack_b.solution_score(&starter), ZERO_SCORE);
        assert_eq!(
            rand_sack_b.solution_best_score(&starter),
            rand_sack_b.capacity
        );
        assert_eq!(rand_sack_b.solution_score(&starter), starter.get_score());
        assert_eq!(
            rand_sack_b.solution_best_score(&starter),
            starter.get_best_score()
        );

        let thrown_dart = rand_sack_b.random_solution();
        assert!(rand_sack_b.is_legal());
        assert!(rand_sack_b.solution_is_legal(&thrown_dart));
        assert!(rand_sack_b.solution_is_complete(&thrown_dart));
        assert_ne!(rand_sack_b.solution_score(&thrown_dart), ZERO_SCORE);
        assert_ne!(
            rand_sack_b.solution_score(&thrown_dart),
            rand_sack_b.capacity
        ); // could be equal by dump luck, but very improbable
        assert_eq!(
            rand_sack_b.solution_score(&thrown_dart),
            thrown_dart.get_score()
        );
        assert_eq!(
            rand_sack_b.solution_best_score(&thrown_dart),
            thrown_dart.get_best_score()
        );
        assert!(thrown_dart.get_score() < rand_sack_b.capacity); // could be equal by dump luck
        assert!(thrown_dart.get_best_score() <= rand_sack_b.capacity);
    } // end test_random_weights

    #[test]
    fn test_random_knapsacks() {
        for size in [4, 5, 6, 7, 8, 16, 32, 64, 128, 256].iter() {
            let sack = ProblemSubsetSum::random(*size);
            assert!(
                sack.is_legal(),
                "illegal random sack with size {}?!?",
                *size
            );
        }
    }

    #[test]
    fn test_children_preduction() {
        const NUM_BITS: usize = 32; // big, to make special cases below REALLY improbable

        // Test register_children_of( )
        let problem = ProblemSubsetSum::random(NUM_BITS); // a lot smaller
        assert!(problem.is_legal());

        let mut solver = DepthFirstSolver::<MinimalSolution>::new(NUM_BITS);

        solver.push(problem.starting_solution());
        assert!(!solver.is_empty());

        let root = solver.pop().expect("Solver should let us pop SOMETHING #1");
        assert!(solver.is_empty());
        assert!(problem.solution_is_legal(&root));
        assert!(!problem.solution_is_complete(&root));

        let children = problem.children_of_solution(&root);
        assert!(!children.is_empty());
        assert!(children.len() <= 2); // So, number of children is 1 or 2
        for child in children {
            assert!(problem.solution_is_legal(&child));
            if !problem.solution_is_complete(&child) {
                solver.push(child);
            }
        }
        assert!(!solver.is_empty());
        assert!(solver.number_of_solutions() <= 2);

        let grandchild = solver.pop().expect("Solver should let us pop SOMETHING #2");
        assert!(!solver.is_empty());
        assert!(solver.number_of_solutions() <= 1);
        assert!(problem.solution_is_legal(&grandchild));
        assert!(!problem.solution_is_complete(&grandchild));

        // Before we go...
        assert!(problem.is_legal());
    } // end test_children_regstration

    #[test]
    fn test_find_depth_first_solution() {
        const NUM_DECISIONS: usize = 4; // for a start

        let little_knapsack = ProblemSubsetSum::random(NUM_DECISIONS);
        let mut first_solver = DepthFirstSolver::<MinimalSolution>::new(NUM_DECISIONS);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        assert!(little_knapsack.is_legal());
        assert!(first_solver.is_empty());

        let the_best = first_solver
            .find_best_solution(&little_knapsack, time_limit)
            .expect("could not find best solution");

        assert!(little_knapsack.solution_is_legal(&the_best));
        assert!(little_knapsack.solution_is_complete(&the_best));

        assert_eq!(
            little_knapsack.solution_score(&the_best),
            little_knapsack.capacity
        );
        assert_eq!(the_best.get_score(), little_knapsack.capacity);
        assert_eq!(the_best.get_best_score(), little_knapsack.capacity);
    }
} // end mod tests
