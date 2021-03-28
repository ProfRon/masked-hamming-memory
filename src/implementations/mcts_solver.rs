use log::*;
use rand::prelude::*; // for info, trace, warn, etc.

use mhd_method::{ScoreType, ZERO_SCORE}; // ScoreType not needed (?!?)

/// # Example Implementations
///
///
///
use mhd_optimizer::{Problem, Solution, Solver};

/**************************************************************************************/
// Helper Struct -- the MCTS Tree Node Struct
#[derive(Debug, Clone)]
pub struct MonteTreeNode {
    pub exhausted: bool,
    pub counter: usize,
    pub max_score: ScoreType,
    pub true_branch: Option<Box<MonteTreeNode>>,
    pub false_branch: Option<Box<MonteTreeNode>>,
}

type UcbType = f64;
const UCB_ZERO: UcbType = 0.0 as UcbType;
const UCB_MAX: UcbType = ScoreType::MAX as UcbType;
const UCB_C_P: UcbType = 2.828427125 as UcbType; // 2 x sqrt(2), subject to change, see Jupyterbook.

impl MonteTreeNode {
    #[inline]
    pub fn new() -> Self {
        Self {
            exhausted: false,
            max_score: ZERO_SCORE,
            counter: 0,
            true_branch: None,
            false_branch: None,
        }
    }

    // Here a synonym for building the root of the whole tree
    #[inline]
    pub fn root() -> Self {
        Self::new()
    }

    pub fn clear(&mut self) {
        // A little tricky .. we do NOT clear the node itself (no way to do so)
        // but rather clear the subbranches and set counter to zero
        if let Some(true_box) = &mut self.true_branch {
            true_box.clear();
            self.true_branch = None; // Do I have to call Drop?!?
        }; // end if true_branch not None
        if let Some(false_box) = &mut self.false_branch {
            false_box.clear();
            self.false_branch = None; // Do I have to call Drop?!?
        }; // end if true_branch not None
           // next two lines only necessary for root, but cheap...
        self.max_score = ZERO_SCORE;
        self.counter = 0;
    }

    pub fn ucts_value(&self, parent_counter: usize, high_score: ScoreType) -> UcbType {
        // avoid dividing by zero
        if 0 == self.counter {
            UCB_MAX
        } else if self.exhausted {
            UCB_ZERO
        } else {
            // if 0 < self.counter and not exhausted
            let n_j = self.counter as UcbType;
            assert!(0 != parent_counter);
            let parent_n = parent_counter as UcbType;

            // left term -- the exploitation term
            let exploitation = (self.max_score as UcbType) / (high_score as UcbType);

            // right summand -- the exploration term
            let exploration = (parent_n.ln() / n_j).sqrt() * UCB_C_P;

            // DONE! Return the sum of...
            exploitation + exploration
        } // end if 0 < counter
    } // end ucts_value

    fn ucts_branch_ucb(
        branch: &Option<Box<MonteTreeNode>>,
        parent_counter: usize,
        high_score: ScoreType,
    ) -> UcbType {
        match branch {
            None => UCB_MAX,
            Some(boxed_node) => boxed_node.ucts_value(parent_counter, high_score),
        }
    } // end ucts_branch_value

    fn best_ucb_branch(&self, full_monte: bool, high_score: ScoreType) -> bool {
        let true_subtree_ucb = Self::ucts_branch_ucb(&self.true_branch, self.counter, high_score);
        let false_subtree_ucb = Self::ucts_branch_ucb(&self.false_branch, self.counter, high_score);
        if full_monte {
            let sum_ucbs = true_subtree_ucb + false_subtree_ucb;
            let true_probability = true_subtree_ucb / sum_ucbs;
            debug_assert!(0.0 <= true_probability);
            debug_assert!(true_probability <= 1.0);
            let coin_flip: bool = rand::thread_rng().gen_bool(true_probability);
            debug!(
                "Full Monte! p(1) = {}, coin flip = {}",
                true_probability, coin_flip
            );
            coin_flip
        } else {
            // if NOT full_monte
            if false_subtree_ucb < true_subtree_ucb {
                true
            } else if false_subtree_ucb < true_subtree_ucb {
                false
            } else {
                // when branches are equal, choose at random
                let coin_flip: bool = rand::thread_rng().gen();
                coin_flip
            } // end if equal
        }
    } // end best_ucb_branch

