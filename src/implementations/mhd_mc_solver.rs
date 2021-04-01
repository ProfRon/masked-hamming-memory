// use log::*;
// use rand::prelude::*; // for info, trace, warn, etc.

use mhd_method::*;
use mhd_optimizer::{Problem, Solution, Solver};

/// # Example Implementations
///
///
///

pub const NUM_BITS : usize = 64;
pub const NUM_BYTES : usize = 8;

pub type MhdMcSample = Sample< NUM_BITS >;
pub type MhdMcMemory = MhdMemory< NUM_BITS >;

pub struct MhdMonteCarloSolver<Sol: Solution, Prob: Problem<Sol = Sol>> {
    pub mhd_memory: MhdMcMemory,
    pub best_solution: Sol,
    pub problem: Prob,
}

impl<Sol: Solution, Prob: Problem<Sol = Sol>> MhdMonteCarloSolver<Sol, Prob> {
    // a replacement for Self::new( size )
    #[inline]
    pub fn builder(problem: &Prob) -> Self {
        Self {
            mhd_memory: MhdMcMemory::default(),
            best_solution: problem.random_solution(),
            problem: problem.clone(), // = problem, note rust syntatic sugar
        }
    }
} // end private Methods

/**************************************************************************************/
/// ## Example Solver Implementation: MCTS, Monte Carlo Tree Search
///
/// Here are the public methods needed to implement Solver<Sol>
impl<Sol: Solution, Prob: Problem<Sol = Sol>> Solver<Sol> for MhdMonteCarloSolver<Sol, Prob> {
    #[inline]
    fn name(&self) -> &'static str {
        "MhdMonteCarloSolver "
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{}, memory has width {} and {} rows",
            self.name(),
            self.mhd_memory.width(),
            self.mhd_memory.num_samples()
        )
    }

    #[inline]
    fn new(_: usize) -> Self {
        panic!("New(size) not define for MonteCarloTreeSolver!");
    }

    // Methods used by the Unified Optimization Algorithm (identified above)

    #[inline]
    fn number_of_solutions(&self) -> usize {
        self.mhd_memory.num_samples()
    }

    #[inline]
    fn is_empty(&self) -> bool { self.mhd_memory.is_empty() }

    #[inline]
    fn clear(&mut self) {
        self.mhd_memory.clear();
        let width = self.best_solution.size();
        self.best_solution = Sol::new( width );
    }

    #[inline]
    fn push(&mut self, solution: Sol) {
        // we'd like to check for completion, but can't use proble.solution_is_complete( s )
        if self.best_score() < solution.get_score() {
            panic!("Push not implemented!");
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<Sol> {
        panic!("Pop not implemented!");
    }

    #[inline]
    fn best_solution(&self) -> &Sol {
        &self.best_solution
    }

    #[inline]
    fn store_best_solution(&mut self, solution: Sol) {
        // we'd like to check for completion, but can't use proble.solution_is_complete( s )
        debug_assert_eq!(solution.get_score(), solution.get_best_score());
        // Occasionally, the following condition IS allowed (to be false)
        // debug_assert!(self.best_score() <= solution.get_score());
        self.best_solution = solution;
    }
} // end imp Solver for MonteCarloTreeSolver

