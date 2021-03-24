/// ## The Solver Trait
///
use mhd_optimizer::Solution;

pub trait Solver<Sol: Solution> {
    // First, one "associated type"
    // type Sol = S;

    /// every instance of this struct should have a descriptive name (for tracing, debugging)
    /// TO DO: Remove this when <https://doc.rust-lang.org/std/any/fn.type_name_of_val.html> stable
    fn name(&self) -> &'static str;

    /// Every instance should have a SHORT description for Debugging,
    /// giving things like the number of solutions in the container, etc.
    fn short_description(&self) -> String;

    // Constructors

    /// Constructor for a "blank" solution (with no decisions made yet) where
    /// size is the number of decisions to be made (free variables to assign values to).
    fn new(size: usize) -> Self;

    // Methods used by the Unified Optimization Algorithm (identified above)

    /// Number of solutions stored in this container
    fn number_of_solutions(&self) -> usize;

    fn is_empty(&self) -> bool {
        0 == self.number_of_solutions()
    }

    /// Discard any solutions currently stored in container
    fn clear(&mut self); // empty out (if not already empty) like std::vec::clear()

    /// Add one incomplete solution to container -- the main difference between each implementation!
    /// This is a very important difference between the various implementations!
    fn push(&mut self, solution: Sol);

    /// Remove one solution from container (if possible)
    /// This is a very important difference between the various implementations!
    fn pop(&mut self) -> Option<Sol>;

} // end Solver Problem