    fn grow_tree<Sol: Solution, Prob: Problem<Sol = Sol>>(
        &mut self,
        problem: &Prob,
        solution: &mut Sol,
        full_monte: bool,
        high_score: ScoreType,
    ) -> ScoreType {
        if self.exhausted {
            trace!("Top of grow_tree, node is exhausted!");
            // We've been here before. We can handle this quickly.
            // Most importantly, do NOT recurse, do NOT complete this solution,
            // and do not consider its score.
            self.max_score
        } else if !problem.solution_is_legal(solution) {
            trace!("Top of grow_tree, solution illegal!");
            self.exhausted = true;
            assert_eq!(ZERO_SCORE, self.max_score);
            ZERO_SCORE
        } else if problem.solution_is_complete(solution) {
            trace!(
                "Top of grow_tree, COMPLETE solution score {} (high score {})",
                solution.get_score(),
                high_score
            );
            // complete and legal
            self.exhausted = true;
            let new_score = problem.solution_score(solution);
            if self.max_score < new_score {
                self.max_score = new_score
            };
            // we could call self.store_best_solution now already, but...
            // we won't need it until later!
            new_score
        } else {
            // end if solution is incomplete but legal and self NOT exhausted

            trace!(
                "Top of grow_tree, depth {}, solution score {} (high score {})",
                problem
                    .first_open_decision(solution)
                    .expect("Must have an open decsion"),
                solution.get_score(),
                high_score
            );

            self.counter += 1;

            // decide on a branch!
            let decision = self.best_ucb_branch(full_monte, high_score);

            // Fix solution ... compare Problem::produce_children()
            let index = problem
                .first_open_decision(solution)
                .expect("Should have an open decision");
            solution.make_decision(index, decision);
            problem.make_implicit_decisions(solution);
            problem.fix_scores(solution);

            // We do NOT check legality or completeness here,
            // those will be tesed on the recursive call.

            let choosen_branch = match decision {
                true => &mut self.true_branch,
                false => &mut self.false_branch,
            };
            // if the choosen branch is not there, put it there.
            if choosen_branch.is_none() {
                *choosen_branch = Some(Box::new(MonteTreeNode::new()));
            };

            // unbox the choosen node
            assert!(choosen_branch.is_some());
            if let Some(boxed_node) = choosen_branch {
                // Recurse!
                let new_score = boxed_node.grow_tree(problem, solution, full_monte, high_score);

                if self.max_score < new_score {
                    self.max_score = new_score
                };

                // We don't have to update self.best_solution here -- we do that when this method
                // is finished (after unrolling all the recursion.

                // check for exhaustion
                //self.exhausted = match ( &self.true_branch, &self.false_branch ) {
                //    ( Some( true_box ), Some( false_box) ) => { true_box.exhausted && false_box.exhausted },
                //    _ => { self.exhausted }, // i.e. NOP, Do Nothing
                //};
                if let (Some(true_box), Some(false_box)) = (&self.true_branch, &self.false_branch) {
                    self.exhausted = true_box.exhausted && false_box.exhausted;
                };
                // return
                new_score
            } else {
                // if let Some( boxed_node ) didn't work
                panic!("Rust is broken! Found 'None' after explicitly getting rid of it!");
            } // end if rust is broken
        } // end if solution incomplete
    } // end grow_tree
} // end impl MonteTreeNode

/**************************************************************************************/
/// ## Example Solver Implementation: MCTS, Monte Carlo Tree Search
///
/// Here are the internal methods, above and beyond (or perhaps rather "beneath")
/// those needed to implement the `Solver` trait (see belw)
///
#[derive(Debug, Clone)]
pub struct MonteCarloTreeSolver<Sol: Solution, Prob: Problem<Sol = Sol>> {
    full_monte: bool,
    mcts_root: MonteTreeNode,
    best_solution: Sol,
    problem: Prob,
}

impl<Sol: Solution, Prob: Problem<Sol = Sol>> MonteCarloTreeSolver<Sol, Prob> {
    // a replacement for Self::new( size )
    pub fn builder( problem: &Prob) -> Self {
        Self {
            full_monte: false, // until overwritten with true
            mcts_root: MonteTreeNode::root(),
            best_solution: problem.random_solution(),
            problem : problem.clone(), // = problem, note rust syntatic sugar
        }
    }
} // end private Methods

