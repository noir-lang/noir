#pragma once

#include "../../primitives/field/field.hpp"
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"

#include "../transcript/transcript.hpp"

#include <plonk/proof_system/utils/linearizer.hpp>
#include <plonk/proof_system/public_inputs/public_inputs.hpp>

#include <plonk/proof_system/widgets/turbo_fixed_base_widget.hpp>

#include <polynomials/polynomial_arithmetic.hpp>

#include <ecc/curves/bn254/fq12.hpp>
#include <ecc/curves/bn254/pairing.hpp>

namespace plonk {
namespace stdlib {
namespace recursion {

template <typename Field, typename Group> struct recursion_output {
    Group P0;
    Group P1;
    // the public inputs of the inner ciruit are now private inputs of the outer circuit!
    std::vector<Field> public_inputs;
};

template <typename Composer> struct lagrange_evaluations {
    field_t<Composer> l_1;
    field_t<Composer> l_n_minus_1;
    field_t<Composer> vanishing_poly;
};

template <typename Composer>
lagrange_evaluations<Composer> get_lagrange_evaluations(const field_t<Composer>& z, const evaluation_domain& domain)
{
    using field_pt = field_t<Composer>;
    field_pt z_pow = z;
    for (size_t i = 0; i < domain.log2_size; ++i) {
        z_pow *= z_pow;
    }
    field_pt numerator = z_pow - field_pt(1);

    lagrange_evaluations<Composer> result;
    result.vanishing_poly = numerator / (z - domain.root_inverse);
    numerator *= domain.domain_inverse;
    result.l_1 = numerator / (z - field_pt(1));
    result.l_n_minus_1 = numerator / ((z * domain.root.sqr()) - field_pt(1));

    barretenberg::polynomial_arithmetic::get_lagrange_evaluations(z.get_value(), domain);
    return result;
}

template <typename Composer, typename program_settings>
recursion_output<
    field_t<Composer>,
    element<Composer, bigfield<Composer, barretenberg::Bn254FqParams>, field_t<Composer>, barretenberg::g1>>
verify_proof(Composer* context,
             std::shared_ptr<waffle::verification_key> key,
             const transcript::Manifest& manifest,
             const waffle::plonk_proof& proof)
{
    using field_pt = field_t<Composer>;
    using fq_pt = bigfield<Composer, barretenberg::Bn254FqParams>;
    using group_pt = element<Composer, fq_pt, field_pt, barretenberg::g1>;

    key->program_width = program_settings::program_width;

    Transcript<Composer> transcript = Transcript<Composer>(context, proof.proof_data, manifest);
    std::array<group_pt, program_settings::program_width> T;
    std::array<group_pt, program_settings::program_width> W;

    std::array<field_pt, program_settings::program_width> wire_evaluations;

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        std::string index = std::to_string(i + 1);
        T[i] = transcript.get_circuit_group_element("T_" + index);
        W[i] = transcript.get_circuit_group_element("W_" + index);
        wire_evaluations[i] = transcript.get_field_element("w_" + index);
    }

    group_pt Z_1 = transcript.get_circuit_group_element("Z");
    group_pt PI_Z = transcript.get_circuit_group_element("PI_Z");
    group_pt PI_Z_OMEGA = transcript.get_circuit_group_element("PI_Z_OMEGA");

    T[0].validate_on_curve();
    Z_1.validate_on_curve();
    PI_Z.validate_on_curve();

    field_t circuit_size(stdlib::witness_t(context, barretenberg::fr(key->n)));
    field_t public_input_size(stdlib::witness_t(context, barretenberg::fr(key->num_public_inputs)));

    transcript.add_field_element("circuit_size", circuit_size);
    transcript.add_field_element("public_input_size", public_input_size);

    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("alpha");
    transcript.apply_fiat_shamir("z");
    field_pt alpha = transcript.get_challenge_field_element("alpha");
    field_pt z_challenge = transcript.get_challenge_field_element("z");
    lagrange_evaluations<Composer> lagrange_evals = get_lagrange_evaluations(z_challenge, key->domain);

    // reconstruct evaluation of quotient polynomial from prover messages
    field_pt T0;

    field_pt t_eval = field_pt(0);

    field_pt alpha_base = alpha;

    alpha_base = program_settings::compute_quotient_evaluation_contribution(key.get(), alpha_base, transcript, t_eval);

    t_eval = t_eval / lagrange_evals.vanishing_poly;
    transcript.add_field_element("t", t_eval);

    transcript.apply_fiat_shamir("nu");
    transcript.apply_fiat_shamir("separator");

    field_pt u = transcript.get_challenge_field_element("separator");

    field_pt batch_evaluation = t_eval;

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        const std::string wire_key = "w_" + std::to_string(i + 1);
        field_pt wire_challenge = transcript.get_challenge_field_element_from_map("nu", wire_key);
        T0 = wire_challenge * wire_evaluations[i];
        batch_evaluation += T0;

