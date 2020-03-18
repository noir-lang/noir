#include "./verifier.hpp"
#include "../public_inputs/public_inputs.hpp"
#include "../utils/linearizer.hpp"
#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <plonk/transcript/transcript.hpp>
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

    return *this;
}

template <typename program_settings> bool VerifierBase<program_settings>::verify_proof(const waffle::plonk_proof& proof)
{
    transcript::Transcript transcript = transcript::Transcript(proof.proof_data, manifest);

    std::array<g1::affine_element, program_settings::program_width> T;
    std::array<g1::affine_element, program_settings::program_width> W;

    std::array<fr, program_settings::program_width> wire_evaluations;
    std::array<fr, program_settings::program_width - 1> sigma_evaluations;

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        std::string index = std::to_string(i + 1);
        T[i] = g1::affine_element::serialize_from_buffer(&transcript.get_element("T_" + index)[0]);
        W[i] = g1::affine_element::serialize_from_buffer(&transcript.get_element("W_" + index)[0]);
        wire_evaluations[i] = fr::serialize_from_buffer(&transcript.get_element("w_" + index)[0]);
    }
    for (size_t i = 0; i < program_settings::program_width - 1; ++i) {
        std::string index = std::to_string(i + 1);
        sigma_evaluations[i] = fr::serialize_from_buffer(&transcript.get_element("sigma_" + index)[0]);
    }
    g1::affine_element Z_1 = g1::affine_element::serialize_from_buffer(&transcript.get_element("Z")[0]);
    g1::affine_element PI_Z = g1::affine_element::serialize_from_buffer(&transcript.get_element("PI_Z")[0]);
    g1::affine_element PI_Z_OMEGA = g1::affine_element::serialize_from_buffer(&transcript.get_element("PI_Z_OMEGA")[0]);

    fr linear_eval = fr::serialize_from_buffer(&transcript.get_element("r")[0]);
    fr z_1_shifted_eval = fr::serialize_from_buffer(&transcript.get_element("z_omega")[0]);

    bool inputs_valid = T[0].on_curve() && Z_1.on_curve() && PI_Z.on_curve();

    if (!inputs_valid) {
        printf("inputs not valid!\n");
        printf("T[0] on curve: %u \n", T[0].on_curve() ? 1 : 0);
        printf("Z_1 on curve: %u \n", Z_1.on_curve() ? 1 : 0);
        printf("PI_Z on curve: %u \n", PI_Z.on_curve() ? 1 : 0);
        return false;
    }

    bool instance_valid = true;
    for (size_t i = 0; i < program_settings::program_width; ++i) {
        instance_valid =
            instance_valid &&
            key->permutation_selectors.at("SIGMA_" + std::to_string(i + 1)).on_curve(); // SIGMA[i].on_curve();
    }
    if (!instance_valid) {
        printf("instance not valid!\n");
        return false;
    }

    bool field_elements_valid = true;
    for (size_t i = 0; i < program_settings::program_width - 1; ++i) {
        field_elements_valid = field_elements_valid && !(sigma_evaluations[i] == fr::zero());
    }
    field_elements_valid = field_elements_valid && !(linear_eval == fr::zero());

    if (!field_elements_valid) {
        printf("proof field elements not valid!\n");
        return false;
    }

    // reconstruct challenges
    fr alpha_pow[4];
    fr nu_pow[11];

    transcript.add_element("circuit_size",
                           { static_cast<uint8_t>(key->n),
                             static_cast<uint8_t>(key->n >> 8),
                             static_cast<uint8_t>(key->n >> 16),
                             static_cast<uint8_t>(key->n >> 24) });
    transcript.add_element("public_input_size",
                           { static_cast<uint8_t>(key->num_public_inputs),
                             static_cast<uint8_t>(key->num_public_inputs >> 8),
                             static_cast<uint8_t>(key->num_public_inputs >> 16),
                             static_cast<uint8_t>(key->num_public_inputs >> 24) });
    transcript.apply_fiat_shamir("init");
    fr beta = fr::serialize_from_buffer(transcript.apply_fiat_shamir("beta").begin());
    fr gamma = fr::serialize_from_buffer(transcript.apply_fiat_shamir("gamma").begin());
    fr alpha = fr::serialize_from_buffer(transcript.apply_fiat_shamir("alpha").begin());
    fr z_challenge = fr::serialize_from_buffer(transcript.apply_fiat_shamir("z").begin());

    fr t_eval = fr::zero();

    barretenberg::polynomial_arithmetic::lagrange_evaluations lagrange_evals =
        barretenberg::polynomial_arithmetic::get_lagrange_evaluations(z_challenge, key->domain);

    // compute the terms we need to derive R(X)
    plonk_linear_terms linear_terms = compute_linear_terms<program_settings>(transcript, lagrange_evals.l_1);

    // reconstruct evaluation of quotient polynomial from prover messages
    fr T0;
    fr T1;
    fr T2;
    fr::__copy(alpha, alpha_pow[0]);
    for (size_t i = 1; i < 4; ++i) {
        alpha_pow[i] = alpha_pow[i - 1] * alpha_pow[0];
    }

    fr sigma_contribution = fr::one();
    const size_t sigma_end =
        (program_settings::use_linearisation ? program_settings::program_width - 1 : program_settings::program_width);
    for (size_t i = 0; i < sigma_end; ++i) {
        T0 = sigma_evaluations[i] * beta;
        T1 = wire_evaluations[i] + gamma;
        T0 += T1;
        sigma_contribution *= T0;
    }

    std::vector<barretenberg::fr> public_inputs =
        barretenberg::fr::from_buffer(transcript.get_element("public_inputs"));

    fr public_input_delta = compute_public_input_delta(public_inputs, beta, gamma, key->domain.root);
    T0 = wire_evaluations[program_settings::program_width - 1] + gamma;
    sigma_contribution *= T0;
    sigma_contribution *= z_1_shifted_eval;
    sigma_contribution *= alpha_pow[0];

    T1 = z_1_shifted_eval - public_input_delta;
    T1 *= lagrange_evals.l_n_minus_1;
    T1 *= alpha_pow[1];

    T2 = lagrange_evals.l_1 * alpha_pow[2];
    T1 -= T2;
    T1 -= sigma_contribution;
    T1 += linear_eval;
    t_eval += T1;

    fr alpha_base = alpha.sqr().sqr();

    alpha_base = program_settings::compute_quotient_evaluation_contribution(key.get(), alpha_base, transcript, t_eval);

    T0 = lagrange_evals.vanishing_poly.invert();
    t_eval *= T0;

    transcript.add_element("t", t_eval.to_buffer());

    fr nu = fr::serialize_from_buffer(transcript.apply_fiat_shamir("nu").begin());

    fr u = fr::serialize_from_buffer(transcript.apply_fiat_shamir("separator").begin());
    fr::__copy(nu, nu_pow[0]);
    for (size_t i = 1; i < 10; ++i) {
        nu_pow[i] = nu_pow[i - 1] * nu_pow[0];
    }

    // reconstruct Kate opening commitments from committed values
    linear_terms.z_1 *= nu_pow[0];
    linear_terms.sigma_last *= nu_pow[0];

    T0 = nu_pow[7] * u;
    linear_terms.z_1 += T0;

    fr batch_evaluation;
    fr::__copy(t_eval, batch_evaluation);
    T0 = nu_pow[0] * linear_eval;
    batch_evaluation += T0;

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        T0 = nu_pow[i + 1] * wire_evaluations[i];
        batch_evaluation += T0;
    }

    for (size_t i = 0; i < program_settings::program_width - 1; ++i) {
        T0 = nu_pow[5 + i] * sigma_evaluations[i];
        batch_evaluation += T0;
    }

    T0 = nu_pow[7] * u;
    T0 *= z_1_shifted_eval;
    batch_evaluation += T0;

    fr nu_base = nu_pow[8];

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        if (program_settings::requires_shifted_wire(program_settings::wire_shift_settings, i)) {
            fr wire_shifted_eval =
                fr::serialize_from_buffer(&transcript.get_element("w_" + std::to_string(i + 1) + "_omega")[0]);
            T0 = wire_shifted_eval * nu_base;
            T0 *= u;
            batch_evaluation += T0;
            nu_base *= nu_pow[0];
        }
    }

    nu_base = program_settings::compute_batch_evaluation_contribution(key.get(), batch_evaluation, nu_base, transcript);

    batch_evaluation.self_neg();

    fr z_omega_scalar;
    z_omega_scalar = z_challenge * key->domain.root;
    z_omega_scalar *= u;

    std::vector<fr> scalars;
    std::vector<g1::affine_element> elements;

    elements.emplace_back(Z_1);
    scalars.emplace_back(linear_terms.z_1);

    fr::__copy(nu_pow[8], nu_base);

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        if (W[i].on_curve()) {
            elements.emplace_back(W[i]);
            if (program_settings::requires_shifted_wire(program_settings::wire_shift_settings, i)) {
                T0 = nu_base * u;
                T0 += nu_pow[1 + i];
                scalars.emplace_back(T0);
                nu_base *= nu_pow[0];
            } else {
                scalars.emplace_back(nu_pow[1 + i]);
            }
        }
    }

    for (size_t i = 0; i < program_settings::program_width - 1; ++i) {
        elements.emplace_back(key->permutation_selectors.at("SIGMA_" + std::to_string(i + 1)));
        scalars.emplace_back(nu_pow[5 + i]);
    }

    elements.emplace_back(key->permutation_selectors.at("SIGMA_" + std::to_string(program_settings::program_width)));
    scalars.emplace_back(linear_terms.sigma_last);

    elements.emplace_back(g1::affine_one);
    scalars.emplace_back(batch_evaluation);

    if (PI_Z_OMEGA.on_curve()) {
        elements.emplace_back(PI_Z_OMEGA);
        scalars.emplace_back(z_omega_scalar);
    }

    elements.emplace_back(PI_Z);
    scalars.emplace_back(z_challenge);

    for (size_t i = 1; i < program_settings::program_width; ++i) {
        fr z_power = z_challenge.pow(static_cast<uint64_t>(key->n * i));
        if (T[i].on_curve()) {
            elements.emplace_back(T[i]);
            scalars.emplace_back(z_power);
        }
    }

    VerifierBaseWidget::challenge_coefficients coeffs{ alpha.sqr().sqr(), alpha, nu_base, nu, nu };

    coeffs = program_settings::append_scalar_multiplication_inputs(key.get(), coeffs, transcript, elements, scalars);

    size_t num_elements = elements.size();
    elements.resize(num_elements * 2);
    barretenberg::scalar_multiplication::generate_pippenger_point_table(&elements[0], &elements[0], num_elements);
    scalar_multiplication::pippenger_runtime_state state(num_elements);
    g1::element P[2];

    P[0] = g1::affine_element(g1::element(PI_Z_OMEGA) * u);
    P[1] = barretenberg::scalar_multiplication::pippenger(&scalars[0], &elements[0], num_elements, state);

    P[1] += T[0];
    P[0] += PI_Z;
    P[0] = -P[0];
    g1::element::batch_normalize(P, 2);

    g1::affine_element P_affine[2];
    barretenberg::fq::__copy(P[0].x, P_affine[1].x);
    barretenberg::fq::__copy(P[0].y, P_affine[1].y);
    barretenberg::fq::__copy(P[1].x, P_affine[0].x);
    barretenberg::fq::__copy(P[1].y, P_affine[0].y);

    barretenberg::fq12 result = barretenberg::pairing::reduced_ate_pairing_batch_precomputed(
        P_affine, key->reference_string.precomputed_g2_lines, 2);

    return (result == barretenberg::fq12::one());
}

template class VerifierBase<standard_verifier_settings>;
template class VerifierBase<mimc_verifier_settings>;
template class VerifierBase<turbo_verifier_settings>;

} // namespace waffle