/**************************************************************************************/
/// ## Example Solver Implementation: MCTS, Monte Carlo Tree Search
///
/// Here are the public methods needed to implement Solver<Sol>
impl<Sol: Solution, Prob: Problem<Sol = Sol>> Solver<Sol> for MonteCarloTreeSolver<Sol, Prob> {
    #[inline]
    fn name(&self) -> &'static str {
        "MonteCarloTreeSolver "
    }

    #[inline]
    fn short_description(&self) -> String {
        format!(
            "{} holding tree with counter {}",
            self.name(),
            self.number_of_solutions()
        )
    }

    #[inline]
    fn new(_: usize) -> Self {
        panic!("New(size) not define for MonteCarloTreeSolver!");
    }

    // Methods used by the Unified Optimization Algorithm (identified above)

    #[inline]
    fn number_of_solutions(&self) -> usize {
        self.mcts_root.counter
    }

    #[inline]
    fn is_empty(&self) -> bool {
        0 == self.mcts_root.counter
    }

    #[inline]
    fn clear(&mut self) {
        self.mcts_root.clear()
    }

    #[inline]
    fn push(&mut self, solution: Sol) {
        // we'd like to check for completion, but can't use proble.solution_is_complete( s )
        if self.best_score() < solution.get_score() {
            self.store_best_solution(solution);
        }
    }

    #[inline]
    fn pop(&mut self) -> Option<Sol> {
        let mut result = self.problem.starting_solution();
        let score = self.mcts_root.grow_tree(
            & self.problem,
            &mut result,
            self.full_monte,
            self.best_score(),
        );
        debug!("Pop called grow_tree, got back {}", score);
        Some(result)
    }

    #[inline]
    fn best_solution(&self) -> &Sol {
        &self.best_solution
    }

    #[inline]
    fn store_best_solution(&mut self, solution: Sol) {
        // we'd like to check for completion, but can't use proble.solution_is_complete( s )
        debug_assert!(solution.get_score() == solution.get_best_score());
        debug_assert!(self.best_score() <= solution.get_score());
        self.best_solution = solution;
    }
} // end imp Solver for MonteCarloTreeSolver

/**************************************************************************************/
//////////////// TESTs for ProblemSubsetSum with  MonteCarloTreeSolver /////////////////
#[cfg(test)]
mod more_tests {
    use super::*;
    use implementations::ProblemSubsetSum;
    use mhd_optimizer::{MinimalSolution, Problem, Solution, Solver};

    const NUM_DECISIONS: usize = 64; // for a start

    #[test]
    fn test_monte_tree() {
        let problem = ProblemSubsetSum::random(NUM_DECISIONS);
        assert!(problem.is_legal());
        let solver
            = MonteCarloTreeSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem);
        assert_eq!(solver.mcts_root.max_score, ZERO_SCORE);

        assert_eq!(
            solver.mcts_root.ucts_value( 0, solver.best_score() ),
            UCB_MAX
        );

        assert_eq!(
            MonteTreeNode::ucts_branch_ucb( &solver.mcts_root.true_branch, 0, solver.best_score() ),
            UCB_MAX
        );
        assert_eq!(
            MonteTreeNode::ucts_branch_ucb( &solver.mcts_root.false_branch, 0, solver.best_score() ),
            UCB_MAX
        );
    }

    #[test]
    fn test_best_first_solver_solver() {
        const NUM_DECISIONS: usize = 64; // for a start
        let problem = ProblemSubsetSum::random(NUM_DECISIONS);
        assert!(problem.is_legal());
        let mut solver =
            MonteCarloTreeSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem);
        assert!(solver.is_empty());

        let solution = solver.pop().expect("pop() should return Some(sol)");
        assert!(!solver.is_empty());
        // One subtree is none, and the other is not none, i.e. is some...
        assert!(solver.mcts_root.true_branch.is_none() || solver.mcts_root.false_branch.is_none());
        assert!(solver.mcts_root.true_branch.is_some() || solver.mcts_root.false_branch.is_some());

        assert!(problem.solution_is_complete(&solution));
        assert!(problem.solution_is_legal(&solution));

        solver.store_best_solution(solution);

        let solution2 = solver.pop().expect("pop() should return Some(sol)");
        assert!(!solver.is_empty());
        // One or both subtrees are not none, i.e. are some...
        assert!(solver.mcts_root.true_branch.is_some() || solver.mcts_root.false_branch.is_some());

        assert!(solver.problem.solution_is_complete(&solution2));
        assert!(solver.problem.solution_is_legal(&solution2));
    }

    #[test]
    fn test_find_best_first_solution() {
        const FEW_DECISIONS: usize = 4; // so we can be sure to find THE optimum!
        let knapsack = ProblemSubsetSum::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            MonteCarloTreeSolver::<MinimalSolution, ProblemSubsetSum>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        let the_best = solver
            .find_best_solution( & knapsack, time_limit)
            .expect("could not find best solution");

        assert!(solver.problem.solution_is_legal(&the_best));
        assert!(solver.problem.solution_is_complete(&the_best));
        assert_eq!(
            solver.problem.solution_score(&the_best),
            the_best.get_score()
        );
        assert!(the_best.get_score() <= solver.problem.capacity);

        let better = solver
            .find_best_solution(& knapsack, time_limit)
            .expect("could not find best solution");

        assert!(solver.problem.solution_is_legal(&better));
        assert!(solver.problem.solution_is_complete(&better));
        assert_eq!(solver.problem.solution_score(&the_best), better.get_score());
        assert!(better.get_score() <= solver.problem.capacity);

        assert!(solver.mcts_root.counter <= 2);
        assert!(the_best.get_score() <= solver.mcts_root.max_score);
        assert!(better.get_score() <= solver.mcts_root.max_score);
    }
}
