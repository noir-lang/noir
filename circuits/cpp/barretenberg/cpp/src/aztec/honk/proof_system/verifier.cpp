#include <cmath>
#include <common/throw_or_abort.hpp>
#include <plonk/proof_system/constants.hpp>
#include "./verifier.hpp"
#include "../../plonk/proof_system/public_inputs/public_inputs.hpp"
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>
#include <math.h>

#pragma GCC diagnostic ignored "-Wunused-variable"

using namespace barretenberg;
using namespace honk::sumcheck;

namespace honk {
template <typename program_settings>
Verifier<program_settings>::Verifier(std::shared_ptr<waffle::verification_key> verifier_key,
                                     const transcript::Manifest& input_manifest)
    : manifest(input_manifest)
    , key(verifier_key)
{}

template <typename program_settings>
Verifier<program_settings>::Verifier(Verifier&& other)
    : manifest(other.manifest)
    , key(other.key)
    , commitment_scheme(std::move(other.commitment_scheme))
{}

template <typename program_settings> Verifier<program_settings>& Verifier<program_settings>::operator=(Verifier&& other)
{
    key = other.key;
    manifest = other.manifest;
    commitment_scheme = (std::move(other.commitment_scheme));
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
template <typename program_settings> bool Verifier<program_settings>::verify_proof(const waffle::plonk_proof& proof)
{
    // TODO(luke): TBD how 'd' gets set here and elsewhere
    const size_t multivariate_d(1);

    const size_t num_polys = program_settings::num_polys;
    using FF = typename program_settings::fr;
    using Transcript = typename program_settings::Transcript;
    using Multivariates = Multivariates<FF, num_polys>;

    key->program_width = program_settings::program_width;

    // Add the proof data to the transcript, according to the manifest. Also initialise the transcript's hash type
    // and challenge bytes.
    auto transcript = transcript::StandardTranscript(
        proof.proof_data, manifest, program_settings::hash_type, program_settings::num_challenge_bytes);

    // Add the circuit size and the number of public inputs) to the transcript.
    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(key->n >> 24),
                             static_cast<uint8_t>(key->n >> 16),
                             static_cast<uint8_t>(key->n >> 8),
                             static_cast<uint8_t>(key->n) });

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

    // Compute some basic public polys like id(X), pow(X), and any required Lagrange polys

    // Execute Sumcheck Verifier
    auto sumcheck = Sumcheck<Multivariates, Transcript, ArithmeticRelation>(transcript);
    // sumcheck.execute_verifier(); // Need to mock prover in tests for this to run

    // Execute Gemini/Shplonk verification:
    // Gemini (reduce_verify()): Compute [Fold_{r}^(0)]_1, [Fold_{-r}^(0)]_1, Fold_{r}^(0)(r)
    // Shplonk (reduce_verify()): Compute simulated [Q_z]_1

    // TODO: Do final pairing check
    barretenberg::fq12 result = barretenberg::fq12::one();

    return (result == barretenberg::fq12::one());
}

template class Verifier<honk::standard_verifier_settings>;

} // namespace honk