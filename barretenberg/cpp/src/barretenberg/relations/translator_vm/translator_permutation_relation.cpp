#include "barretenberg/relations/translator_vm/translator_permutation_relation_impl.hpp"
#include "barretenberg/translator_vm/translator_flavor.hpp"
namespace bb {
template class TranslatorPermutationRelationImpl<fr>;
DEFINE_SUMCHECK_RELATION_CLASS(TranslatorPermutationRelationImpl, TranslatorFlavor);
} // namespace bb