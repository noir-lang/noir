#include "barretenberg/relations/translator_vm/translator_decomposition_relation_impl.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/translator_vm_recursion/translator_recursive_flavor.hpp"

namespace bb {
template class TranslatorDecompositionRelationImpl<stdlib::field_t<UltraCircuitBuilder>>;
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(TranslatorDecompositionRelationImpl,
                                        TranslatorRecursiveFlavor_<UltraCircuitBuilder>);
} // namespace bb
