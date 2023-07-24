#pragma once

#include "barretenberg/ecc/curves/bn254/fq12.hpp"
#include "barretenberg/ecc/curves/bn254/pairing.hpp"
#include "barretenberg/plonk/flavor/flavor.hpp"
#include "barretenberg/plonk/proof_system/public_inputs/public_inputs.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/plonk/proof_system/utils/kate_verification.hpp"
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/biggroup/biggroup.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/recursion/aggregation_state/aggregation_state.hpp"
#include "barretenberg/stdlib/recursion/transcript/transcript.hpp"
#include "barretenberg/stdlib/recursion/verifier/program_settings.hpp"

namespace proof_system::plonk {
namespace stdlib {
namespace recursion {

template <typename Builder> struct lagrange_evaluations {
    field_t<Builder> l_start;
    field_t<Builder> l_end;
    field_t<Builder> vanishing_poly;
};

template <typename Curve, typename Transcript, typename program_settings>
void populate_kate_element_map(typename Curve::Builder* ctx,
                               typename Transcript::Key* key,
                               const Transcript& transcript,
                               std::map<std::string, typename Curve::g1_ct>& kate_g1_elements,
                               std::map<std::string, typename Curve::fr_ct>& kate_fr_elements_at_zeta,
                               std::map<std::string, typename Curve::fr_ct>& kate_fr_elements_at_zeta_large,
                               std::map<std::string, typename Curve::fr_ct>& kate_fr_elements_at_zeta_omega,
                               typename Curve::fr_ct& batch_opening_scalar)
{
    using fr_ct = typename Curve::fr_ct;
    const auto& polynomial_manifest = key->polynomial_manifest;
    for (size_t i = 0; i < key->polynomial_manifest.size(); ++i) {
        const auto& item = polynomial_manifest[i];
        const std::string label(item.commitment_label);
        const std::string poly_label(item.polynomial_label);
        switch (item.source) {
        case PolynomialSource::WITNESS: {
            // get_circuit_group_element validates that the point produced lies on the curve
            const auto element = transcript.get_circuit_group_element(label);
            ASSERT(element.get_value().on_curve());
            if (element.get_value().is_point_at_infinity()) {
                std::cerr << label << " witness is point at infinity! Error!" << std::endl;
                ctx->failure("witness " + label + " is point at infinity");
            }
            kate_g1_elements.insert({ label, element });
            break;
        }
        case PolynomialSource::SELECTOR:
        case PolynomialSource::PERMUTATION: {
            const auto element = key->commitments.at(label);
            // TODO: with user-defined circuits, we will need verify that the point
            // lies on the curve with constraints
            if (!element.get_value().on_curve()) {
                std::cerr << label << " commitment not on curve!" << std::endl;
            }
            if (element.get_value().is_point_at_infinity()) {
                std::cerr << label << " commitment is point at infinity! Error!" << std::endl;
                ctx->failure("commitment " + label + " is point at infinity");
            }
            kate_g1_elements.insert({ label, element });
            break;
        }
        case PolynomialSource::OTHER: {
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

    fr_ct z_power = 1;
    for (size_t i = 0; i < program_settings::program_width; ++i) {
        std::string quotient_label = "T_" + std::to_string(i + 1);
        const auto element = transcript.get_circuit_group_element(quotient_label);

        kate_g1_elements.insert({ quotient_label, element });
        kate_fr_elements_at_zeta_large.insert({ quotient_label, quotient_nu * z_power });
        z_power *= key->z_pow_n;
    }

    const auto PI_Z = transcript.get_circuit_group_element("PI_Z");
    const auto PI_Z_OMEGA = transcript.get_circuit_group_element("PI_Z_OMEGA");

    fr_ct u = transcript.get_challenge_field_element("separator", 0);

    fr_ct batch_evaluation =
        proof_system::plonk::compute_kate_batch_evaluation<fr_ct, Transcript, program_settings>(key, transcript);
    batch_opening_scalar = -batch_evaluation;

    kate_g1_elements.insert({ "PI_Z_OMEGA", PI_Z_OMEGA });
    kate_fr_elements_at_zeta_large.insert({ "PI_Z_OMEGA", zeta * key->domain.root * u });

    kate_g1_elements.insert({ "PI_Z", PI_Z });
    kate_fr_elements_at_zeta.insert({ "PI_Z", zeta });
}

template <typename Curve>
lagrange_evaluations<typename Curve::Builder> get_lagrange_evaluations(
    const typename Curve::fr_ct& z,
    const evaluation_domain<typename Curve::Builder>& domain,
    const size_t num_roots_cut_out_of_vanishing_polynomial = 4)
{
    // compute Z_H*(z), l_start(z), l_{end}(z)
    // Note that as we modify the vanishing polynomial by cutting out some roots, we must simultaneously ensure that
    // the lagrange polynomials we require would be l_1(z) and l_{n-k}(z) where k =
    // num_roots_cut_out_of_vanishing_polynomial. For notational simplicity, we call l_1 as l_start and l_{n-k} as
    // l_end.
    //
    // NOTE: If in future, there arises a need to cut off more zeros, this method will not require any changes.
    //

    typedef typename Curve::fr_ct fr_ct;
    typedef typename Curve::Builder Builder;

    fr_ct z_pow = z.pow(field_t<Builder>(domain.size));
    fr_ct numerator = z_pow - fr_ct(1);

    // compute modified vanishing polynomial Z_H*(z)
    //                       (z^{n} - 1)
    // Z_H*(z) = --------------------------------------------
    //           (z - w^{n-1})(z - w^{n-2})...(z - w^{n - k})
    //
    fr_ct denominators_vanishing_poly = fr_ct(1);
    lagrange_evaluations<Builder> result;

    fr_ct work_root = domain.root_inverse;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial; ++i) {
        denominators_vanishing_poly *= (z - work_root);
        work_root *= domain.root_inverse;
    }
    result.vanishing_poly = numerator / denominators_vanishing_poly;

    // The expressions of the lagrange polynomials are:
    //           (X^n - 1)
    // L_1(X) = -----------
    //             X - 1
    //
    // L_{i}(X) = L_1(X.w^{-i+1})
    //                                                          (X^n - 1)
    // => L_{n-k}(X) = L_1(X.w^{k-n+1}) = L_1(X.w^{k + 1}) = ----------------
    //                                                        (X.w^{k+1} - 1)
    //
    numerator *= domain.domain_inverse;

    result.l_start = numerator / (z - fr_ct(1));

    // compute w^{num_roots_cut_out_of_vanishing_polynomial + 1}
    fr_ct l_end_root = (num_roots_cut_out_of_vanishing_polynomial & 1) ? domain.root.sqr() : domain.root;
    for (size_t i = 0; i < num_roots_cut_out_of_vanishing_polynomial / 2; ++i) {
        l_end_root *= domain.root.sqr();
    }
    result.l_end = numerator / ((z * l_end_root) - fr_ct(1));

    return result;
}

/**
 * Refer to src/barretenberg/plonk/proof_system/verifier/verifier.cpp verify_proof() for the native implementation,
 * which includes detailed comments.
 */
template <typename Curve, typename program_settings>
aggregation_state<Curve> verify_proof(typename Curve::Builder* context,
                                      std::shared_ptr<verification_key<Curve>> key,
                                      const transcript::Manifest& manifest,
                                      const plonk::proof& proof,
                                      const aggregation_state<Curve> previous_output = aggregation_state<Curve>())
{
    using Builder = typename Curve::Builder;

    key->program_width = program_settings::program_width;

    Transcript<Builder> transcript = Transcript<Builder>(context, proof.proof_data, manifest);

    return verify_proof_<Curve, program_settings>(context, key, transcript, previous_output);
}

/**
 * Refer to src/barretenberg/plonk/proof_system/verifier/verifier.cpp verify_proof() for the native implementation,
 * which includes detailed comments.
 */
template <typename Curve, typename program_settings>
aggregation_state<Curve> verify_proof_(typename Curve::Builder* context,
                                       std::shared_ptr<verification_key<Curve>> key,
                                       Transcript<typename Curve::Builder>& transcript,
                                       const aggregation_state<Curve> previous_output = aggregation_state<Curve>())
{
    using fr_ct = typename Curve::fr_ct;
    using fq_ct = typename Curve::fq_ct;
    using g1_ct = typename Curve::g1_ct;
    using Builder = typename Curve::Builder;

    key->program_width = program_settings::program_width;

    std::map<std::string, g1_ct> kate_g1_elements;
    std::map<std::string, fr_ct> kate_fr_elements_at_zeta;
    std::map<std::string, fr_ct> kate_fr_elements_at_zeta_large;
    std::map<std::string, fr_ct> kate_fr_elements_at_zeta_omega;

    const auto PI_Z = transcript.get_circuit_group_element("PI_Z");
    const auto PI_Z_OMEGA = transcript.get_circuit_group_element("PI_Z_OMEGA");

    field_t circuit_size = key->n;
    field_t public_input_size = key->num_public_inputs;

    transcript.add_field_element("circuit_size", circuit_size);
    transcript.add_field_element("public_input_size", public_input_size);

    transcript.apply_fiat_shamir("init");
    transcript.apply_fiat_shamir("eta");
    transcript.apply_fiat_shamir("beta");
    transcript.apply_fiat_shamir("alpha");
    transcript.apply_fiat_shamir("z");

    fr_ct alpha = transcript.get_challenge_field_element("alpha");
    fr_ct zeta = transcript.get_challenge_field_element("z");

    key->z_pow_n = zeta.pow(key->domain.domain);

    lagrange_evaluations<Builder> lagrange_evals = get_lagrange_evaluations<Curve>(zeta, key->domain);

    // reconstruct evaluation of quotient polynomial from prover messages

    fr_ct quotient_numerator_eval = fr_ct(0);
    program_settings::compute_quotient_evaluation_contribution(key.get(), alpha, transcript, quotient_numerator_eval);

    fr_ct t_eval = quotient_numerator_eval / lagrange_evals.vanishing_poly;
    transcript.add_field_element("t", t_eval);

    transcript.apply_fiat_shamir("nu");
    transcript.apply_fiat_shamir("separator");

    fr_ct u = transcript.get_challenge_field_element("separator", 0);

    fr_ct batch_opening_scalar;

    populate_kate_element_map<Curve, Transcript<Builder>, program_settings>(context,
                                                                            key.get(),
                                                                            transcript,
                                                                            kate_g1_elements,
                                                                            kate_fr_elements_at_zeta,
                                                                            kate_fr_elements_at_zeta_large,
                                                                            kate_fr_elements_at_zeta_omega,
                                                                            batch_opening_scalar);

    std::vector<fr_ct> double_opening_scalars;
    std::vector<g1_ct> double_opening_elements;
    std::vector<fr_ct> opening_scalars;
    std::vector<g1_ct> opening_elements;
    std::vector<fr_ct> big_opening_scalars;
    std::vector<g1_ct> big_opening_elements;
    std::vector<g1_ct> elements_to_add;

    for (const auto& [label, fr_value] : kate_fr_elements_at_zeta) {
        const auto& g1_value = kate_g1_elements[label];
        if (fr_value.get_value() == 0 && fr_value.witness_index != IS_CONSTANT) {
            std::cerr << "bad scalar zero at " << label << std::endl;
        }
        if (fr_value.get_value() == 0 && fr_value.witness_index == IS_CONSTANT) {
            std::cerr << "scalar zero at " << label << std::endl;
            continue;
        }

        if (fr_value.get_value() == 1 && fr_value.witness_index == IS_CONSTANT) {
            elements_to_add.emplace_back(g1_value);
            continue;
        }
        opening_scalars.emplace_back(fr_value);
        opening_elements.emplace_back(g1_value);
    }

    for (const auto& [label, fr_value] : kate_fr_elements_at_zeta_large) {
        const auto& g1_value = kate_g1_elements[label];
        if (fr_value.get_value() == 0 && fr_value.witness_index != IS_CONSTANT) {
            std::cerr << "bad scalar zero at " << label << std::endl;
        }
        if (fr_value.get_value() == 0 && fr_value.witness_index == IS_CONSTANT) {
            std::cerr << "scalar zero at " << label << std::endl;
            continue;
        }

        if (fr_value.get_value() == 1 && fr_value.witness_index == IS_CONSTANT) {
            elements_to_add.emplace_back(g1_value);
            continue;
        }
        big_opening_scalars.emplace_back(fr_value);
        big_opening_elements.emplace_back(g1_value);
    }

    for (const auto& [label, fr_value] : kate_fr_elements_at_zeta_omega) {
        const auto& g1_value = kate_g1_elements[label];
        double_opening_scalars.emplace_back(fr_value);
        double_opening_elements.emplace_back(g1_value);
    }

    const auto double_opening_result = g1_ct::batch_mul(double_opening_elements, double_opening_scalars, 128);

    opening_elements.emplace_back(double_opening_result);
    opening_scalars.emplace_back(u);

    std::vector<g1_ct> rhs_elements;
    std::vector<fr_ct> rhs_scalars;

    rhs_elements.push_back(PI_Z_OMEGA);
    rhs_scalars.push_back(u);

    if (previous_output.has_data) {
        fr_ct random_separator = transcript.get_challenge_field_element("separator", 1);

        opening_elements.push_back(previous_output.P0);
        opening_scalars.push_back(random_separator);

        rhs_elements.push_back((-(previous_output.P1)));
        rhs_scalars.push_back(random_separator);
    }

    /**
     * N.B. if this key contains a recursive proof, then ALL potential verification keys being verified by the outer
     *circuit must ALSO contain a recursive proof (this is not a concern if the key is being generated from circuit
     *constants). In addition the location in the public inputs of the recurisve outputs must be the same! i.e. this
     *code path should be used with extreme caution if the verification key is not being generated from circuit
     *constants
     **/
    if (key->contains_recursive_proof) {
        const auto public_inputs = transcript.get_field_element_vector("public_inputs");
        const auto recover_fq_from_public_inputs =
            [&public_inputs](const size_t idx0, const size_t idx1, const size_t idx2, const size_t idx3) {
                const fr_ct l0 = public_inputs[idx0];
                const fr_ct l1 = public_inputs[idx1];
                const fr_ct l2 = public_inputs[idx2];
                const fr_ct l3 = public_inputs[idx3];
                l0.create_range_constraint(fq_ct::NUM_LIMB_BITS, "l0");
                l1.create_range_constraint(fq_ct::NUM_LIMB_BITS, "l1");
                l2.create_range_constraint(fq_ct::NUM_LIMB_BITS, "l2");
                l3.create_range_constraint(fq_ct::NUM_LAST_LIMB_BITS, "l3");
                return fq_ct(l0, l1, l2, l3, false);
            };

        fr_ct recursion_separator_challenge = transcript.get_challenge_field_element("separator", 2);

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

        opening_elements.push_back(g1_ct(x0, y0));
        opening_scalars.push_back(recursion_separator_challenge);

        rhs_elements.push_back((-g1_ct(x1, y1)));
        rhs_scalars.push_back(recursion_separator_challenge);
    }

    auto opening_result = g1_ct::template bn254_endo_batch_mul_with_generator(
        big_opening_elements, big_opening_scalars, opening_elements, opening_scalars, batch_opening_scalar, 128);

    opening_result = opening_result + double_opening_result;
    for (const auto& to_add : elements_to_add) {
        opening_result = opening_result + to_add;
    }

    g1_ct rhs = g1_ct::template wnaf_batch_mul<128>(rhs_elements, rhs_scalars);

    rhs = (-rhs) - PI_Z;

    // TODO(zac: remove this once a3-packages has migrated to calling `assign_object_to_proof_outputs`)
    std::vector<uint32_t> proof_witness_indices = {
        opening_result.x.binary_basis_limbs[0].element.normalize().witness_index,
        opening_result.x.binary_basis_limbs[1].element.normalize().witness_index,
        opening_result.x.binary_basis_limbs[2].element.normalize().witness_index,
        opening_result.x.binary_basis_limbs[3].element.normalize().witness_index,
        opening_result.y.binary_basis_limbs[0].element.normalize().witness_index,
        opening_result.y.binary_basis_limbs[1].element.normalize().witness_index,
        opening_result.y.binary_basis_limbs[2].element.normalize().witness_index,
        opening_result.y.binary_basis_limbs[3].element.normalize().witness_index,
        rhs.x.binary_basis_limbs[0].element.normalize().witness_index,
        rhs.x.binary_basis_limbs[1].element.normalize().witness_index,
        rhs.x.binary_basis_limbs[2].element.normalize().witness_index,
        rhs.x.binary_basis_limbs[3].element.normalize().witness_index,
        rhs.y.binary_basis_limbs[0].element.normalize().witness_index,
        rhs.y.binary_basis_limbs[1].element.normalize().witness_index,
        rhs.y.binary_basis_limbs[2].element.normalize().witness_index,
        rhs.y.binary_basis_limbs[3].element.normalize().witness_index,
    };
    auto result = aggregation_state<Curve>{
        opening_result, rhs, transcript.get_field_element_vector("public_inputs"), proof_witness_indices, true
    };
    return result;
}

template <typename Flavor>
aggregation_state<bn254<typename Flavor::CircuitBuilder>> verify_proof(
    typename Flavor::CircuitBuilder* context,
    std::shared_ptr<verification_key<bn254<typename Flavor::CircuitBuilder>>> key,
    const plonk::proof& proof,
    const aggregation_state<bn254<typename Flavor::CircuitBuilder>> previous_output =
        aggregation_state<bn254<typename Flavor::CircuitBuilder>>())
{
    // TODO(Cody): Be sure this is kosher
    const auto manifest =
        Flavor::create_manifest(static_cast<size_t>(key->num_public_inputs.get_value().from_montgomery_form().data[0]));
    return verify_proof<bn254<typename Flavor::CircuitBuilder>,
                        recursion::recursive_ultra_verifier_settings<stdlib::bn254<typename Flavor::CircuitBuilder>>>(
        context, key, manifest, proof, previous_output);
}

} // namespace recursion
} // namespace stdlib
} // namespace proof_system::plonk
