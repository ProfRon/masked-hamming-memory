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
#[derive(Default, Debug, Clone)]
pub struct MonteTreeNode {
    pub exhausted: bool,
    pub counter: usize,
    pub max_score: ScoreType,
    pub true_branch: Option<Box<MonteTreeNode>>,
    pub false_branch: Option<Box<MonteTreeNode>>,
}

type UcbType = f64;
#[allow(clippy::unnecessary_cast)]
const UCB_ZERO: UcbType = 0.0 as UcbType;
#[allow(clippy::unnecessary_cast)]
const UCB_MAX: UcbType = ScoreType::MAX as UcbType;
#[allow(clippy::unnecessary_cast)]
const UCB_C_P: UcbType = 2.828427125 as UcbType; // 2 x sqrt(2), subject to change, see Jupyterbook.

impl MonteTreeNode {
    #[inline]
    pub fn default() -> Self {
        Self {
            exhausted: false,
            max_score: ZERO_SCORE,
            counter: 0,
            true_branch: None,
            false_branch: None,
        }
    }

    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    // Here a synonym for building the root of the whole tree
    #[inline]
    pub fn root() -> Self {
        Self::new()
    }

    pub fn debug_dump_branch(branch: &Option<Box<MonteTreeNode>>, depth: usize) -> String {
        let mut indent = String::new();
        // indent
        for _ in 0..depth {
            indent.push_str("  ")
        }
        // Now build the result
        let mut result: String;
        // Print a line, possibly followed by subbranches
        match branch {
            None => {
                result = String::from("None");
            }
            Some(node) => {
                result = format!(
                    "ex {}, max {}, cntr {}\n",
                    node.exhausted, node.max_score, node.counter
                );
                indent.push_str("  ");
                result.push_str(&format!(
                    "{}{}{}\n",
                    indent,
                    "True :",
                    Self::debug_dump_branch(&node.true_branch, depth + 1)
                ));
                result.push_str(&format!(
                    "{}{}{}",
                    indent,
                    "False:",
                    Self::debug_dump_branch(&node.false_branch, depth + 1)
                ));
            }
        }; // end match
           // finished!
        result
    } // end debug_dump_node

    #[inline]
    pub fn debug_dump_node(&self) -> String {
        // self *should* only be cloned once... right?!? Not the whole tree?!?
        let opt = Some(Box::new(self.clone()));
        let mut result = String::from("Root:");
        result.push_str(&Self::debug_dump_branch(&opt, 0));
        result
    }

