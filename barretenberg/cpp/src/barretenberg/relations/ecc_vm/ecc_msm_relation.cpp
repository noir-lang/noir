#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"
#include "ecc_msm_relation_impl.hpp"

namespace bb {

template class ECCVMMSMRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMMSMRelationImpl, ECCVMFlavor);

} // namespace bb
