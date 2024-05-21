#include "barretenberg/relations/translator_vm/translator_extra_relations_impl.hpp"
#include "barretenberg/translator_vm/translator_flavor.hpp"
namespace bb {
template class TranslatorOpcodeConstraintRelationImpl<fr>;
template class TranslatorAccumulatorTransferRelationImpl<fr>;

DEFINE_SUMCHECK_RELATION_CLASS(TranslatorOpcodeConstraintRelationImpl, TranslatorFlavor);
DEFINE_SUMCHECK_RELATION_CLASS(TranslatorAccumulatorTransferRelationImpl, TranslatorFlavor);
} // namespace bb