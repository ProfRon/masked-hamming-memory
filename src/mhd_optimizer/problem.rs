use rand::prelude::*;
use std::fmt::Debug;

use mhd_method::{Sample, ScoreType}; // Not used: NUM_BYTES
use mhd_optimizer::Solution;
// use mhd_optimizer::Solver;

/// ## The Problem Trait
///
pub trait Problem: Sized + Clone + Debug {
    // Every Problem will probably need it's own "associated" solution type
    type Sol: Solution;

    /// Every instance of this struct should have a descriptive name (for tracing, debugging).
    /// Default works, but is very long (override it to make it friendlier).
    #[inline]
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// Every instance should have a SHORT description for Debugging,
    /// giving things like a knapsack's capacity, pehaps more.
    fn short_description(&self) -> String;

    // Constructors

    /// `new` creates a default ("zero") instance of the problem,
    /// where `size` is the number of decisions to be made (free variables to assign values to).
    fn new(size: usize) -> Self;

    /// `random` creates a full-fledged, i.e. complete but random instance of the problem,
    /// where `size` is the number of decisions to be made (free variables to assign values to).
    /// Do not confuse with `random_solution`!
    #[inline]
    fn random(size: usize) -> Self {
        let mut result = Self::new(size);
        result.randomize();
        result
    }

    /// The number of decisions to be made (free variables to assign values to)-
    fn problem_size(&self) -> usize;

    /// Given a solution (self), reset all the values at random, while preserving legality.
    fn randomize(&mut self);

    /// is_legal tests whether a problem -- not whether a solution -- is legal
    /// (the Solution trait has its own is_legal method).
    /// For example, are all of the weights of a knapsack greater than zero, is the dimension
    /// greater than zero, is the capacity OK, etc.
    /// In other words, is a valid soution possible (not whether a given solution valid).
    fn is_legal(&self) -> bool;

    /// ## Solution attributes that only the problem can evaluate
    /// What is the score of a given Solution?
    fn solution_score(&self, solution: &Self::Sol) -> ScoreType;

    /// What is the "upper" bound of the score of a given Solution?
    /// Note: If we're maximizing, this is the upper bound,
    /// but if we're minimizing, this is the lower bound.
    fn solution_best_score(&self, solution: &Self::Sol) -> ScoreType;

    /// Helper function to record the score and best score of a given solution
    #[inline]
    fn fix_scores(&self, solution: &mut Self::Sol) {
        solution.put_score(self.solution_score(solution));
        solution.put_best_score(self.solution_best_score(solution));
    }

    /// Is a given solution legal *for this problem*?
    fn solution_is_legal(&self, solution: &Self::Sol) -> bool;

    /// Is a given solution complete *for this problem*?
    fn solution_is_complete(&self, solution: &Self::Sol) -> bool;

    /// ## Methods used by the Unified Optimization Algorithm (identified above)
    ///
    /// Create a random complete solution of this problem:
    fn random_solution(&self) -> Self::Sol;

    /// Create a (clone of) the starting solution for this problem,
    /// i.e. the solution with NO decisions made yet.
    fn starting_solution(&self) -> Self::Sol;

    /// Is new_solution better than old_solution?
    /// Note that the default version assumes we're maximizing.
    #[inline]
    fn better_than(&self, new_solution: &Self::Sol, old_solution: &Self::Sol) -> bool {
        old_solution.get_best_score() <= new_solution.get_best_score()
    }

    /// Is the "upper bound" of new_solution better than score the old solution?
    /// Note that the default version assumes we're maximizing.
    #[inline]
    fn can_be_better_than(&self, new_solution: &Self::Sol, old_solution: &Self::Sol) -> bool {
        self.solution_best_score(old_solution) <= self.solution_best_score(new_solution)
    }

    /// Find the index of the next decision to make (bit to set), if any,
    /// or return None if there are no more open decisions.
    fn first_open_decision(&self, solution: &Self::Sol) -> Option<usize>;

    /// Find the largest index of a closed decision, if any,
    /// or return None if there are no closed decisions
    /// (which defines the starting solution, by the way).
    fn last_closed_decision(&self, solution: &Self::Sol) -> Option<usize>;

    /// Apply this problem's own logic to check if any decisions are implicitly already decided.
    /// Example: if some items are heavier than a knapsack's remainng capacity, we don't have
    /// to consider putting them into the knapsack.
    /// Side effect: Calculate and store current score and best_score.
    /// (So you do not need to call fix_scores before or after -- thus avoiding a nasty
    /// dependency loop).
    fn apply_rules(&self, sol: &mut Self::Sol);

    /// Check if all implicit decisions have been made
    /// This function exists only for debugging purposes.
    fn rules_audit_passed(&self, sol: &Self::Sol) -> bool;

    /// `produce_child` takes a copy (clone) of `parent` and tries making the first open deccision.
    /// It returns either Some(child) or None, if the child would not have been legal,
    /// e.g. if the weight of a knapsack would exceed the capacity.
    #[inline]
    fn produce_child(&self, parent: &Self::Sol, index: usize, decision: bool) -> Self::Sol {
        let mut child = parent.clone();
        debug_assert!(self.solution_is_legal(&child));
        child.make_decision(index, decision);
        debug_assert!(self.solution_is_legal(&child));
        self.apply_rules(&mut child);
        debug_assert!(self.rules_audit_passed(&child));
        child
    } // end produce one child

    /// This method (`children_of_solution`) return zero, one or two solutions in the form of a
    /// vector. Given `parent`, the method find the first open decision, and tries setting it
    /// to both true and to false -- thus producing two children, both of which are tested for
    /// legality. Only legal children are returned (so there can be 0, 1 or 2).
    /// Actually, there should only be two, if make_implicit_decisions is correct.
    /// TODO: Fix this to return a pair, not a vector
    #[inline]
    fn children_of_solution(&self, parent: &Self::Sol) -> Vec<Self::Sol> {
        debug_assert!(self.rules_audit_passed(parent));
        debug_assert!(!self.solution_is_complete(parent));
        let mut result = Vec::<Self::Sol>::new(); // initially empty...
                                                  // parent must not be a complete solution, so we can use unwrpa in the next line:
        let index = self
            .first_open_decision(parent)
            .expect("There must be an open decision");

        // The order of the next two operations is important!
        // Try deciding TRUE
        result.push(self.produce_child(parent, index, true));

        // Try deciding FALSE
        result.push(self.produce_child(parent, index, false));

        result
    } // end children_of_solution

    fn random_completion(&self, solution: &Self::Sol, index: usize, decision: bool) -> Self::Sol {
        let mut generator = thread_rng();
        let mut result = solution.clone();
        let mut decision_num = index;
        let mut next_decision = decision;
        loop {
            debug_assert!(!self.solution_is_complete(&result));
            result.make_decision(decision_num, next_decision);
            self.apply_rules(&mut result);
            debug_assert!(self.rules_audit_passed(&result));
            if self.solution_is_complete(&result) {
                // We're done! Return!
                return result;
            } else {
                // if result not yet complete
                decision_num = self
                    .first_open_decision(&result)
                    .expect("Should be an open decision");
                next_decision = generator.gen();
            };
        } // end loop
    } // end random_completion

    #[inline]
    fn sample_from_solution(&self, solution: &Self::Sol) -> Sample {
        debug_assert!(self.solution_is_complete(solution));
        let result = Sample {
            width: self.problem_size(),
            score: self.solution_score(solution),
            bytes: solution.query().to_vec(),
        };
        result
    } // end sample_from_solution
} // end trait Problem
