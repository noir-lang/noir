#include "./ultra_verifier.hpp"
#include "barretenberg/commitment_schemes/zeromorph/zeromorph.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/transcript/transcript.hpp"
#include "barretenberg/ultra_honk/oink_verifier.hpp"

namespace bb {

/**
 * @brief This function verifies an Ultra Honk proof for a given Flavor.
 *
 */
template <typename Flavor> bool UltraVerifier_<Flavor>::verify_proof(const HonkProof& proof)
{
    using FF = typename Flavor::FF;

    transcript = std::make_shared<Transcript>(proof);
    OinkVerifier<Flavor> oink_verifier{ instance->verification_key, transcript };
    auto [relation_parameters, witness_commitments, public_inputs, alphas] = oink_verifier.verify();
    instance->relation_parameters = std::move(relation_parameters);
    instance->witness_commitments = std::move(witness_commitments);
    instance->alphas = std::move(alphas);

    for (size_t idx = 0; idx < CONST_PROOF_SIZE_LOG_N; idx++) {
        instance->gate_challenges.emplace_back(
            transcript->template get_challenge<FF>("Sumcheck:gate_challenge_" + std::to_string(idx)));
    }

    DeciderVerifier decider_verifier{ instance, transcript };

    return decider_verifier.verify();
}

template class UltraVerifier_<UltraFlavor>;
template class UltraVerifier_<UltraKeccakFlavor>;
template class UltraVerifier_<MegaFlavor>;

} // namespace bb
