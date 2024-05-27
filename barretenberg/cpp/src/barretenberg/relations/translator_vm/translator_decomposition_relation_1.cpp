#include "barretenberg/relations/translator_vm/translator_decomposition_relation_impl.hpp"
#include "barretenberg/translator_vm/translator_flavor.hpp"
namespace bb {
// Split up due to compile time, used to use DEFINE_SUMCHECK_RELATION_CLASS
template class TranslatorDecompositionRelationImpl<fr>;
template void TranslatorDecompositionRelationImpl<TranslatorFlavor::FF>::accumulate<
    bb::Relation<TranslatorDecompositionRelationImpl<TranslatorFlavor::FF>>::SumcheckArrayOfValuesOverSubrelations,
    TranslatorFlavor::AllEntities<TranslatorFlavor::FF>>(
    bb::Relation<TranslatorDecompositionRelationImpl<TranslatorFlavor::FF>>::SumcheckArrayOfValuesOverSubrelations&,
    TranslatorFlavor::AllEntities<TranslatorFlavor::FF> const&,
    RelationParameters<TranslatorFlavor::FF> const&,
    TranslatorFlavor::FF const&);
} // namespace bb