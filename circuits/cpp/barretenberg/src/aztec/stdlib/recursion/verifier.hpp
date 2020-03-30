#pragma once

#include "../primitives/bigfield/bigfield.hpp"
#include "../primitives/biggroup/biggroup.hpp"
#include "../primitives/bool/bool.hpp"
#include "../primitives/field/field.hpp"

namespace stdlib {
namespace recursion {

template <typename Group> struct recursion_output {
    Group P1;
    Group P2;
};

template <typename Composer> struct lagrange_evaluations {
    field_t<Composer> l_1;
    field_t<Composer> l_n_minus_1;
    field_t<Composer> vanishing_poly;
};

template <typename Composer>
lagrange_evaluations<Composer> get_lagrange_evaluations(const field_t<Composer>& z, const evaluation_domain& domain)
{
    using field_t = field_t<Composer>;
    field_t z_pow = z;
    for (size_t i = 0; i < domain.log2_size; ++i) {
        z_pow *= z_pow;
    }
    field_t numerator = z_pow - field_t(1);

    lagrange_evaluations<Composer> result;
    result.vanishing_poly = numerator / (z - domain.root_inverse);
    result.l_1 = numerator / (z - field_t(1));
    result.l_n_minus_1 = numerator / ((z * domain.root.sqr()) - field_t(1));
    return result;
}

template <typename Composer, typename program_settings>
recursion_output<Group> verify_proof(Composer* context,
                                     std::shared_ptr<verification_key> verifier_key,
                                     const transcript::Manifest& manifest,
                                     const waffle::plonk_proof& proof)
{
    using bool_t = bool_t<Composer>;
    using field_t = field_t<Composer>;
    using witness_t = witness_t<Composer>;
    using fq_t = bigfield<Composer, barretenberg::Bn254FqParams>;
    using group_t = element<Composer, fq_t, field_t>;

    auto convert_fq = [&](const barretenberg::fq& input) {
        field_t<Composer> low(context);
        field_t<Composer> high(context);
        uint256_t input_u256(input);
        field_t low(witness_t(context, input_u256.slice(0, 128));
        field_t hi(witness_t(context, input_u256.slice(128, 256)));
        return fq(context, low, hi);
    };

    auto convert_g1 = [&](const barretenberg::g1::affine_element& input) {
        fq_t x = convert_fq(input.x);
        fq_t y = convert_fq(input.y);
        return group_t(context, x, y);
    };

    Transcript transcript =
        Transcript(proof.proof_data, manifest, program_settings::hash_type, program_settings::num_challenge_bytes);

    std::array<group_t, program_settings::program_width> T;
    std::array<group_t, program_settings::program_width> W;
    std::array<group_t, program_settings::program_width> Sigmas;
    constexpr size_t num_sigma_evaluations =
        (program_settings::use_linearisation ? program_settings::program_width - 1 : program_settings::program_width);

    std::array<field_t, program_settings::program_width> wire_evaluations;
    std::array<field_t, num_sigma_evaluations> sigma_evaluations;

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        std::string index = std::to_string(i + 1);
        T[i] = transcript.get_group_element("T_" + index);
        W[i] = transcript.get_group_element("W_" + index);
        wire_evaluations[i] = transcript.get_field_element("w_" + index);
        Sigmas[i] = convert_g1(key->permutation_selectors.at("SIGMA_" + std::to_string(i + 1)));
    }

    for (size_t i = 0; i < num_sigma_evaluations; ++i) {
        std::string index = std::to_string(i + 1);
        sigma_evaluations[i] = transcript.get_field_element("sigma_" + index);
    }

    group_t Z_1 = transcript.get_group_element("Z");
    group_t PI_Z = transcript.get_group_element("PI_Z");
    group_t PI_Z_OMEGA = transcript.get_group_element("PI_Z_OMEGA");

    field_t z_1_shifted_eval = transcript.get_field_element("z_omega");

    bool_t inputs_valid = T[0].on_curve() && Z_1.on_curve() && PI_Z.on_curve();
    for (size_t i = 0; i < settings::program_width; ++i) {
        inputs_valid = inputs_valid && Sigmas[i].on_curve();
        inputs_valid = inputs_valid && !(sigma_evaluations[i] == field_t(0));
    }
    if constexpr (program_settings::use_linearisation) {
        field_t linear_eval = transcript.get_field_element("r");
        inputs_valid = inputs_valid && !(linear_eval == field_t(0)());
    }
    context->assert_equal_constant(inputs_valid, barretenberg::fr(1));

    field_t alpha_pow[4];

    transcript.add_const_element("circuit_size",
                                 { static_cast<uint8_t>(key->n),
                                   static_cast<uint8_t>(key->n >> 8),
                                   static_cast<uint8_t>(key->n >> 16),
                                   static_cast<uint8_t>(key->n >> 24) });
    transcript.add_const_element("public_input_size",
                                 { static_cast<uint8_t>(key->num_public_inputs),
                                   static_cast<uint8_t>(key->num_public_inputs >> 8),
                                   static_cast<uint8_t>(key->num_public_inputs >> 16),
                                   static_cast<uint8_t>(key->num_public_inputs >> 24) });
    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("alpha");
    transcript.apply_fiat_shamir("z");

    field_t beta = transcript.get_challenge_field_element("beta");
    field_t alpha = transcript.get_challenge_field_element("alpha");
    field_t z_challenge = transcript.get_challenge_field_element("z");
    field_t gamma = transcript.get_challenge_field_element("beta", 1);

    field_t t_eval = field_t(0);

    lagrange_evaluations<Composer> lagrange_evals = get_lagrange_evaluations(z_challenge, key->domain);

    plonk_linear_terms linear_terms =
        compute_linear_terms<field_t, Transcript, program_settings::program_width>(transcript, lagrange_evals.l_1);

    // reconstruct evaluation of quotient polynomial from prover messages
    field_t T0;
    field_t T1;
    field_t T2;
    alpha_pow[0] = alpha;
    for (size_t i = 1; i < 4; ++i) {
        alpha_pow[i] = alpha_pow[i - 1] * alpha_pow[0];
    }

    field_t sigma_contribution(context, barretenberg::fr(1));

    for (size_t i = 0; i < program_settings::program_width - 1; ++i) {
        sigma_contribution *= (sigma_evaluations[i] * beta + wire_evaluations[i] + gamma);
    }

    std::vector<field_t> public_inputs = (transcript.get_field_elements("public_inputs"));

    // TODO fix:
    field_t public_input_delta = compute_public_input_delta(public_inputs, beta, gamma, key->domain.root);
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

    if constexpr (program_settings::use_linearisation) {
        fr linear_eval = transcript.get_field_element("r")[0]);
        T1 += linear_eval;
    }
    t_eval += T1;

