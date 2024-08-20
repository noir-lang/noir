#include "barretenberg/relations/translator_vm/translator_extra_relations_impl.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/translator_vm_verifier/translator_recursive_flavor.hpp"

namespace bb {
template class TranslatorOpcodeConstraintRelationImpl<stdlib::field_t<UltraCircuitBuilder>>;
template class TranslatorAccumulatorTransferRelationImpl<stdlib::field_t<UltraCircuitBuilder>>;
template class TranslatorOpcodeConstraintRelationImpl<stdlib::field_t<MegaCircuitBuilder>>;
template class TranslatorAccumulatorTransferRelationImpl<stdlib::field_t<MegaCircuitBuilder>>;
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(TranslatorOpcodeConstraintRelationImpl,
                                        TranslatorRecursiveFlavor_<UltraCircuitBuilder>);
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(TranslatorOpcodeConstraintRelationImpl,
                                        TranslatorRecursiveFlavor_<MegaCircuitBuilder>);
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(TranslatorAccumulatorTransferRelationImpl,
                                        TranslatorRecursiveFlavor_<UltraCircuitBuilder>);
DEFINE_SUMCHECK_VERIFIER_RELATION_CLASS(TranslatorAccumulatorTransferRelationImpl,
                                        TranslatorRecursiveFlavor_<MegaCircuitBuilder>);

} // namespace bb