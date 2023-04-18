#include <cmath>
#include "barretenberg/common/throw_or_abort.hpp"
#include <cstddef>
#include <memory>
#include "barretenberg/honk/transcript/transcript.hpp"
#include "./verifier.hpp"
#include "barretenberg/plonk/proof_system/public_inputs/public_inputs.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/pcs/gemini/gemini.hpp"
#include "barretenberg/honk/pcs/kzg/kzg.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include "barretenberg/proof_system/polynomial_store/polynomial_store.hpp"
#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp"
#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include "barretenberg/proof_system/composer/permutation_helper.hpp"
#include <math.h>
#include <optional>
#include <string>
#include "barretenberg/honk/utils/power_polynomial.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_computation_relation.hpp"
#include "barretenberg/honk/sumcheck/relations/grand_product_initialization_relation.hpp"

using namespace barretenberg;
using namespace proof_system::honk::sumcheck;

namespace proof_system::honk {
template <typename program_settings>
Verifier<program_settings>::Verifier(std::shared_ptr<plonk::verification_key> verifier_key)
    : key(verifier_key)
{}

template <typename program_settings>
Verifier<program_settings>::Verifier(Verifier&& other)
    : key(other.key)
    , kate_verification_key(std::move(other.kate_verification_key))
{}

template <typename program_settings> Verifier<program_settings>& Verifier<program_settings>::operator=(Verifier&& other)
{
    key = other.key;
    kate_verification_key = (std::move(other.kate_verification_key));
    kate_g1_elements.clear();
    kate_fr_elements.clear();
    return *this;
}

/**
* @brief This function verifies a Honk proof for given program settings.
*
* @details A Standard Honk proof contains the following:
    Multilinear evaluations:
        w_i(X),        i = 1,2,3
        sigma_i(X),    i = 1,2,3
        q_i(X),        i = 1,2,3,4,5
        z_perm(X),
        L_0(X),
        id(X)

    Univariate evaluations:
        a_0 = Fold_{-r}^(0)(-r),
        a_l = Fold^(l)(-r^{2^l}), i = 1,...,d-1

    Univariate polynomials (evaluations over MAX_RELATION_LENGTH-many points):
        S_l, l = 0,...,d-1

    Commitments:
        [w_i]_1,        i = 1,2,3
        [z_perm]_1,
        [Fold^(l)]_1,   l = 1,...,d-1
        [Q]_1,
        [W]_1
*/
template <typename program_settings> bool Verifier<program_settings>::verify_proof(const plonk::proof& proof)
{
    using FF = typename program_settings::fr;
    using Commitment = barretenberg::g1::element;
    using CommitmentAffine = barretenberg::g1::affine_element;
    using Gemini = pcs::gemini::MultilinearReductionScheme<pcs::kzg::Params>;
    using Shplonk = pcs::shplonk::SingleBatchOpeningScheme<pcs::kzg::Params>;
    using KZG = pcs::kzg::UnivariateOpeningScheme<pcs::kzg::Params>;
    const size_t NUM_POLYNOMIALS = honk::StandardArithmetization::NUM_POLYNOMIALS;
    const size_t NUM_UNSHIFTED = honk::StandardArithmetization::NUM_UNSHIFTED_POLYNOMIALS;
    const size_t NUM_PRECOMPUTED = honk::StandardArithmetization::NUM_PRECOMPUTED_POLYNOMIALS;

    constexpr auto num_wires = program_settings::num_wires;

    transcript = VerifierTranscript<FF>{ proof.proof_data };

    // TODO(Adrian): Change the initialization of the transcript to take the VK hash?
    const auto circuit_size = transcript.template receive_from_prover<uint32_t>("circuit_size");
    const auto public_input_size = transcript.template receive_from_prover<uint32_t>("public_input_size");

    if (circuit_size != key->circuit_size) {
        return false;
    }
    if (public_input_size != key->num_public_inputs) {
        return false;
    }

    std::vector<FF> public_inputs;
    for (size_t i = 0; i < public_input_size; ++i) {
        auto public_input_i = transcript.template receive_from_prover<FF>("public_input_" + std::to_string(i));
        public_inputs.emplace_back(public_input_i);
    }

    // Get commitments to the wires
    std::array<CommitmentAffine, num_wires> wire_commitments;
    for (size_t i = 0; i < num_wires; ++i) {
        wire_commitments[i] = transcript.template receive_from_prover<CommitmentAffine>("W_" + std::to_string(i + 1));
    }

    // Get permutation challenges
    auto [beta, gamma] = transcript.get_challenges("beta", "gamma");

    const FF public_input_delta = compute_public_input_delta<FF>(public_inputs, beta, gamma, circuit_size);

    sumcheck::RelationParameters<FF> relation_parameters{
        .beta = beta,
        .gamma = gamma,
        .public_input_delta = public_input_delta,
    };

    // Get commitment to Z_PERM
    auto z_permutation_commitment = transcript.template receive_from_prover<CommitmentAffine>("Z_PERM");

    // // TODO(Cody): Compute some basic public polys like id(X), pow(X), and any required Lagrange polys

    // Execute Sumcheck Verifier
    auto sumcheck = Sumcheck<FF,
                             VerifierTranscript<FF>,
                             ArithmeticRelation,
                             GrandProductComputationRelation,
                             GrandProductInitializationRelation>(circuit_size, transcript);
    std::optional sumcheck_output = sumcheck.execute_verifier(relation_parameters);

    // If Sumcheck does not return an output, sumcheck verification has failed
    if (!sumcheck_output.has_value()) {
        return false;
    }

    auto [multivariate_challenge, multivariate_evaluations] = *sumcheck_output;

    // Execute Gemini/Shplonk verification:

    // Construct inputs for Gemini verifier:
    // - Multivariate opening point u = (u_0, ..., u_{d-1})
    // - batched unshifted and to-be-shifted polynomial commitments
    auto batched_commitment_unshifted = Commitment::zero();
    auto batched_commitment_to_be_shifted = Commitment::zero();

    // Compute powers of batching challenge rho
    Fr rho = transcript.get_challenge("rho");
    std::vector<Fr> rhos = Gemini::powers_of_rho(rho, NUM_POLYNOMIALS);

    // Compute batched multivariate evaluation
    Fr batched_evaluation = Fr::zero();
    for (size_t i = 0; i < NUM_POLYNOMIALS; ++i) {
        batched_evaluation += multivariate_evaluations[i] * rhos[i];
    }

    // Construct batched commitment for NON-shifted polynomials
    for (size_t i = 0; i < NUM_PRECOMPUTED; ++i) {
        auto commitment = key->commitments[honk::StandardArithmetization::ENUM_TO_COMM[i]];
        batched_commitment_unshifted += commitment * rhos[i];
    }
    // add wire commitments
    for (size_t i = 0; i < num_wires; ++i) {
        batched_commitment_unshifted += wire_commitments[i] * rhos[NUM_PRECOMPUTED + i];
    }
    // add z_permutation commitment
    batched_commitment_unshifted += z_permutation_commitment * rhos[NUM_PRECOMPUTED + num_wires];

    // Construct batched commitment for to-be-shifted polynomials
    batched_commitment_to_be_shifted = z_permutation_commitment * rhos[NUM_UNSHIFTED];

    // Produce a Gemini claim consisting of:
    // - d+1 commitments [Fold_{r}^(0)], [Fold_{-r}^(0)], and [Fold^(l)], l = 1:d-1
    // - d+1 evaluations a_0_pos, and a_l, l = 0:d-1
    auto gemini_claim = Gemini::reduce_verify(multivariate_challenge,
                                              batched_evaluation,
                                              batched_commitment_unshifted,
                                              batched_commitment_to_be_shifted,
                                              transcript);

    // Produce a Shplonk claim: commitment [Q] - [Q_z], evaluation zero (at random challenge z)
    auto shplonk_claim = Shplonk::reduce_verify(gemini_claim, transcript);

    // Aggregate inputs [Q] - [Q_z] and [W] into an 'accumulator' (can perform pairing check on result)
    auto kzg_claim = KZG::reduce_verify(shplonk_claim, transcript);

    // Return result of final pairing check
    return kzg_claim.verify(kate_verification_key);
}

template class Verifier<honk::standard_verifier_settings>;

} // namespace proof_system::honk
