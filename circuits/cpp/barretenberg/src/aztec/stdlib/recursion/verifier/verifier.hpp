#pragma once

#include "../../primitives/field/field.hpp"
#include "../../primitives/bool/bool.hpp"
#include "../../primitives/bigfield/bigfield.hpp"
#include "../../primitives/biggroup/biggroup.hpp"

#include "../transcript/transcript.hpp"

#include <plonk/proof_system/utils/linearizer.hpp>
#include <plonk/proof_system/utils/kate_verification.hpp>
#include <plonk/proof_system/public_inputs/public_inputs.hpp>

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
    bool has_data = false;
};

template <typename Composer> struct lagrange_evaluations {
    field_t<Composer> l_1;
    field_t<Composer> l_n_minus_1;
    field_t<Composer> vanishing_poly;
};

template <typename Field, typename Group, typename Transcript, typename program_settings>
void populate_kate_element_map(waffle::verification_key* key,
                               const Transcript& transcript,
                               std::map<std::string, Group>& kate_g1_elements,
                               std::map<std::string, Field>& kate_fr_elements_at_zeta,
                               std::map<std::string, Field>& kate_fr_elements_at_zeta_large,
                               std::map<std::string, Field>& kate_fr_elements_at_zeta_omega)
{
    // const auto separator_challenge = transcript.get_challenge_field_element("separator", 0);

    const auto& polynomial_manifest = key->polynomial_manifest;
    for (size_t i = 0; i < key->polynomial_manifest.size(); ++i) {
        const auto& item = polynomial_manifest[i];
        const std::string label(item.commitment_label);
        const std::string poly_label(item.polynomial_label);
        switch (item.source) {
        case waffle::PolynomialSource::WITNESS: {
            const auto element = transcript.get_group_element(label);
            ASSERT(element.on_curve());
            kate_g1_elements.insert({ label, element });
            break;
        }
        case waffle::PolynomialSource::SELECTOR: {
            const auto element = key->constraint_selectors.at(label);
            kate_g1_elements.insert({ label, element });
            break;
        }
        case waffle::PolynomialSource::PERMUTATION: {
            const auto element = key->permutation_selectors.at(label);
            kate_g1_elements.insert({ label, element });
            break;
        }
        }
        if (item.requires_shifted_evaluation) {
            const auto challenge = transcript.get_challenge_field_element_from_map("nu", poly_label + "_omega");
            kate_fr_elements_at_zeta_omega.insert({ label, challenge });
        } else {
            const auto challenge = transcript.get_challenge_field_element_from_map("nu", poly_label);
            kate_fr_elements_at_zeta.insert({ label, challenge });
        }
    }

    const auto zeta = transcript.get_challenge_field_element("z", 0);
    const auto quotient_nu = transcript.get_challenge_field_element_from_map("nu", "t");

    Field z_pow_n = zeta;
    const size_t log2_n = numeric::get_msb(key->n);
    for (size_t j = 0; j < log2_n; ++j) {
        z_pow_n = z_pow_n.sqr();
    }
    Field z_power = 1;
    for (size_t i = 0; i < program_settings::program_width; ++i) {
        std::string quotient_label = "T_" + std::to_string(i + 1);
        const auto element = transcript.get_group_element(quotient_label);
        // const auto scalar = quotient_nu * zeta.pow(static_cast<uint64_t>(i * key->domain.size));

        kate_g1_elements.insert({ quotient_label, element });
        kate_fr_elements_at_zeta_large.insert({ quotient_label, quotient_nu * z_power });
        z_power *= z_pow_n;
    }

    const auto PI_Z = transcript.get_group_element("PI_Z");
    const auto PI_Z_OMEGA = transcript.get_group_element("PI_Z_OMEGA");

    Field u = transcript.get_challenge_field_element("separator", 0);

    Field batch_evaluation =
        waffle::compute_kate_batch_evaluation<Field, Transcript, program_settings>(key, transcript);
    kate_g1_elements.insert({ "BATCH_EVALUATION", g1::affine_one });
    kate_fr_elements_at_zeta_large.insert({ "BATCH_EVALUATION", -batch_evaluation });

    kate_g1_elements.insert({ "PI_Z_OMEGA", PI_Z_OMEGA });
    kate_fr_elements_at_zeta_large.insert({ "PI_Z_OMEGA", zeta * key->domain.root * u });

    kate_g1_elements.insert({ "PI_Z", PI_Z });
    kate_fr_elements_at_zeta.insert({ "PI_Z", zeta });
}

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

    return result;
}

template <typename Composer, typename program_settings>
recursion_output<
    field_t<Composer>,
    element<Composer, bigfield<Composer, barretenberg::Bn254FqParams>, field_t<Composer>, barretenberg::g1>>
