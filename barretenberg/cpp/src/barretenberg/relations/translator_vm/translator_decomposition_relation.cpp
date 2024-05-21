#include "barretenberg/relations/translator_vm/translator_decomposition_relation_impl.hpp"
#include "barretenberg/translator_vm/translator_flavor.hpp"
namespace bb {
template class TranslatorDecompositionRelationImpl<fr>;
DEFINE_SUMCHECK_RELATION_CLASS(TranslatorDecompositionRelationImpl, TranslatorFlavor);
} // namespace bb