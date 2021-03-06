/// # Example Implementations
///
/// ## Example Problem Implementation: 0-1 Knapsack
///
/// This struct extends the subset sum struct to implement the "real" or "full valued"
/// zero-one knapsack problem.
/// Each item that can go in the knapsack has a value.
/// The function to optimize (maximize) is now the sum of the values (a.k.a. profits) of
/// the items in the knapsack,
/// while still maintaining the constraint on the sum of the weights (the "capacity").
///
extern crate rand_distr;

use rand::prelude::*;
use rand_distr::{Distribution, Gamma};

use implementations::ProblemSubsetSum;
use mhd_memory::{ScoreType, ZERO_SCORE}; // Not used: NUM_BYTES
use optimizer::{MinimalSolution, PriorityType, Problem, Solution};

/********************************************************************************************/
///## Customized Solution Type for the 0/1 Knapsack
/// The MinimalSolution will not suffice here (experience teaches us).
/// So we define our own before we go further.
///

#[derive(Debug, Clone, PartialEq)]
pub struct ZeroOneKnapsackSolution {
    pub basis: MinimalSolution,
    pub score: ScoreType,
    pub best_score: ScoreType, // best score possible
                               // no priority field -- we use basis.priority!
}

impl Solution for ZeroOneKnapsackSolution {
    // type ScoreType = ScoreType;
    // type PriorityType = <MinimalSolution as Solution>::PriorityType;

    // Default is too long; here is a friendlier version of name()
    #[inline]
    fn name(&self) -> &'static str {
        "ZeroOneKnapsackSolution"
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{}: weight {}, score {}, best score {}",
            self.name(),
            self.basis.get_score(),
            self.get_score(),
            self.get_best_score()
        )
    }

    #[inline]
    fn new(size: usize) -> Self {
        Self {
            basis: MinimalSolution::new(size),
            score: ZERO_SCORE,
            best_score: ZERO_SCORE,
        }
    }

    #[inline]
    fn randomize(&mut self) {
        self.basis.randomize();
        let mut generator = rand::thread_rng();
        self.score = generator.gen();
        self.best_score = self.score + generator.gen::<ScoreType>();
    }

    #[inline]
    fn priority(&self) -> PriorityType {
        self.basis.priority()
    }

    #[inline]
    fn set_priority(&mut self, prio: PriorityType) {
        self.basis.priority = prio
    }

    // Getters and Setters
    #[inline]
    fn size(&self) -> usize {
        self.basis.size()
    }

    #[inline]
    fn get_score(&self) -> ScoreType {
        self.score
    }

    #[inline]
    fn put_score(&mut self, score: ScoreType) {
        self.score = score;
    }

    #[inline]
    fn get_best_score(&self) -> ScoreType {
        self.best_score
    }

    #[inline]
    fn put_best_score(&mut self, best: ScoreType) {
        self.best_score = best;
    }

    #[inline]
    fn mask(&self) -> &[u8] {
        self.basis.mask()
    }

    #[inline]
    fn query(&self) -> &[u8] {
        &self.basis.query()
    }

    #[inline]
    fn get_decision(&self, decision_number: usize) -> Option<bool> {
        self.basis.get_decision(decision_number)
    }

    #[inline]
    fn make_decision(&mut self, decision_number: usize, decision: bool) {
        self.basis.make_decision(decision_number, decision);
    }
} // end impl Soluton for ZeroOneKnapsackSolution

/// ## Default Sorting Implementations (hopefully allowed)
use std::cmp::*;

// Ord requires Eq, which requires PartialEq
// impl PartialEq for ZeroOneKnapsackSolution {
//     fn eq(&self, other: &Self) -> bool {
//         self == other
//     }
// }

impl Eq for ZeroOneKnapsackSolution {}

impl Ord for ZeroOneKnapsackSolution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority()
            .partial_cmp(&other.priority())
            .expect("Ordering")
    }
}

impl PartialOrd for ZeroOneKnapsackSolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority().partial_cmp(&other.priority())
    }
}

