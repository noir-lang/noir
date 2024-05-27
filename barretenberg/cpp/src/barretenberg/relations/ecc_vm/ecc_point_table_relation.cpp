#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "ecc_point_table_relation_impl.hpp"

namespace bb {
template class ECCVMPointTableRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMPointTableRelationImpl, ECCVMFlavor);

} // namespace bb
