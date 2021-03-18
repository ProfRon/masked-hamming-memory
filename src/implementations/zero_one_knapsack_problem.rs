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
use rand_distr::{Distribution, Poisson};

use implementations::ProblemSubsetSum;
use mhd_method::{ScoreType, NUM_BITS, ZERO_SCORE}; // Not used: NUM_BYTES
use mhd_optimizer::{MinimalSolution, Solution};
use mhd_optimizer::{Problem, Solver};

/********************************************************************************************/
///## Customized Solution Type for the 0/1 Knapsack
/// The MinimalSolution will not suffice here (experience teaches us).
/// So we define our own before we go further.
///

#[derive(Debug, Clone)]
pub struct ZeroOneKnapsackSolution {
    pub basis: MinimalSolution,
    pub score: ScoreType,
    pub best_score: ScoreType, // best score possible
}

impl Solution for ZeroOneKnapsackSolution {
    // type ScoreType = ScoreType;

    fn name(&self) -> &'static str {
        "ZeroOneKnapsackSolution"
    }

    fn short_description(&self) -> String {
        format!(
            "{}: weight {}, score {}, best score {}",
            self.name(),
            self.basis.get_score(),
            self.get_score(),
            self.get_best_score()
        )
    }

    fn new(size: usize) -> Self {
        assert!(NUM_BITS <= size);
        Self {
            basis: MinimalSolution::new(size),
            score: 0 as ScoreType,
            best_score: 0 as ScoreType,
        }
    }

    fn randomize(&mut self) {
        self.basis.randomize();
        let mut generator = rand::thread_rng();
        self.score = generator.gen();
        self.best_score = self.score + generator.gen::<ScoreType>();
    }

    // Experimental heuristic!!
    // Here, we estimate the urgendy (scheduling priority) of a solution
    // as its value (its score) divied by its weight (the score of the basis),
    // i.e. the density of the value per kilogram, so to apeak...
    // Note: We add one to weight to avoid dividing by zero,
    // and multiply by 100 to get percent, actually to compensate for integer return value
    fn estimate(&self) -> ScoreType {
        // 100 * self.get_score() / (1 + self.basis.get_score())
        self.get_score( )
    }

    // Getters and Setters
    fn get_score(&self) -> ScoreType {
        self.score
    }

    fn put_score(&mut self, score: ScoreType) {
        self.score = score;
    }

    fn get_best_score(&self) -> ScoreType {
        self.best_score
    }

    fn put_best_score(&mut self, best: ScoreType) {
        self.best_score = best;
    }

    fn get_decision(&self, decision_number: usize) -> Option<bool> {
        self.basis.get_decision(decision_number)
    }

    fn make_decision(&mut self, decision_number: usize, decision: bool) {
        self.basis.make_decision(decision_number, decision);
    }
} // end impl Soluton for ZeroOneKnapsackSolution

/// ## Default Sorting Implementations (hopefully allowed)
use std::cmp::*;

// Ord requires Eq, which requires PartialEq
impl PartialEq for ZeroOneKnapsackSolution {
    fn eq(&self, other: &Self) -> bool {
        self.estimate() == other.estimate()
    }
}

impl Eq for ZeroOneKnapsackSolution {}

impl Ord for ZeroOneKnapsackSolution {
    fn cmp(&self, other: &Self) -> Ordering {
        self.estimate().cmp(&other.estimate())
    }
}

