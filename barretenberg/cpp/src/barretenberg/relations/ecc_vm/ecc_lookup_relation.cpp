#include "barretenberg/flavor/ecc_vm.hpp"
#include "barretenberg/flavor/relation_definitions_fwd.hpp"
#include "barretenberg/honk/proof_system/lookup_library.hpp"
#include "ecc_msm_relation.hpp"

namespace proof_system::honk::sumcheck {

/**
 * @brief Expression for ECCVM lookup tables.
 * @details We use log-derivative lookup tables for the following case:
 * Table writes: ECCVMPointTable columns: we define Straus point table:
 * { {0, -15[P]}, {1, -13[P]}, ..., {15, 15[P]} }
 * write source: { precompute_round, precompute_tx, precompute_ty }
 * Table reads: ECCVMMSM columns. Each row adds up to 4 points into MSM accumulator
 * read source: { msm_slice1, msm_x1, msm_y1 }, ..., { msm_slice4, msm_x4, msm_y4 }
 * @param evals transformed to `evals + C(in(X)...)*scaling_factor`
 * @param in an std::array containing the fully extended Accumulator edges.
 * @param parameters contains beta, gamma, and public_input_delta, ....
 * @param scaling_factor optional term to scale the evaluation before adding to evals.
 */
template <typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void ECCVMLookupRelationBase<FF>::accumulate(ContainerOverSubrelations& accumulator,
                                             const AllEntities& in,
                                             const Parameters& params,
                                             [[maybe_unused]] const FF& scaling_factor)
{
    lookup_library::accumulate_logderivative_lookup_subrelation_contributions<FF, ECCVMLookupRelationBase<FF>>(
        accumulator, in, params, scaling_factor);
}

template class ECCVMLookupRelationBase<barretenberg::fr>;
template class ECCVMLookupRelationBase<grumpkin::fr>;
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationBase, flavor::ECCVM);
DEFINE_SUMCHECK_RELATION_CLASS(ECCVMLookupRelationBase, flavor::ECCVMGrumpkin);

} // namespace proof_system::honk::sumcheck