    fr alpha_base = alpha.sqr().sqr();

    alpha_base = program_settings::compute_quotient_evaluation_contribution(key.get(), alpha_base, transcript, t_eval);

    if constexpr (!program_settings::use_linearisation) {
        fr z_eval = transcript.get_field_element("z")[0]);
        t_eval += (linear_terms.z_1 * z_eval);
        t_eval += (linear_terms.sigma_last * sigma_evaluations[program_settings::program_width - 1]);
    }

    t_eval = t_eval / lagrange_evals.vanishing_poly;
    transcript.add_field_element("t", t_eval);

    transcript.apply_fiat_shamir("nu");
    transcript.apply_fiat_shamir("separator");

    std::vector<field_t> nu_challenges;
    for (size_t i = 0; i < transcript.get_num_challenges("nu"); ++i) {
        nu_challenges.emplace_back(transcript.get_challenge_field_element("nu", i));
    }
    field_t u = transcript.get_challenge_field_element("separator");

    field_t batch_evaluation = t_eval;
    constexpr size_t nu_offset = program_settings::use_linearisation ? 1 : 0;
    if constexpr (program_settings::use_linearisation) {
        field_t linear_eval = transcript.get_field_element("r");
        T0 = nu_challenges[0] * linear_eval;
        batch_evaluation += T0;
    } else {
        field_t z_eval = transcript.get_field_element("z");
        T0 = z_eval * nu_challenges[2 * program_settings::program_width];
        batch_evaluation += T0;
        T0 = nu_challenges[2 * program_settings::program_width - 1] *
             sigma_evaluations[program_settings::program_width - 1];
        batch_evaluation += T0;
    }
    for (size_t i = 0; i < program_settings::program_width; ++i) {
        T0 = nu_challenges[i + nu_offset] * wire_evaluations[i];
        batch_evaluation += T0;
    }

