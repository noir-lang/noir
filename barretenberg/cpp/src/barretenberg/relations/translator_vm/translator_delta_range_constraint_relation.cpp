#include "barretenberg/relations/translator_vm/translator_delta_range_constraint_relation_impl.hpp"
#include "barretenberg/translator_vm/translator_flavor.hpp"
namespace bb {
template class TranslatorDeltaRangeConstraintRelationImpl<fr>;
DEFINE_SUMCHECK_RELATION_CLASS(TranslatorDeltaRangeConstraintRelationImpl, TranslatorFlavor);
} // namespace bb