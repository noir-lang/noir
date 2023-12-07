#include "generic_permutation_relation.hpp"
#include "barretenberg/flavor/relation_definitions_fwd.hpp"
#include "barretenberg/flavor/toy_avm.hpp"
#include "barretenberg/honk/proof_system/logderivative_library.hpp"
#include "relation_definer.hpp"

namespace proof_system::honk::sumcheck {

/**
 * @brief Expression for generic log-derivative-based set permutation.
 * @param accumulator transformed to `evals + C(in(X)...)*scaling_factor`
 * @param in an std::array containing the fully extended Accumulator edges.
 * @param relation_params contains beta, gamma, and public_input_delta, ....
 * @param scaling_factor optional term to scale the evaluation before adding to evals.
 */
template <typename Settings, typename FF>
template <typename ContainerOverSubrelations, typename AllEntities, typename Parameters>
void GenericPermutationRelationImpl<Settings, FF>::accumulate(ContainerOverSubrelations& accumulator,
                                                              const AllEntities& in,
                                                              const Parameters& params,
                                                              const FF& scaling_factor)
{
    logderivative_library::accumulate_logderivative_permutation_subrelation_contributions<
        FF,
        GenericPermutationRelationImpl<Settings, FF>>(accumulator, in, params, scaling_factor);
}

// template class GenericPermutationRelationImpl<ExamplePermutationSettings, barretenberg::fr>;
// template <typename FF_>
// using GenericPermutationRelationExampleSettingsImpl = GenericPermutationRelationImpl<ExamplePermutationSettings,
// FF_>; DEFINE_SUMCHECK_RELATION_CLASS(GenericPermutationRelationExampleSettingsImpl, flavor::AVMTemplate);

DEFINE_IMPLEMENTATIONS_FOR_ALL_SETTINGS(GenericPermutationRelationImpl, flavor::ToyAVM);
} // namespace proof_system::honk::sumcheck
