/// ## The Problem Trait
///
use ::mhd_optimizer::Solution;
use ::mhd_optimizer::Solver;

pub trait Problem< Sol : Solution >
    where Self: Sized
{

    // First, three "associated type"
    // type Sol       : Solution;
    // type Slvr      : Solver< Self:Sol >;

    // type ScoreType = Sol::ScoreType;

    // every instance of this struct should have a descriptive name (for tracing, debugging)
    // TODO: Remove this when <https://doc.rust-lang.org/std/any/fn.type_name_of_val.html> stable

    fn name ( & self )  -> &'static str;
    // Constructors

    // size is the number of decisions to be made (free variables to assign values to).
    // `new` creates a default ("zero") instance of the problem.
    // `random` creates a full-fledged
    fn new(     size: usize ) -> Self;
    fn random( size : usize ) -> Self {
        let mut result = Self::new( size );
        result.randomize();
        result
    }

    // Utilities

    // See note on size, above (size is the number of decisions to be made).
    fn problem_size( & self ) -> usize;
    fn randomize( &mut self );

    // Note: is_leagal tests whether a problem -- not whether a solution -- is legal
    // (the Solution trait has its own is_legal method).
    // We're testing if an instance of a PROBLEM is correct, solvable and non-trivial.
    fn is_legal( & self ) -> bool;

    // Solution attributest that only the problem can evaluate
    fn solution_score(      & self, solution : & Sol ) -> Sol::ScoreType;
    fn solution_best_score( & self, solution : & Sol ) -> Sol::ScoreType; // "upper bound"

    fn solution_is_legal(    & self, solution : & Sol ) -> bool;
    fn solution_is_complete( & self, solution : & Sol ) -> bool;

    // Methods used by the Unified Optimization Algorithm (identified above)

    fn random_solution(   & self ) -> Sol;
    fn starting_solution( & self ) -> Sol;

    fn better_than(        & self, new_solution : & Sol, old_solution : & Sol ) -> bool {
        self.solution_score( old_solution ) < self.solution_score( new_solution )
    }
    fn can_be_better_than( & self, new_solution : & Sol, old_solution : & Sol ) -> bool {
        self.solution_best_score( old_solution ) <= self.solution_best_score( new_solution )
    }
    fn register_children_of( & self, parent : & Sol, solver : & mut impl Solver< Sol > );

} // end trait Problem
