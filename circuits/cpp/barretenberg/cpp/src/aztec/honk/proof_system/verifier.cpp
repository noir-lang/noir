#include <cmath>
#include <common/throw_or_abort.hpp>
#include <cstddef>
#include <memory>
#include <plonk/proof_system/constants.hpp>
#include "./verifier.hpp"
#include "../../plonk/proof_system/public_inputs/public_inputs.hpp"
#include "ecc/curves/bn254/fr.hpp"
#include "honk/pcs/commitment_key.hpp"
#include "honk/pcs/gemini/gemini.hpp"
#include "honk/pcs/kzg/kzg.hpp"
#include "numeric/bitop/get_msb.hpp"
#include "proof_system/flavor/flavor.hpp"
#include "proof_system/polynomial_cache/polynomial_cache.hpp"
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <honk/composer/composer_helper/permutation_helper.hpp>
#include <math.h>
#include <string>
#include <honk/utils/power_polynomial.hpp>
#include <honk/sumcheck/relations/grand_product_computation_relation.hpp>
#include <honk/sumcheck/relations/grand_product_initialization_relation.hpp>

using namespace barretenberg;
using namespace honk::sumcheck;

namespace honk {
template <typename program_settings>
Verifier<program_settings>::Verifier(std::shared_ptr<bonk::verification_key> verifier_key,
                                     const transcript::Manifest& input_manifest)
    : manifest(input_manifest)
    , key(verifier_key)
{}

template <typename program_settings>
Verifier<program_settings>::Verifier(Verifier&& other)
    : manifest(other.manifest)
    , key(other.key)
    , kate_verification_key(std::move(other.kate_verification_key))
{}

template <typename program_settings> Verifier<program_settings>& Verifier<program_settings>::operator=(Verifier&& other)
{
    key = other.key;
    manifest = other.manifest;
    kate_verification_key = (std::move(other.kate_verification_key));
    kate_g1_elements.clear();
    kate_fr_elements.clear();
    return *this;
}

/**
* @brief This function verifies a Honk proof for given program settings.
*
* TODO(luke): Complete this description
* @detail A Standard Honk proof contains the following:
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
    using Commitment = barretenberg::g1::affine_element;
    using Transcript = typename program_settings::Transcript;
    using Gemini = pcs::gemini::MultilinearReductionScheme<pcs::kzg::Params>;
    using Shplonk = pcs::shplonk::SingleBatchOpeningScheme<pcs::kzg::Params>;
    using KZG = pcs::kzg::UnivariateOpeningScheme<pcs::kzg::Params>;
    using MLEOpeningClaim = pcs::MLEOpeningClaim<pcs::kzg::Params>;
    using GeminiProof = pcs::gemini::Proof<pcs::kzg::Params>;
    using POLYNOMIAL = bonk::StandardArithmetization::POLYNOMIAL;
    const size_t NUM_UNSHIFTED = bonk::StandardArithmetization::NUM_UNSHIFTED_POLYNOMIALS;
    const size_t NUM_PRECOMPUTED = bonk::StandardArithmetization::NUM_PRECOMPUTED_POLYNOMIALS;

    key->program_width = program_settings::program_width;

    size_t log_n(numeric::get_msb(key->circuit_size));

    // Add the proof data to the transcript, according to the manifest. Also initialise the transcript's hash type
    // and challenge bytes.
    auto transcript = transcript::StandardTranscript(
        proof.proof_data, manifest, program_settings::hash_type, program_settings::num_challenge_bytes);

    // Add the circuit size and the number of public inputs) to the transcript.
    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(key->circuit_size >> 24),
                             static_cast<uint8_t>(key->circuit_size >> 16),
                             static_cast<uint8_t>(key->circuit_size >> 8),
                             static_cast<uint8_t>(key->circuit_size) });

    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs >> 24),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs) });

    // Compute challenges from the proof data, based on the manifest, using the Fiat-Shamir heuristic
    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("eta");
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("alpha");
    for (size_t idx = 0; idx < log_n; idx++) {
        transcript.apply_fiat_shamir("u_" + std::to_string(log_n - idx));
    }
    transcript.apply_fiat_shamir("rho");
    transcript.apply_fiat_shamir("r");
    transcript.apply_fiat_shamir("nu");
    transcript.apply_fiat_shamir("z");
    transcript.apply_fiat_shamir("separator");

    // // TODO(Cody): Compute some basic public polys like id(X), pow(X), and any required Lagrange polys

    // Execute Sumcheck Verifier
    auto sumcheck = Sumcheck<FF,
                             Transcript,
                             ArithmeticRelation,
                             GrandProductComputationRelation,
                             GrandProductInitializationRelation>(transcript);
    bool sumcheck_result = sumcheck.execute_verifier();

    // Execute Gemini/Shplonk verification:

    // Construct inputs for Gemini verifier:
    // - Multivariate opening point u = (u_1, ..., u_d)
    // - MLE opening claim = {commitment, eval} for each multivariate and shifted multivariate polynomial
    std::vector<FF> opening_point;
    std::vector<MLEOpeningClaim> opening_claims;
    std::vector<MLEOpeningClaim> opening_claims_shifted;

    // Construct MLE opening point
    // Note: for consistency the evaluation point must be constructed as u = (u_d,...,u_1)
    for (size_t round_idx = 0; round_idx < key->log_circuit_size; round_idx++) {
        std::string label = "u_" + std::to_string(key->log_circuit_size - round_idx);
        opening_point.emplace_back(transcript.get_challenge_field_element(label));
    }

    // Get vector of multivariate evaluations produced by Sumcheck
    auto multivariate_evaluations = transcript.get_field_element_vector("multivariate_evaluations");

    // Construct NON-shifted opening claims
    for (size_t i = 0; i < NUM_UNSHIFTED; ++i) {
        if (i < NUM_PRECOMPUTED) { // if precomputed, commitment comes from verification key
            Commitment commitment = key->commitments[bonk::StandardArithmetization::ENUM_TO_COMM[i]];
            opening_claims.emplace_back(commitment, multivariate_evaluations[i]);
        } else { // if witness, commitment comes from prover (via transcript)
            Commitment commitment = transcript.get_group_element(bonk::StandardArithmetization::ENUM_TO_COMM[i]);
            opening_claims.emplace_back(commitment, multivariate_evaluations[i]);
        }
    }

    // Constructed shifted opening claims
    Commitment commitment = transcript.get_group_element("Z_PERM");
    Fr evaluation = multivariate_evaluations[POLYNOMIAL::Z_PERM_SHIFT];
    opening_claims_shifted.emplace_back(commitment, evaluation);

    // Reconstruct the Gemini Proof from the transcript
    GeminiProof gemini_proof;

    for (size_t i = 1; i < key->log_circuit_size; i++) {
        std::string label = "FOLD_" + std::to_string(i);
        gemini_proof.commitments.emplace_back(transcript.get_group_element(label));
    };

    for (size_t i = 0; i < key->log_circuit_size; i++) {
        std::string label = "a_" + std::to_string(i);
        gemini_proof.evals.emplace_back(transcript.get_field_element(label));
    };

    // Produce a Gemini claim consisting of:
    // - d+1 commitments [Fold_{r}^(0)], [Fold_{-r}^(0)], and [Fold^(l)], l = 1:d-1
    // - d+1 evaluations a_0_pos, and a_l, l = 0:d-1
    auto gemini_claim =
        Gemini::reduce_verify(opening_point, opening_claims, opening_claims_shifted, gemini_proof, &transcript);

    // Reconstruct the Shplonk Proof (commitment [Q]) from the transcript
    auto shplonk_proof = transcript.get_group_element("Q");

    // Produce a Shplonk claim: commitment [Q] - [Q_z], evaluation zero (at random challenge z)
    auto shplonk_claim = Shplonk::reduce_verify(gemini_claim, shplonk_proof, &transcript);

    // Reconstruct the KZG Proof (commitment [W]_1) from the transcript
    auto kzg_proof = transcript.get_group_element("W");

    // Aggregate inputs [Q] - [Q_z] and [W] into an 'accumulator' (can perform pairing check on result)
    auto kzg_claim = KZG::reduce_verify(shplonk_claim, kzg_proof);

    // Do final pairing check
    bool pairing_result = kzg_claim.verify(kate_verification_key.get());

    bool result = sumcheck_result && pairing_result;

    return result;
}

template class Verifier<honk::standard_verifier_settings>;

} // namespace honk