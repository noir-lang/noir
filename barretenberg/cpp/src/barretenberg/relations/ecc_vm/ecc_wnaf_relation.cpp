#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "ecc_wnaf_relation_impl.hpp"

namespace bb {

template class ECCVMWnafRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMWnafRelationImpl, ECCVMFlavor);

} // namespace bb