    for (size_t i = 0; i < program_settings::program_width - 1; ++i) {
        T0 = nu_challenges[program_settings::program_width + i + nu_offset] * sigma_evaluations[i];
        batch_evaluation += T0;
    }

    constexpr size_t nu_z_offset = (program_settings::use_linearisation) ? 2 * program_settings::program_width
                                                                         : 2 * program_settings::program_width + 1;

    T0 = nu_challenges[nu_z_offset] * u;
    T0 *= z_1_shifted_eval;
    batch_evaluation += T0;

    size_t nu_ptr = nu_z_offset + 1;

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        if (program_settings::requires_shifted_wire(program_settings::wire_shift_settings, i)) {
            field_t wire_shifted_eval =
                transcript.get_field_element("w_" + std::to_string(i + 1) + "_omega")[0]);
            T0 = wire_shifted_eval * nu_challenges[nu_ptr++];
            T0 *= u;
            batch_evaluation += T0;
        }
    }

    program_settings::compute_batch_evaluation_contribution(key.get(), batch_evaluation, nu_ptr, transcript);

    batch_evaluation = -batch_evaluation;

    field_t z_omega_scalar;
    z_omega_scalar = z_challenge * key->domain.root;
    z_omega_scalar *= u;

    std::vector<field_t> scalars;
    std::vector<group_t> elements;

    elements.emplace_back(Z_1);
    if constexpr (program_settings::use_linearisation) {
        linear_terms.z_1 *= nu_challenges[0];
        linear_terms.z_1 += (nu_challenges[nu_z_offset] * u);
        scalars.emplace_back(linear_terms.z_1);
    } else {
        T0 = nu_challenges[nu_z_offset] * u + nu_challenges[2 * program_settings::program_width];
        scalars.emplace_back(T0);
    }

    nu_ptr = nu_z_offset + 1;
    for (size_t i = 0; i < program_settings::program_width; ++i) {
        if (W[i].on_curve()) {
            elements.emplace_back(W[i]);
            if (program_settings::requires_shifted_wire(program_settings::wire_shift_settings, i)) {
                T0 = nu_challenges[nu_ptr] * u;
                T0 += nu_challenges[i + nu_offset];
                scalars.emplace_back(T0);
            } else {
                scalars.emplace_back(nu_challenges[i + nu_offset]);
            }
        }
        if (program_settings::requires_shifted_wire(program_settings::wire_shift_settings, i)) {
            ++nu_ptr;
        }
    }

    for (size_t i = 0; i < program_settings::program_width - 1; ++i) {
        elements.emplace_back(sigmas[i]);
        scalars.emplace_back(nu_challenges[program_settings::program_width + i + nu_offset]);
    }

    if constexpr (program_settings::use_linearisation) {
        elements.emplace_back(sigmas[program_settings::program_width - 1]);
        linear_terms.sigma_last *= nu_challenges[0];
        scalars.emplace_back(linear_terms.sigma_last);
    } else {
        elements.emplace_back(sigmas[program_settings::program_width - 1]);
        scalars.emplace_back(nu_challenges[2 * program_settings::program_width - 1]);
    }

    elements.emplace_back(group_t::one());
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

    VerifierBaseWidget::challenge_coefficients<field_t> coeffs{ alpha.sqr().sqr(), alpha, nu_ptr, 0 };

    std::vector<barretenberg::g1::affine_element> g1_inputs;
    program_settings::append_scalar_multiplication_inputs(key.get(), coeffs, transcript, g1_inputs, scalars);
    for (size_t i = 0; i < g1_inputs.size(); ++i) {
        elements.push_back(convert_g1(g1_inputs[i]));
    }

    recursion_output<group_t> result{
        (PI_Z_OMEGA * u) + PI_Z,
        group_t::batch_mul(scalars, elements),
    };

    return result;
}

} // namespace recursion
} // namespace stdlib