#include "barretenberg/relations/translator_vm/translator_permutation_relation_impl.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/translator_vm_recursion/translator_recursive_flavor.hpp"

namespace bb {
template class TranslatorPermutationRelationImpl<stdlib::field_t<UltraCircuitBuilder>>;
template class TranslatorPermutationRelationImpl<stdlib::field_t<MegaCircuitBuilder>>;
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(TranslatorPermutationRelationImpl,
                                        TranslatorRecursiveFlavor_<UltraCircuitBuilder>);
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(TranslatorPermutationRelationImpl,
                                        TranslatorRecursiveFlavor_<MegaCircuitBuilder>);
} // namespace bb