impl PartialOrd for ZeroOneKnapsackSolution {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.estimate().cmp(&other.estimate()))
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
        result.put_score(self.solution_score(&result));
        result.put_best_score(self.solution_best_score(&result));
        debug_assert!(self.solution_is_legal(&result));
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
        let distr = Poisson::new(50.0).unwrap();

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
        let mut result = self.solution_score(&solution);
        for index in 0..self.problem_size() {
            if None == solution.get_decision(index) {
                result += self.values[index];
            }
        } // end for all bits
        debug_assert!(self.solution_score(&solution) <= result);
        debug_assert!(
            !self.solution_is_complete(&solution) || (self.solution_score(&solution) == result)
        );
        result
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

    fn make_implicit_decisions(&self, sol: &mut Self::Sol) {
        self.basis.make_implicit_decisions(&mut sol.basis);
        // If there were any constraints on decisions that depeded on values,
        // we would have to do more work -- but there aren't (are there?), so we're done!
    }
    fn register_one_child(
        &self,
        parent: &Self::Sol,
        solver: &mut impl Solver<Self::Sol>,
        index: usize,
        decision: bool,
    ) {
        // sadly, self.sack.register_one_child( parent, solver, index, decision )
        // doesn't work, because it assigns the wrong score to the child.
        // Fascinatingly, it DOES work if we cut and paste the code!  :-\
        let mut new_solution = parent.clone();
        new_solution.make_decision(index, decision);
        // Add weight (may be taken off later)
        self.basis.fix_scores(&mut new_solution.basis);
        self.make_implicit_decisions(&mut new_solution);
        if self.solution_is_legal(&new_solution) {
            self.fix_scores(&mut new_solution);
            debug_assert_eq!(new_solution.get_score(), self.solution_score(&new_solution));
            debug_assert_eq!(
                new_solution.basis.get_score(),
                self.basis.solution_score(&new_solution.basis)
            );
            solver.push(new_solution);
        } // else if solution is illegal, do nothinng
    }

    fn register_children_of(&self, parent: &Self::Sol, solver: &mut impl Solver<Self::Sol>) {
        // it would be nice to just call self.sack.register_children_of(),
        // since we're not changing the codem,
        // but it would call the wrong self.register_one_chilld (!).
        debug_assert!(self.solution_is_legal(parent));
        match self.first_open_decision(parent) {
            None => {} // do nothing!
            Some(index) => {
                self.register_one_child(parent, solver, index, false);
                self.register_one_child(parent, solver, index, true);
            } // end if found Some(index) -- an open decision
        } // end match
    } // end register_children
} // end impl ProblemSubsetSum

/********************************************************************************************/
///////////////////// TESTs for ProblemSubsetSum with  FirstDepthFirstSolver /////////////////
#[cfg(test)]
mod tests {

    use super::*;
    use implementations::{DepthFirstSolver, Problem01Knapsack, ZeroOneKnapsackSolution};
    use mhd_optimizer::{Problem, Solution};

    #[test]
    fn test_random_weights() {
        const TEST_SIZE: usize = 8;
        let mut rand_sack_a = Problem01Knapsack::new(TEST_SIZE);

        assert_eq!(rand_sack_a.name(), "Problem01Knapsack");

        assert!(!rand_sack_a.is_legal());
        assert_eq!(rand_sack_a.problem_size(), TEST_SIZE);
        assert_eq!(rand_sack_a.weights_sum(), 0);

        rand_sack_a.randomize();

        assert!(rand_sack_a.is_legal());
        assert_eq!(rand_sack_a.problem_size(), TEST_SIZE);

        assert_ne!(rand_sack_a.weights_sum(), 0);
        assert_ne!(rand_sack_a.values_sum(), 0);
        assert_ne!(rand_sack_a.capacity(), 0);

        let rand_sack_b = Problem01Knapsack::random(TEST_SIZE);

        assert!(rand_sack_b.is_legal());
        assert_eq!(rand_sack_b.problem_size(), TEST_SIZE);
        assert_ne!(rand_sack_b.weights_sum(), 0);
        assert_ne!(rand_sack_b.values_sum(), 0);
        assert_ne!(rand_sack_b.capacity(), 0);

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

        problem.register_children_of(&root, &mut solver);
        assert!(!solver.is_empty());
        assert!(solver.number_of_solutions() <= 2);

        let node_a = solver.pop().expect("Solver should let us pop SOMETHING #2");
        // assert!( ! solver.is_empty() );
        assert!(solver.number_of_solutions() <= 1);
        assert!(problem.solution_is_legal(&node_a));
        assert!(!problem.solution_is_complete(&node_a));

        problem.register_children_of(&node_a, &mut solver);
        assert!(!solver.is_empty());
        assert!(solver.number_of_solutions() <= 3);

        let node_b = solver.pop().expect("Solver should let us pop SOMETHING #3");
        // assert!( ! solver.is_empty() );
        assert!(solver.number_of_solutions() <= 2);
        assert!(problem.solution_is_legal(&node_b));
        assert!(!problem.solution_is_complete(&node_b));

        problem.register_children_of(&node_b, &mut solver);
        assert!(!solver.is_empty());
        assert!(solver.number_of_solutions() <= 4);

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

        let the_best = little_knapsack
            .find_best_solution(&mut first_solver, time_limit)
            .expect("could not find best solution");
        assert!(little_knapsack.solution_is_legal(&the_best));
        assert!(little_knapsack.solution_is_complete(&the_best));

        let best_score = the_best.get_score();
        assert!(ZERO_SCORE < best_score);
        assert_eq!(best_score, little_knapsack.solution_score(&the_best));
        assert_eq!(best_score, little_knapsack.solution_best_score(&the_best));
    }
} // end mod tests
