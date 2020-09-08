use super::*;
use noir_evaluator::Directive;

pub struct DirectiveSolver {}

impl DirectiveSolver {
    pub fn solve<'a>(
        initial_witness: &mut BTreeMap<Witness, FieldElement>,
        gate: &'a Directive,
    ) -> Option<&'a Directive> {
        // Steps to solve a directive

        // We need a compiler which can read the AST and evaluate FieldElements, for loops, and if statements
        // For now, just field operations should be fine
        // What we essentially have is two languages with the same syntax, but different compiling phases.

        // At this stage, we should have the necessary witness values in the Map!

        todo!()
    }
}