    #[inline]
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
           // next tree lines only necessary for root, but cheap...
        self.exhausted = false;
        self.max_score = ZERO_SCORE;
        self.counter = 0;
    }

    #[inline]
    pub fn ucts_value(&self, parent_counter: usize, high_score: ScoreType) -> UcbType {
        // avoid dividing by zero
        if self.exhausted {
            UCB_ZERO
        } else if 0 == self.counter {
            UCB_MAX
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

    #[inline]
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
        assert!(UCB_ZERO != true_subtree_ucb || UCB_ZERO != false_subtree_ucb);
        if UCB_ZERO == true_subtree_ucb {
            return false;
        };
        if UCB_ZERO == false_subtree_ucb {
            return true;
        };
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
            // if NOT full_monte, deterninistially take subtree with larger UCB
            // (but break ties randomly).
            if false_subtree_ucb < true_subtree_ucb {
                true
            } else if true_subtree_ucb < false_subtree_ucb {
                false
            } else {
                // when branches are equal, choose at random
                let coin_flip: bool = rand::thread_rng().gen();
                coin_flip
            } // end if equal
        }
    } // end best_ucb_branch

    ///////////////////////// GROW TREE ////////////////////////////////
    fn grow_tree<Sol: Solution, Prob: Problem<Sol = Sol>>(
        &mut self,
        problem: &Prob,
        solution: &mut Sol,
        full_monte: bool,
        high_score: ScoreType,
    ) -> ScoreType {
        assert!(problem.solution_is_legal(solution)); // !!!
        assert!(!self.exhausted); // logic above should make that impossible
        if problem.solution_is_complete(solution) {
            trace!(
                "Top of grow_tree, COMPLETE solution score {} (high score {})",
                solution.get_score(),
                high_score
            );
            // complete and legal
            self.exhausted = true;
            let new_score = problem.solution_score(solution);
            self.max_score = std::cmp::max(self.max_score, new_score);

            // we could call self.store_best_solution now already, but...
            // we won't need it until later!
            debug_assert!(problem.rules_audit_passed(solution));
            new_score
        } else {
            // end if solution is incomplete but legal and self NOT exhausted

            self.counter += 1;

            // decide on a branch!
            let decision = self.best_ucb_branch(full_monte, high_score);

            // Fix solution ... compare Problem::produce_children()
            debug_assert!(problem.solution_is_legal(solution));
            let index = problem
                .first_open_decision(solution)
                .expect("Should have an open decision");

            trace!(
                "Grow_tree: depth {}, counter = {}, solution score {} (high score {}) => {}",
                index,
                self.counter,
                solution.get_score(),
                high_score,
                decision
            );

            solution.make_decision(index, decision);
            debug_assert!(problem.solution_is_legal(solution));
            problem.apply_rules(solution);
            debug_assert!(problem.rules_audit_passed(solution));

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
                assert!(!boxed_node.exhausted); // if it was, we shouldn't be here...

                // BOUND:
                // we COULD call problem.could_be_better than, but we'd need access to the current
                // best solution.  We use high_score instead.
                let new_score: ScoreType;
                if solution.get_best_score() <= high_score
                    || problem.solution_is_complete(&solution)
                {
                    boxed_node.exhausted = true;
                    new_score = solution.get_score();
                } else {
                    // a new  best solution is possible, but solution is incomplete
                    // so...               Recursion!
                    new_score = boxed_node.grow_tree(problem, solution, full_monte, high_score);
                };

                self.max_score = std::cmp::max(self.max_score, new_score);

                // We don't have to update self.best_solution here -- we do that when this method
                // is finished (after unrolling all the recursion.

                // check for exhaustion
                //self.exhausted = match ( &self.true_branch, &self.false_branch ) {
                //    ( Some( true_box ), Some( false_box) ) => { true_box.exhausted && false_box.exhausted },
                //    _ => { self.exhausted }, // i.e. NOP, Do Nothing
                //};
                if let Some(true_box) = &self.true_branch {
                    if let Some(false_box) = &self.false_branch {
                        self.exhausted = true_box.exhausted && false_box.exhausted;
                    }; // end if unbox false branch
                }; // endif unbox true branch

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
    pub full_monte: bool,
    pub mcts_root: MonteTreeNode,
    pub best_solution: Sol,
    pub problem: Prob,
}

impl<Sol: Solution, Prob: Problem<Sol = Sol>> MonteCarloTreeSolver<Sol, Prob> {
    // a replacement for Self::new( size )
    #[inline]
    pub fn builder(problem: &Prob) -> Self {
        Self {
            full_monte: false, // until overwritten with true
            mcts_root: MonteTreeNode::root(),
            best_solution: problem.random_solution(),
            problem: problem.clone(), // = problem, note rust syntatic sugar
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
        "MonteCarloSolver "
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
    fn is_finished(&self) -> bool {
        self.mcts_root.exhausted
    }

    #[inline]
    fn clear(&mut self) {
        self.mcts_root.clear();
        let size = self.best_solution.size();
        self.best_solution = Sol::new(size);
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
            &self.problem,
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
        debug_assert_eq!(solution.get_score(), solution.get_best_score());
        // Occasionally, the following condition IS allowed (to be false)
        // debug_assert!(self.best_score() <= solution.get_score());
        self.best_solution = solution;
    }
} // end imp Solver for MonteCarloTreeSolver

/**************************************************************************************/
//////////////// TESTs for ProblemSubsetSum with  MonteCarloTreeSolver /////////////////
#[cfg(test)]
mod more_tests {
    use super::*;
    use implementations::*;
    use mhd_optimizer::{MinimalSolution, Problem, Solution, Solver};

    const NUM_DECISIONS: usize = 64; // for a start

    #[test]
    fn test_monte_tree() {
        let problem = ProblemSubsetSum::random(NUM_DECISIONS);
        assert!(problem.is_legal());
        let solver = MonteCarloTreeSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem);
        assert_eq!(solver.mcts_root.max_score, ZERO_SCORE);

        assert_eq!(solver.mcts_root.ucts_value(0, solver.best_score()), UCB_MAX);

        assert_eq!(
            MonteTreeNode::ucts_branch_ucb(&solver.mcts_root.true_branch, 0, solver.best_score()),
            UCB_MAX
        );
        assert_eq!(
            MonteTreeNode::ucts_branch_ucb(&solver.mcts_root.false_branch, 0, solver.best_score()),
            UCB_MAX
        );
    }

    #[test]
    fn test_mcts_solver() {
        const NUM_DECISIONS: usize = 8; // for a start
        let problem = ProblemSubsetSum::random(NUM_DECISIONS);
        assert!(problem.is_legal());
        let mut solver =
            MonteCarloTreeSolver::<MinimalSolution, ProblemSubsetSum>::builder(&problem);
        assert!(solver.is_empty());

        debug!("Start of test_mcts_solver, knapsack = {:?}", problem);

        let solution1 = solver.pop().expect("pop() should return Some(sol)");

        debug!(
            "Tree after 1st solver.pop:\n{}",
            solver.mcts_root.debug_dump_node()
        );

        assert!(!solver.is_empty());
        assert_eq!(solver.mcts_root.counter, 1);
        // One subtree is none, and the other is not none, i.e. is some...
        assert!(solver.mcts_root.true_branch.is_none() || solver.mcts_root.false_branch.is_none());
        assert!(solver.mcts_root.true_branch.is_some() || solver.mcts_root.false_branch.is_some());

        assert!(problem.rules_audit_passed(&solution1));

        if problem.solution_is_complete(&solution1) {
            solver.new_best_solution(&problem, solution1); // Warning: solution1 moved!
        } else {
            warn!(
                "First Solution returned is not complete? S1 = {:?}",
                solution1
            );
            warn!(
                "                      current best solution = {:?}",
                solver.best_solution()
            );
        };

        let solution2 = solver.pop().expect("pop() should return Some(sol)");

        debug!(
            "\nTree after 2nd solver.pop:\n{}",
            solver.mcts_root.debug_dump_node()
        );

        // assert!(!solver.is_empty());
        assert_eq!(solver.mcts_root.counter, 2);

        // One or both subtrees are not none, i.e. are some...
        assert!(solver.mcts_root.true_branch.is_some() || solver.mcts_root.false_branch.is_some());

        assert!(solver.problem.rules_audit_passed(&solution2));

        if problem.solution_is_complete(&solution2) {
            solver.new_best_solution(&problem, solution2); // Warning: solution1 moved!
        } else {
            warn!(
                "Second Solution returned is not complete? S1 = {:?}",
                solution2
            );
            warn!(
                "                      current best solution = {:?}",
                solver.best_solution()
            );
        };
    }

    #[test]
    fn test_mcts_find_solution() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!
        const MAX_COUNTER: usize = 1 << FEW_DECISIONS;

        let knapsack = ProblemSubsetSum::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            MonteCarloTreeSolver::<MinimalSolution, ProblemSubsetSum>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!("Start of test find_solution, knapsack = {:?}", knapsack);

        let the_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find best solution");

        debug!(
            "Tree after solver.find_solution:\n{}",
            solver.mcts_root.debug_dump_node()
        );

        assert!(solver.mcts_root.exhausted);
        assert!(solver.mcts_root.counter <= MAX_COUNTER);

        assert!(solver.problem.solution_is_legal(&the_best));
        assert!(solver.problem.solution_is_complete(&the_best));
        assert_eq!(
            solver.problem.solution_score(&the_best),
            the_best.get_score()
        );
        assert!(the_best.get_score() <= solver.problem.capacity);
    }

    #[test]
    fn test_mcts_find_01knapsack_solution() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!
        const MAX_COUNTER: usize = 1 << FEW_DECISIONS;

        let knapsack = Problem01Knapsack::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            MonteCarloTreeSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!("Start of test find_solution, knapsack = {:?}", knapsack);

        let the_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find best solution");

        debug!(
            "Tree after solver.find_solution:\n{}",
            solver.mcts_root.debug_dump_node()
        );

        assert!(solver.mcts_root.exhausted);
        assert!(solver.mcts_root.counter <= MAX_COUNTER);

        assert!(solver.problem.solution_is_legal(&the_best));
        assert!(solver.problem.solution_is_complete(&the_best));
        assert_eq!(
            solver.problem.solution_score(&the_best),
            the_best.get_score()
        );
    }

    #[test]
    fn test_mcts_solve_mutliple_knapsacks() {
        const FEW_DECISIONS: usize = 8; // so we can be sure to find THE optimum!
        const MAX_COUNTER: usize = 1 << FEW_DECISIONS;

        let knapsack = Problem01Knapsack::random(FEW_DECISIONS);
        assert!(knapsack.is_legal());
        let mut solver =
            MonteCarloTreeSolver::<ZeroOneKnapsackSolution, Problem01Knapsack>::builder(&knapsack);

        use std::time::Duration;
        let time_limit = Duration::new(1, 0); // 1 second

        debug!("Start of test find_solution, knapsack = {:?}", knapsack);

        let the_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find best solution");

        assert!(solver.problem.solution_is_legal(&the_best));
        assert!(solver.problem.solution_is_complete(&the_best));
        assert_eq!(
            solver.problem.solution_score(&the_best),
            the_best.get_score()
        );

        // Now test solver.clear()!!!
        solver.clear();
        assert!(solver.is_empty());
        assert_eq!(solver.mcts_root.counter, 0);
        assert!(!solver.mcts_root.exhausted);
        assert!(solver.mcts_root.true_branch.is_none());
        assert!(solver.mcts_root.false_branch.is_none());

        // Two birds with one stone -- we haven't tested the full monte yet!!!
        solver.full_monte = true;
        debug!("Tree after clear:\n{}", solver.mcts_root.debug_dump_node());

        let second_best = solver
            .find_best_solution(&knapsack, time_limit)
            .expect("could not find 2nd best solution");

        assert!(solver.mcts_root.exhausted);
        assert!(solver.mcts_root.counter <= MAX_COUNTER);

        assert!(solver.problem.solution_is_legal(&second_best));
        assert!(solver.problem.solution_is_complete(&second_best));
        assert_eq!(
            solver.problem.solution_score(&second_best),
            second_best.get_score()
        );
    }
}