verify_proof(
    Composer* context,
    std::shared_ptr<waffle::verification_key> key,
    const transcript::Manifest& manifest,
    const waffle::plonk_proof& proof,
    const recursion_output<
        field_t<Composer>,
        element<Composer, bigfield<Composer, barretenberg::Bn254FqParams>, field_t<Composer>, barretenberg::g1>>
        previous_output = recursion_output<
            field_t<Composer>,
            element<Composer, bigfield<Composer, barretenberg::Bn254FqParams>, field_t<Composer>, barretenberg::g1>>())
{
    using field_pt = field_t<Composer>;
    using fq_pt = bigfield<Composer, barretenberg::Bn254FqParams>;
    using group_pt = element<Composer, fq_pt, field_pt, barretenberg::g1>;

    key->program_width = program_settings::program_width;

    Transcript<Composer> transcript = Transcript<Composer>(context, proof.proof_data, manifest);
    std::map<std::string, barretenberg::g1::affine_element> kate_g1_elements;
    std::map<std::string, field_pt> kate_fr_elements_at_zeta;
    std::map<std::string, field_pt> kate_fr_elements_at_zeta_large;
    std::map<std::string, field_pt> kate_fr_elements_at_zeta_omega;

    const auto PI_Z = transcript.get_group_element("PI_Z");
    const auto PI_Z_OMEGA = transcript.get_group_element("PI_Z_OMEGA");

    field_t circuit_size(stdlib::witness_t(context, barretenberg::fr(key->n)));
    field_t public_input_size(stdlib::witness_t(context, barretenberg::fr(key->num_public_inputs)));

    transcript.add_field_element("circuit_size", circuit_size);
    transcript.add_field_element("public_input_size", public_input_size);

    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("eta");
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("alpha");
    transcript.apply_fiat_shamir("z");
    field_pt alpha = transcript.get_challenge_field_element("alpha");
    field_pt zeta = transcript.get_challenge_field_element("z");
    lagrange_evaluations<Composer> lagrange_evals = get_lagrange_evaluations(zeta, key->domain);

    // reconstruct evaluation of quotient polynomial from prover messages
    field_pt T0;

    field_pt t_eval = field_pt(0);

    field_pt alpha_base = alpha;

    alpha_base = program_settings::compute_quotient_evaluation_contribution(key.get(), alpha_base, transcript, t_eval);

    t_eval = t_eval / lagrange_evals.vanishing_poly;
    transcript.add_field_element("t", t_eval);

    transcript.apply_fiat_shamir("nu");
    transcript.apply_fiat_shamir("separator");

    field_pt u = transcript.get_challenge_field_element("separator", 0);

    populate_kate_element_map<field_pt, g1::affine_element, Transcript<Composer>, program_settings>(
        key.get(),
        transcript,
        kate_g1_elements,
        kate_fr_elements_at_zeta,
        kate_fr_elements_at_zeta_large,
        kate_fr_elements_at_zeta_omega);

    std::vector<field_pt> double_opening_scalars;
    std::vector<group_pt> double_opening_elements;
    std::vector<field_pt> opening_scalars;
    std::vector<group_pt> opening_elements;
    std::vector<field_pt> big_opening_scalars;
    std::vector<group_pt> big_opening_elements;
    std::vector<group_pt> elements_to_add;
    for (const auto& [label, fr_value] : kate_fr_elements_at_zeta) {
        const auto& g1_value = kate_g1_elements[label];
        if (!g1_value.on_curve()) {
            continue; // TODO handle this
        }

        if (fr_value.get_value() == 0 && fr_value.witness_index == UINT32_MAX) {
            continue;
        }

        if (fr_value.get_value() == 1 && fr_value.witness_index == UINT32_MAX) {
            elements_to_add.emplace_back(Transcript<waffle::TurboComposer>::convert_g1(context, g1_value));
            continue;
        }
        opening_scalars.emplace_back(fr_value);
        opening_elements.emplace_back(Transcript<waffle::TurboComposer>::convert_g1(context, g1_value));
    }

    for (const auto& [label, fr_value] : kate_fr_elements_at_zeta_large) {
        const auto& g1_value = kate_g1_elements[label];
        if (!g1_value.on_curve()) {
            continue; // TODO handle this
        }

        if (fr_value.get_value() == 0 && fr_value.witness_index == UINT32_MAX) {
            continue;
        }

        if (fr_value.get_value() == 1 && fr_value.witness_index == UINT32_MAX) {
            elements_to_add.emplace_back(Transcript<waffle::TurboComposer>::convert_g1(context, g1_value));
            continue;
        }
        big_opening_scalars.emplace_back(fr_value);
        big_opening_elements.emplace_back(Transcript<waffle::TurboComposer>::convert_g1(context, g1_value));
    }

    for (const auto& [label, fr_value] : kate_fr_elements_at_zeta_omega) {
        const auto& g1_value = kate_g1_elements[label];
        if (!g1_value.on_curve()) {
            continue; // TODO handle this
        }

        if (fr_value.get_value() == 0 && fr_value.witness_index == UINT32_MAX) {
            continue;
        }
        double_opening_scalars.emplace_back(fr_value);
        double_opening_elements.emplace_back(Transcript<waffle::TurboComposer>::convert_g1(context, g1_value));
    }
    const auto double_opening_result = group_pt::batch_mul(double_opening_elements, double_opening_scalars, 128);

    opening_elements.emplace_back(double_opening_result);
    opening_scalars.emplace_back(u);

    std::vector<group_pt> lhs_elements;
    std::vector<field_pt> lhs_scalars;

    lhs_elements.push_back(PI_Z_OMEGA);
    lhs_scalars.push_back(u);

    if (previous_output.has_data) {
        field_pt random_separator = transcript.get_challenge_field_element("separator", 1);

        opening_elements.push_back(previous_output.P0);
        opening_scalars.push_back(random_separator);

        lhs_elements.push_back((-(previous_output.P1)).normalize());
        lhs_scalars.push_back(random_separator);
    }

    auto opening_result =
        group_pt::mixed_batch_mul(big_opening_elements, big_opening_scalars, opening_elements, opening_scalars, 128);

    opening_result = opening_result + double_opening_result;
    for (const auto& to_add : elements_to_add) {
        opening_result = opening_result + to_add;
    }
    opening_result = opening_result.normalize();

    group_pt lhs = group_pt::batch_mul(lhs_elements, lhs_scalars, 128);
    lhs = lhs + PI_Z;
    lhs = (-lhs).normalize();
    return recursion_output<field_pt, group_pt>{
        opening_result,
        lhs,
        transcript.get_field_element_vector("public_inputs"),
        true,
    };
}

} // namespace recursion
} // namespace stdlib
} // namespace plonk