        if (program_settings::requires_shifted_wire(program_settings::wire_shift_settings, i)) {
            field_pt wire_shifted_eval = transcript.get_field_element("w_" + std::to_string(i + 1) + "_omega");
            T0 = wire_shifted_eval * wire_challenge;
            T0 *= u;
            batch_evaluation += T0;
        }
    }

    program_settings::compute_batch_evaluation_contribution(key.get(), batch_evaluation, transcript);

    batch_evaluation = -batch_evaluation;

    field_pt z_omega_scalar;
    z_omega_scalar = z_challenge * key->domain.root;
    z_omega_scalar *= u;

    std::vector<field_pt> big_scalars;
    std::vector<group_pt> big_elements;
    std::vector<field_pt> small_scalars;
    std::vector<group_pt> small_elements;

    for (size_t i = 0; i < program_settings::program_width; ++i) {
        W[i].validate_on_curve();
        big_elements.emplace_back(W[i]);

        const std::string wire_key = "w_" + std::to_string(i + 1);
        field_pt wire_challenge = transcript.get_challenge_field_element_from_map("nu", wire_key);

        if (program_settings::requires_shifted_wire(program_settings::wire_shift_settings, i)) {
            T0 = wire_challenge * u;
            T0 += wire_challenge;
            big_scalars.emplace_back(T0);
        } else {
            big_scalars.emplace_back(wire_challenge);
        }
    }

    big_elements.emplace_back(group_pt::one(context));
    big_scalars.emplace_back(batch_evaluation);

    PI_Z_OMEGA.validate_on_curve();
    big_elements.emplace_back(PI_Z_OMEGA);
    big_scalars.emplace_back(z_omega_scalar);

    small_elements.emplace_back(PI_Z);
    small_scalars.emplace_back(z_challenge);

    // TODO FIX
    field_pt z_pow_n = z_challenge;
    const size_t log2_n = numeric::get_msb(key->n);
    for (size_t j = 0; j < log2_n; ++j) {
        z_pow_n = z_pow_n.sqr();
    }
    field_pt z_power = z_pow_n;
    for (size_t i = 1; i < program_settings::program_width; ++i) {
        T[i].validate_on_curve();
        big_elements.emplace_back(T[i]);
        big_scalars.emplace_back(z_power);

        z_power *= z_pow_n;
    }

    std::vector<barretenberg::g1::affine_element> g1_inputs;
    program_settings::append_scalar_multiplication_inputs(key.get(), alpha, transcript, g1_inputs, small_scalars);
    for (size_t i = 0; i < g1_inputs.size(); ++i) {
        small_elements.push_back(Transcript<waffle::TurboComposer>::convert_g1(context, g1_inputs[i]));
        // TODO: add method of enabling widgets to directly add transcript G1 elements into array
        if (i == 0) {
            auto input = small_elements[small_elements.size() - 1];
            context->assert_equal(Z_1.x.binary_basis_limbs[0].element.witness_index,
                                  input.x.binary_basis_limbs[0].element.witness_index);
            context->assert_equal(Z_1.x.binary_basis_limbs[1].element.witness_index,
                                  input.x.binary_basis_limbs[1].element.witness_index);
            context->assert_equal(Z_1.x.binary_basis_limbs[2].element.witness_index,
                                  input.x.binary_basis_limbs[2].element.witness_index);
            context->assert_equal(Z_1.x.binary_basis_limbs[3].element.witness_index,
                                  input.x.binary_basis_limbs[3].element.witness_index);
            context->assert_equal(Z_1.y.binary_basis_limbs[0].element.witness_index,
                                  input.y.binary_basis_limbs[0].element.witness_index);
            context->assert_equal(Z_1.y.binary_basis_limbs[1].element.witness_index,
                                  input.y.binary_basis_limbs[1].element.witness_index);
            context->assert_equal(Z_1.y.binary_basis_limbs[2].element.witness_index,
                                  input.y.binary_basis_limbs[2].element.witness_index);
            context->assert_equal(Z_1.y.binary_basis_limbs[3].element.witness_index,
                                  input.y.binary_basis_limbs[3].element.witness_index);
            context->assert_equal(Z_1.x.prime_basis_limb.witness_index, input.x.prime_basis_limb.witness_index);
            context->assert_equal(Z_1.y.prime_basis_limb.witness_index, input.y.prime_basis_limb.witness_index);
        }
    }
    group_pt rhs = group_pt::mixed_batch_mul(big_elements, big_scalars, small_elements, small_scalars, 129);
    rhs = (rhs + T[0]).normalize();
    group_pt lhs = group_pt::batch_mul({ PI_Z_OMEGA }, { u }, 128);
    lhs = lhs + PI_Z;
    lhs = (-lhs).normalize();
    return recursion_output<field_pt, group_pt>{
        rhs,
        lhs,
        transcript.get_field_element_vector("public_inputs"),
    };
}

} // namespace recursion
} // namespace stdlib
} // namespace plonk