/********************************************************************************************/
/// ## Example Problem Implementation: 0-1 Knapsack
/// Here the actual Struct:
#[derive(Debug, Clone)]
pub struct Problem01Knapsack {
    pub basis: ProblemSubsetSum,
    pub values: Vec<ScoreType>,
} // end struct Problem01Knapsack

// Utility Methods (not part of the Problem trait)
impl Problem01Knapsack {
    // type ScoreType = ZeroOneKnapsackSolution::ScoreType;

    pub fn weights_sum(&self) -> ScoreType {
        self.basis.weights_sum()
    }

    pub fn values_sum(&self) -> ScoreType {
        self.values.iter().sum()
    }

    pub fn capacity(&self) -> ScoreType {
        self.basis.capacity
    }

    pub fn solution_from_basis(&self, starter_basis: &MinimalSolution) -> ZeroOneKnapsackSolution {
        let mut result = ZeroOneKnapsackSolution {
            basis: starter_basis.clone(),
            score: ZERO_SCORE,
            best_score: ZERO_SCORE,
        };
        self.apply_rules(&mut result);
        debug_assert!(self.rules_audit_passed(&result));
        result
    }
}

// Problem Trait Methods
impl Problem for Problem01Knapsack {
    type Sol = ZeroOneKnapsackSolution;

