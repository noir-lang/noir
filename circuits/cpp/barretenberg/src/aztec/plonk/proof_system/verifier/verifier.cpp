#include "./verifier.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include "../utils/kate_verification.hpp"
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <polynomials/polynomial_arithmetic.hpp>

using namespace barretenberg;

namespace waffle {
template <typename program_settings>
VerifierBase<program_settings>::VerifierBase(std::shared_ptr<verification_key> verifier_key,
                                             const transcript::Manifest& input_manifest)
    : manifest(input_manifest)
    , key(verifier_key)
{}

template <typename program_settings>
VerifierBase<program_settings>::VerifierBase(VerifierBase&& other)
    : manifest(other.manifest)
    , key(other.key)
{}

template <typename program_settings>
VerifierBase<program_settings>& VerifierBase<program_settings>::operator=(VerifierBase&& other)
{
    key = other.key;
    manifest = other.manifest;
    kate_g1_elements.clear();
    kate_fr_elements.clear();
    return *this;
}

template <typename program_settings> bool VerifierBase<program_settings>::validate_commitments()
{
    // TODO
    return true;
}

template <typename program_settings> bool VerifierBase<program_settings>::validate_scalars()
{
    // TODO
    return true;
}

template <typename program_settings> bool VerifierBase<program_settings>::verify_proof(const waffle::plonk_proof& proof)
{
    key->program_width = program_settings::program_width;
    transcript::StandardTranscript transcript = transcript::StandardTranscript(
        proof.proof_data, manifest, program_settings::hash_type, program_settings::num_challenge_bytes);
    g1::affine_element PI_Z = g1::affine_element::serialize_from_buffer(&transcript.get_element("PI_Z")[0]);
    g1::affine_element PI_Z_OMEGA = g1::affine_element::serialize_from_buffer(&transcript.get_element("PI_Z_OMEGA")[0]);

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
    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("eta");
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("alpha");
    transcript.apply_fiat_shamir("z");

    const auto alpha = fr::serialize_from_buffer(transcript.get_challenge("alpha").begin());
    const auto zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    const auto lagrange_evals = barretenberg::polynomial_arithmetic::get_lagrange_evaluations(zeta, key->domain);

    key->z_pow_n = zeta;
    for (size_t i = 0; i < key->domain.log2_size; ++i) {
        key->z_pow_n *= key->z_pow_n;
    }
    fr t_eval(0);
    program_settings::compute_quotient_evaluation_contribution(key.get(), alpha, transcript, t_eval);
    t_eval *= lagrange_evals.vanishing_poly.invert();
    transcript.add_element("t", t_eval.to_buffer());

    transcript.apply_fiat_shamir("nu");
    transcript.apply_fiat_shamir("separator");
    const auto separator_challenge = fr::serialize_from_buffer(transcript.get_challenge("separator").begin());

    fr batch_evaluation =
        compute_kate_batch_evaluation<fr, transcript::StandardTranscript, program_settings>(key.get(), transcript);

    kate_g1_elements.insert({ "BATCH_EVALUATION", g1::affine_one });
    kate_fr_elements.insert({ "BATCH_EVALUATION", -batch_evaluation });

    kate_g1_elements.insert({ "PI_Z_OMEGA", PI_Z_OMEGA });
    kate_fr_elements.insert({ "PI_Z_OMEGA", zeta * key->domain.root * separator_challenge });

    kate_g1_elements.insert({ "PI_Z", PI_Z });
    kate_fr_elements.insert({ "PI_Z", zeta });

    populate_kate_element_map<fr, g1::affine_element, transcript::StandardTranscript, program_settings>(
        key.get(), transcript, kate_g1_elements, kate_fr_elements);

    program_settings::append_scalar_multiplication_inputs(key.get(), alpha, transcript, kate_fr_elements);

    validate_commitments();
    validate_scalars();

    std::vector<fr> scalars;
    std::vector<g1::affine_element> elements;

    for (const auto& [key, value] : kate_g1_elements) {
        if (value.on_curve()) {
            scalars.emplace_back(kate_fr_elements.at(key));
            elements.emplace_back(value);
        }
    }


    size_t num_elements = elements.size();
    elements.resize(num_elements * 2);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(&elements[0], &elements[0], num_elements);
    scalar_multiplication::pippenger_runtime_state state(num_elements);

    g1::element P[2];

    P[0] = barretenberg::scalar_multiplication::pippenger(&scalars[0], &elements[0], num_elements, state);
    P[1] = -(g1::element(PI_Z_OMEGA) * separator_challenge + PI_Z);

    if (key->contains_recursive_proof) {
        ASSERT(key->recursive_proof_public_input_indices.size() == 16);
        const auto& inputs = transcript.get_field_element_vector("public_inputs");

        const auto recover_fq_from_public_inputs =
            [&inputs](const size_t idx0, const size_t idx1, const size_t idx2, const size_t idx3) {
                const uint256_t l0 = inputs[idx0];
                const uint256_t l1 = inputs[idx1];
                const uint256_t l2 = inputs[idx2];
                const uint256_t l3 = inputs[idx3];

                const uint256_t limb = l0 + (l1 << 68) + (l2 << 136) + (l3 << 204);
                return barretenberg::fq(limb);
            };

        const auto recursion_separator_challenge = transcript.get_challenge_field_element("separator").sqr();

        const auto x0 = recover_fq_from_public_inputs(key->recursive_proof_public_input_indices[0],
                                                      key->recursive_proof_public_input_indices[1],
                                                      key->recursive_proof_public_input_indices[2],
                                                      key->recursive_proof_public_input_indices[3]);
        const auto y0 = recover_fq_from_public_inputs(key->recursive_proof_public_input_indices[4],
                                                      key->recursive_proof_public_input_indices[5],
                                                      key->recursive_proof_public_input_indices[6],
                                                      key->recursive_proof_public_input_indices[7]);
        const auto x1 = recover_fq_from_public_inputs(key->recursive_proof_public_input_indices[8],
                                                      key->recursive_proof_public_input_indices[9],
                                                      key->recursive_proof_public_input_indices[10],
                                                      key->recursive_proof_public_input_indices[11]);
        const auto y1 = recover_fq_from_public_inputs(key->recursive_proof_public_input_indices[12],
                                                      key->recursive_proof_public_input_indices[13],
                                                      key->recursive_proof_public_input_indices[14],
                                                      key->recursive_proof_public_input_indices[15]);
        P[0] += g1::element(x0, y0, 1) * recursion_separator_challenge;
        P[1] += g1::element(x1, y1, 1) * recursion_separator_challenge;
    }

    g1::element::batch_normalize(P, 2);

    g1::affine_element P_affine[2]{
        { P[0].x, P[0].y },
        { P[1].x, P[1].y },
    };

    barretenberg::fq12 result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
        P_affine, key->reference_string->get_precomputed_g2_lines(), 2);

    return (result == barretenberg::fq12::one());
}

template class VerifierBase<unrolled_standard_verifier_settings>;
template class VerifierBase<unrolled_turbo_verifier_settings>;
template class VerifierBase<unrolled_plookup_verifier_settings>;
template class VerifierBase<standard_verifier_settings>;
template class VerifierBase<mimc_verifier_settings>;
template class VerifierBase<turbo_verifier_settings>;
template class VerifierBase<plookup_verifier_settings>;
template class VerifierBase<generalized_permutation_verifier_settings>;

} // namespace waffle