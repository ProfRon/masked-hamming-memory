/// ## The Solver Trait
///
use mhd_optimizer::Solution;

pub trait Solver<Sol: Solution> {
    // First, one "associated type"
    // type Sol = S;

    // every instance of this struct should have a descriptive name (for tracing, debugging)
    // TO DO: Remove this when <https://doc.rust-lang.org/std/any/fn.type_name_of_val.html> stable
    fn name(&self) -> &'static str;

    // Constructors

    fn new(size: usize) -> Self;

    // Methods used by the Unified Optimization Algorithm (identified above)

    fn number_of_solutions(&self) -> usize;
    fn is_empty(&self) -> bool {
        0 == self.number_of_solutions()
    }
    fn clear(&mut self); // empty out (if not already empty) like std::vec::clear()

    fn push(&mut self, solution: Sol);
    fn pop(&mut self) -> Option<Sol>;
} // end Solver Problem