    fn name(&self) -> &'static str {
        "Problem01Knapsack"
    }

    fn short_description(&self) -> String {
        format!(
            "{} {}, value sum {}",
            self.name(),
            self.basis.short_description(),
            self.values_sum()
        )
    }

    fn new(size: usize) -> Self {
        Self {
            basis: ProblemSubsetSum::new(size),
            values: vec![ZERO_SCORE; size],
        }
    }

    // fn random( size : usize ) -> Self -- take the default implementation

    fn problem_size(&self) -> usize {
        self.values.len()
    }

    fn randomize(&mut self) {
        self.basis.randomize(); // Sets weights and capacity
        let num_bits = self.problem_size();
        assert_eq!(num_bits, self.values.len(), "Values vector has wrong size");

        self.basis.randomize();

        // self.weights =  (0..self.problem_size()).map( |_| fancy_random_int( ) ).collect();
        let mut rng = rand::thread_rng();
        // The parameters shape=2.0 and scale=1000.0 were arrived at by playing around in a
        // Jupyter Notebook but remain failry arbitrary
        let distr = Gamma::new(2.0, 1000.0).unwrap();

        self.values = (0..num_bits)
            .map(|_| (distr.sample(&mut rng) + 1.0) as ScoreType)
            .collect();

        // This has been removed to not make the problem TOO easy...
        // self.values.sort_unstable();
        // self.values.reverse();

        debug_assert!(self.is_legal());
    }

    fn is_legal(&self) -> bool {
        // Note: We're NOT testing whether a solution is legal (we do that below),
        // We're testing if a PROBLEM is OK.
        // The subset sum part must be legal -- and then the values too...
        // We used to check if there were zero values, but now we don't...
        // Some of the files we've parsed have zero values. OK...  Why not?!?
        self.basis.is_legal() && (self.problem_size() == self.values.len())
    }

    // first, methods not defined previously, but which arose while implemeneting the others (see below)
    fn solution_score(&self, solution: &Self::Sol) -> ScoreType {
        let mut result = ZERO_SCORE;
        // Note to self -- later we can be faster here by doing this byte-wise
        for index in 0..self.problem_size() {
            if let Some(decision) = solution.get_decision(index) {
                if decision {
                    result += self.values[index];
                }
            }
        } // end for all bits
        result as ScoreType
    } // end solution_is_legal

    fn solution_best_score(&self, solution: &Self::Sol) -> ScoreType {
        debug_assert!(self.solution_is_legal(solution));
        // add up all values which are either open or not set to zero,
        let mut result = ZERO_SCORE;
        for index in 0..self.problem_size() {
            match solution.get_decision(index) {
                // open decision! So we COULD put this item in the knapsack...
                None => result += self.values[index],
                Some(decision) => {
                    if decision {
                        result += self.values[index]
                    } // else add zero, i.e. do nothing
                }
            }; // end match
        } // end for all bits
        debug_assert!(self.solution_score(&solution) <= result);
        // next assert fails if solution is complete and best_score != score
        debug_assert!(
            !self.solution_is_complete(&solution) || (self.solution_score(&solution) == result)
        );
        result
    }

    fn fix_scores(&self, solution: &mut Self::Sol) {
        // the next line is the (only) reason we didn't just take the default
        self.basis.fix_scores(&mut solution.basis);
        solution.put_score(self.solution_score(solution));
        solution.put_best_score(self.solution_best_score(solution));
    }

    fn solution_is_legal(&self, solution: &Self::Sol) -> bool {
        self.basis.solution_is_legal(&solution.basis)
    } // end solution_is_legal

    fn solution_is_complete(&self, solution: &Self::Sol) -> bool {
        self.basis.solution_is_complete(&solution.basis)
    } // end solution_is_complete

    fn random_solution(&self) -> Self::Sol {
        self.solution_from_basis(&self.basis.random_solution())
    }

    fn starting_solution(&self) -> Self::Sol {
        self.solution_from_basis(&self.basis.starting_solution())
    }

    // Take the default better_than() method
    // Take the default can_be_better_than() method

    fn first_open_decision(&self, solution: &Self::Sol) -> Option<usize> {
        self.basis.first_open_decision(&solution.basis)
    }

    fn last_closed_decision(&self, solution: &Self::Sol) -> Option<usize> {
        self.basis.last_closed_decision(&solution.basis)
    }

    fn apply_rules(&self, sol: &mut Self::Sol) {
        debug_assert!(self.solution_is_legal(&sol));
        self.basis.apply_rules(&mut sol.basis);
        // self.basis now has a correct score (knapsack's weight) and best_score.
        // Further, all implicit decisions have been made!
        // We COULD just call solution_score and solution_best_score, but why do two
        // passes over the decisions when we can do both at once?
        let mut min_value = ZERO_SCORE;
        let mut max_value = ZERO_SCORE;
        for bit in 0..self.problem_size() {
            match sol.get_decision(bit) {
                None => max_value += self.values[bit],
                Some(decision) => {
                    if decision {
                        max_value += self.values[bit];
                        min_value += self.values[bit];
                    }
                }
            } // end match decision (option)
        } // end for all bits
        sol.put_score(min_value);
        sol.put_best_score(max_value);

        debug_assert!(self.rules_audit_passed(sol));
    }

    fn rules_audit_passed(&self, sol: &Self::Sol) -> bool {
        assert!(self.solution_is_legal(&sol));
        assert!(self.basis.rules_audit_passed(&sol.basis));
        // We COULD just call solution_score and solution_best_score, but why do two
        // passes over the decisions when we can do both at once?
        let mut min_value = ZERO_SCORE;
        let mut max_value = ZERO_SCORE;
        for bit in 0..self.problem_size() {
            match sol.get_decision(bit) {
                None => max_value += self.values[bit],
                Some(decision) => {
                    if decision {
                        max_value += self.values[bit];
                        min_value += self.values[bit];
                    }
                }
            } // end match decision (option)
        } // end for all bits
        assert_eq!(min_value, sol.get_score());
        assert_eq!(max_value, sol.get_best_score());
        assert_eq!(sol.get_score(), self.solution_score(sol));
        assert_eq!(sol.get_best_score(), self.solution_best_score(sol));
        true
    }
} // end impl ProblemSubsetSum

/********************************************************************************************/
///////////////////// TESTs for ProblemSubsetSum with  FirstDepthFirstSolver /////////////////
#[cfg(test)]
mod tests {

    use super::*;
    use implementations::{DepthFirstSolver, Problem01Knapsack, ZeroOneKnapsackSolution};
    use log::*;
    use optimizer::{Problem, Solution, Solver};

