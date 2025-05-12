use std::collections::BTreeSet;

use acir::FieldElement;
use noirc_evaluator::ssa::ssa_gen::Ssa;

pub(crate) fn build_dictionary_from_ssa(_ssa: &Ssa) -> BTreeSet<FieldElement> {
    // TODO(#8467): Traverse the SSA to collect fields.
    BTreeSet::new()
}
