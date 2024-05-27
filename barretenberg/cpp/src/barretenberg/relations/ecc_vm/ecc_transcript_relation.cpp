#include <array>
#include <tuple>

#include "./ecc_transcript_relation_impl.hpp"
#include "barretenberg/eccvm/eccvm_flavor.hpp"
#include "barretenberg/flavor/relation_definitions.hpp"

namespace bb {
template class ECCVMTranscriptRelationImpl<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMTranscriptRelationImpl, ECCVMFlavor);
} // namespace bb