    #[test]
    fn test_random_weights() {
        const TEST_SIZE: usize = 8;
        debug!("Testing new (blank) 01Knapsack...");
        let mut rand_sack_a = Problem01Knapsack::new(TEST_SIZE);

        assert_eq!(rand_sack_a.name(), "Problem01Knapsack");

        assert!(!rand_sack_a.is_legal());
        assert_eq!(rand_sack_a.problem_size(), TEST_SIZE);
        assert_eq!(rand_sack_a.weights_sum(), 0);

        trace!("Testing randomized 01Knapsack...");
        rand_sack_a.randomize();

        assert!(rand_sack_a.is_legal());
        assert_eq!(rand_sack_a.problem_size(), TEST_SIZE);

        assert_ne!(rand_sack_a.weights_sum(), 0);
        assert_ne!(rand_sack_a.values_sum(), 0);
        assert_ne!(rand_sack_a.capacity(), 0);

        debug!("Testing random 01Knapsack...");
        let rand_sack_b = Problem01Knapsack::random(TEST_SIZE);

        assert!(rand_sack_b.is_legal());
        assert_eq!(rand_sack_b.problem_size(), TEST_SIZE);
        assert_ne!(rand_sack_b.weights_sum(), 0);
        assert_ne!(rand_sack_b.values_sum(), 0);
        assert_ne!(rand_sack_b.capacity(), 0);

        debug!("Testing random 01Knapsack's starting solution...");
        let starter = rand_sack_b.starting_solution();
        assert!(rand_sack_b.is_legal());
        assert!(rand_sack_b.solution_is_legal(&starter));
        assert!(!rand_sack_b.solution_is_complete(&starter));
        assert_eq!(rand_sack_b.solution_score(&starter), ZERO_SCORE);
        assert_eq!(rand_sack_b.solution_score(&starter), starter.get_score());
        assert_eq!(
            rand_sack_b.solution_best_score(&starter),
            starter.get_best_score()
        );

        debug!("Testing random 01Knapsack's random solution...");
        let thrown_dart = rand_sack_b.random_solution();
        assert!(rand_sack_b.is_legal());
        assert!(rand_sack_b.solution_is_legal(&thrown_dart));
        assert!(rand_sack_b.solution_is_complete(&thrown_dart));
        assert_ne!(rand_sack_b.solution_score(&thrown_dart), ZERO_SCORE);
        assert_eq!(
            rand_sack_b.solution_score(&thrown_dart),
            thrown_dart.get_score()
        );
        assert_eq!(
            rand_sack_b.solution_best_score(&thrown_dart),
            thrown_dart.get_best_score()
        );
    } // end test_random_weights

    #[test]
    fn test_random_knapsacks() {
        for size in [4, 5, 6, 7, 8, 16, 32, 64, 128, 256].iter() {
            debug!("Testing random 01Knapsack size {}", size);
            let sack = Problem01Knapsack::random(*size);
            assert!(
                sack.is_legal(),
                "illegal random sack with size {}?!?",
                *size
            );
        }
    }

    #[test]
    fn test_children_regstration() {
        const NUM_BITS: usize = 32; // big, to make special cases below REALLY improbable

        // Test register_children_of( )
        let problem = Problem01Knapsack::random(NUM_BITS); // a lot smaller
        assert!(problem.is_legal());

        let mut solver = DepthFirstSolver::<ZeroOneKnapsackSolution>::new(NUM_BITS);

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

        let little_knapsack = Problem01Knapsack::random(NUM_DECISIONS);
        let mut first_solver = DepthFirstSolver::new(NUM_DECISIONS);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        assert!(little_knapsack.is_legal());

        debug!("About to call find_best_solution (01knapsack, depthFirst solver...");
        let the_best = first_solver
            .find_best_solution(&little_knapsack, time_limit)
            .expect("could not find best solution");
        assert!(little_knapsack.solution_is_legal(&the_best));
        assert!(little_knapsack.solution_is_complete(&the_best));

        let best_score = the_best.get_score();
        assert!(ZERO_SCORE < best_score);
        assert_eq!(best_score, little_knapsack.solution_score(&the_best));
        assert_eq!(best_score, little_knapsack.solution_best_score(&the_best));
    }
} // end mod tests
