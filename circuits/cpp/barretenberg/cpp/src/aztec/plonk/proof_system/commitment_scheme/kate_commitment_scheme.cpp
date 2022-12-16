#include <common/throw_or_abort.hpp>
#include "kate_commitment_scheme.hpp"
#include "../../../polynomials/polynomial_arithmetic.hpp"

namespace waffle {

// Constructors for KateCommitmentScheme
template <typename settings>
KateCommitmentScheme<settings>::KateCommitmentScheme()
    : CommitmentScheme::CommitmentScheme()
{}

template <typename settings>
void KateCommitmentScheme<settings>::commit(fr* coefficients,
                                            std::string commitment_tag,
                                            fr item_constant,
                                            work_queue& queue)
{
    queue.add_to_queue({
        .work_type = work_queue::WorkType::SCALAR_MULTIPLICATION,
        .mul_scalars = coefficients,
        .tag = commitment_tag,
        .constant = item_constant,
        .index = 0,
    });
}

template <typename settings>
void KateCommitmentScheme<settings>::compute_opening_polynomial(const fr* src,
                                                                fr* dest,
                                                                const fr& z_point,
                                                                const size_t n)
{
    // open({cm_i}, {cm'_i}, {z, z'}, {s_i, s'_i})

    // if `coeffs` represents F(X), we want to compute W(X)
    // where W(X) = F(X) - F(z) / (X - z)
    // i.e. divide by the degree-1 polynomial [-z, 1]

    // We assume that the commitment is well-formed and that there is no remainder term.
    // Under these conditions we can perform this polynomial division in linear time with good constants.
    // Note that the opening polynomial always has (n+1) coefficients for Standard/Turbo/Ultra due to
    // the blinding of the quotient polynomial parts.
    fr f = polynomial_arithmetic::evaluate(src, z_point, n + 1);

    // compute (1 / -z)
    fr divisor = -z_point.invert();

    // we're about to shove these coefficients into a pippenger multi-exponentiation routine, where we need
    // to convert out of montgomery form. So, we can use lazy reduction techniques here without triggering overflows
    dest[0] = src[0] - f;
    dest[0] *= divisor;
    for (size_t i = 1; i < n; ++i) {
        dest[i] = src[i] - dest[i - 1];
        dest[i] *= divisor;
    }
}

template <typename settings>
void KateCommitmentScheme<settings>::generic_batch_open(const fr* src,
                                                        fr* dest,
                                                        const size_t num_polynomials,
                                                        const fr* z_points,
                                                        const size_t num_z_points,
                                                        const fr* challenges,
                                                        const size_t n,
                                                        std::string* tags,
                                                        fr* item_constants,
                                                        work_queue& queue)
{
    // In this function, we compute the opening polynomials using Kate scheme for multiple input
    // polynomials with multiple evaluation points. The input polynomials are separated according
    // to the point at which they need to be opened at, viz.
    //
    // z_1 -> [F_{1,1},  F_{1,2},  F_{1, 3},  ...,  F_{1, m}]
    // z_2 -> [F_{2,1},  F_{2,2},  F_{2, 3},  ...,  F_{2, m}]
    // ...
    // z_t -> [F_{t,1},  F_{t,2},  F_{t, 3},  ...,  F_{t, m}]
    //
    // Note that the input polynomials are assumed to be stored in their coefficient forms
    // in a single array `src` in the same order as above. Polynomials which are to be opened at a
    // same point `z_i` are combined linearly using the powers of the challenge `γ_i = challenges[i]`.
    //
    // The output opened polynomials [W_{1},  W_{2}, ...,  W_{t}] are saved in the array `dest`.
    //             1
    // W_{i} = ---------- * \sum_{j=1}^{m} (γ_i)^{j-1} * [ F_{i,j}(X) - F_{i,j}(z_i) ]
    //           X - z_i
    //
    // P.S. This function isn't actually used anywhere in PLONK but was written as a generic batch
    // opening test case.

    // compute [-z, -z', ... ]
    fr divisors[num_z_points];
    for (size_t i = 0; i < num_z_points; ++i) {
        divisors[i] = -z_points[i];
    }
    fr::batch_invert(divisors, num_z_points);

    for (size_t i = 0; i < num_z_points; ++i) {
        fr challenge = challenges[i];
        fr divisor = divisors[i];
        size_t src_offset = (i * n * num_polynomials);
        size_t dest_offset = (i * n);

        // compute i-th linear combination polynomial
        // F_i(X) = \sum_{j = 1, 2, ..., num_poly} \gamma^{j - 1} * f_{i, j}(X)
        for (size_t k = 0; k < n; ++k) {
            fr coeff_sum = 0;
            fr challenge_pow = 1;
            for (size_t j = 0; j < num_polynomials; ++j) {
                coeff_sum += challenge_pow * src[src_offset + (j * n) + k];
                challenge_pow *= challenge;
            }
            dest[dest_offset + k] = coeff_sum;
        }

        // evaluation of the i-th linear combination polynomial F_i(X) at z
        fr d_i_eval = polynomial_arithmetic::evaluate(&dest[dest_offset], z_points[i], n);

        // compute coefficients of h_i(X) = (F_i(X) - F_i(z))/(X - z) as done in the previous function
        dest[dest_offset] -= d_i_eval;
        dest[dest_offset] *= divisor;
        for (size_t k = 1; k < n; ++k) {
            dest[dest_offset + k] -= dest[dest_offset + k - 1];
            dest[dest_offset + k] *= divisor;
        }

        // commit to the i-th opened polynomial
        KateCommitmentScheme::commit(&dest[dest_offset], tags[i], item_constants[i], queue);
    }
}

template <typename settings>
void KateCommitmentScheme<settings>::batch_open(const transcript::StandardTranscript& transcript,
                                                work_queue& queue,
                                                std::shared_ptr<proving_key> input_key)
{
    /*
    Compute batch opening polynomials according to the Kate commitment scheme.

    Step 1: Compute the polynomial F(X) s.t. W_{\zeta}(X) = (F(X) - F(\zeta))/(X - \zeta) defined in round 5 of the
    PLONK paper. Step 2: Compute the polynomial z(X) s.t. W_{\zeta \omega}(X) = (z(X) - z(\zeta \omega))/(X -
    \zeta.\omega). Step 3: Compute coefficient form of W_{\zeta}(X) and W_{\zeta \omega}(X). Step 4: Commit to
    W_{\zeta}(X) and W_{\zeta \omega}(X).
    */
    std::vector<std::pair<fr*, fr>> opened_polynomials_at_zeta;
    std::vector<std::pair<fr*, fr>> opened_polynomials_at_zeta_omega;

    // Add the following tuples to the above data structures:
    //
    // [a(X), nu_1], [b(X), nu_2], [c(X), nu_3],
    // [S_{\sigma_1}(X), nu_4], [S_{\sigma_2}(X), nu_5],
    // [z(X), nu_6]
    //
    // Note that the challenges nu_1, ..., nu_6 depend on the label of the respective polynomial.

    // Add challenge-poly tuples for all polynomials in the manifest
    for (size_t i = 0; i < input_key->polynomial_manifest.size(); ++i) {
        const auto& info = input_key->polynomial_manifest[i];
        const std::string poly_label(info.polynomial_label);

        fr* poly = input_key->polynomial_cache.get(poly_label).get_coefficients();

        if (!info.is_linearised || !settings::use_linearisation) {
            const fr nu_challenge = transcript.get_challenge_field_element_from_map("nu", poly_label);
            opened_polynomials_at_zeta.push_back({ poly, nu_challenge });
        }
        if (info.requires_shifted_evaluation) {
            const auto nu_challenge = transcript.get_challenge_field_element_from_map("nu", poly_label + "_omega");
            opened_polynomials_at_zeta_omega.push_back({ poly, nu_challenge });
        }
    }

    const auto zeta = transcript.get_challenge_field_element("z");

    // Note: the opening poly W_\frak{z} is always size (n + 1) due to blinding
    // of the quotient polynomial
    polynomial opening_poly(input_key->n + 1, input_key->n + 1);
    polynomial shifted_opening_poly(input_key->n, input_key->n);

    const polynomial& linear_poly = input_key->polynomial_cache.get("linear_poly");

    if constexpr (!settings::use_linearisation) {
        // Add the tuples [t_{mid}(X), \zeta^{n}], [t_{high}(X), \zeta^{2n}]
        // Note: We don't need to include the t_{low}(X) since it is multiplied by 1 for combining with other witness
        // polynomials.
        //
        for (size_t i = 1; i < settings::program_width; ++i) {
            const size_t offset = i * input_key->small_domain.size;
            const fr scalar = zeta.pow(static_cast<uint64_t>(offset));
            opened_polynomials_at_zeta.push_back({ &input_key->quotient_polynomial_parts[i][0], scalar });
        }
    }

    // Add up things to get coefficients of opening polynomials.
    ITERATE_OVER_DOMAIN_START(input_key->small_domain);
    opening_poly[i] = settings::use_linearisation ? linear_poly[i] : input_key->quotient_polynomial_parts[0][i];
    for (const auto& [poly, challenge] : opened_polynomials_at_zeta) {
        opening_poly[i] += poly[i] * challenge;
    }
    shifted_opening_poly[i] = 0;
    for (const auto& [poly, challenge] : opened_polynomials_at_zeta_omega) {
        shifted_opening_poly[i] += poly[i] * challenge;
    }
    ITERATE_OVER_DOMAIN_END;

    // Adjust the (n + 1)th coefficient of t_{0,1,2}(X) or r(X) (Note: t_4 (Turbo/Ultra) has only n coefficients)
    if (!settings::use_linearisation) {
        opening_poly[input_key->n] = 0;
        const fr zeta_pow_n = zeta.pow(static_cast<uint64_t>(input_key->n));

        const size_t num_deg_n_poly =
            settings::program_width == 3 ? settings::program_width : settings::program_width - 1;
        fr scalar_mult = 1;
        for (size_t i = 0; i < num_deg_n_poly; i++) {
            opening_poly[input_key->n] += input_key->quotient_polynomial_parts[i][input_key->n] * scalar_mult;
            scalar_mult *= zeta_pow_n;
        }
    } else {
        opening_poly[input_key->n] = linear_poly[input_key->n];
    }

    // compute the shifted evaluation point \frak{z}*omega
    const auto zeta_omega = zeta * input_key->small_domain.root;

    // Compute the W_{\zeta}(X) and W_{\zeta \omega}(X) polynomials
    KateCommitmentScheme::compute_opening_polynomial(&opening_poly[0], &opening_poly[0], zeta, input_key->n);
    KateCommitmentScheme::compute_opening_polynomial(
        &shifted_opening_poly[0], &shifted_opening_poly[0], zeta_omega, input_key->n);

    input_key->polynomial_cache.put("opening_poly", std::move(opening_poly));
    input_key->polynomial_cache.put("shifted_opening_poly", std::move(shifted_opening_poly));

    // Commit to the opening and shifted opening polynomials
    KateCommitmentScheme::commit(
        input_key->polynomial_cache.get("opening_poly").get_coefficients(), "PI_Z", fr(0), queue);
    KateCommitmentScheme::commit(
        input_key->polynomial_cache.get("shifted_opening_poly").get_coefficients(), "PI_Z_OMEGA", fr(0), queue);
}

template <typename settings>
void KateCommitmentScheme<settings>::batch_verify(const transcript::StandardTranscript& transcript,
                                                  std::map<std::string, g1::affine_element>& kate_g1_elements,
                                                  std::map<std::string, fr>& kate_fr_elements,
                                                  std::shared_ptr<verification_key> input_key,
                                                  const barretenberg::fr& r_0)
{
    // Compute batch evaluation commitment [F]_1
    // In this method, we accumulate scalars and corresponding group elements for the multi-scalar
    // multiplication required in the steps 10 and 11 of the verifier in the PLONK paper.
    //
    // Step 10: Compute batch opening commitment [F]_1
    //          [F]  :=  [t_{low}]_1 + \zeta^{n}.[tmid]1 + \zeta^{2n}.[t_{high}]_1
    //                   + [D]_1 + \nu_{a}.[a]_1 + \nu_{b}.[b]_1 + \nu_{c}.[c]_1
    //                   + \nu_{\sigma1}.[s_{\sigma_1}]1 + \nu_{\sigma2}.[s_{\sigma_2}]1
    //
    // We do not compute [D]_1 term in this method as the information required to compute [D]_1
    // in inadequate as far as this KateCommitmentScheme class is concerned.
    //
    // Step 11: Compute batch evaluation commitment [E]_1
    //          [E]_1  :=  (t_eval + \nu_{r}.r_eval + \nu_{a}.a_eval + \nu_{b}.b_eval
    //                      \nu_{c}.c_eval + \nu_{\sigma1}.sigma1_eval + \nu_{\sigma2}.sigma2_eval +
    //                      nu_z_omega.separator.z_eval_omega) . [1]_1
    //
    // Note that we do not actually compute the scalar multiplications but just accumulate the scalars
    // and the group elements in different vectors.
    //

    fr batch_eval(0);
    const auto& polynomial_manifest = input_key->polynomial_manifest;
    for (size_t i = 0; i < input_key->polynomial_manifest.size(); ++i) {
        const auto& item = polynomial_manifest[i];
        const std::string label(item.commitment_label);
        const std::string poly_label(item.polynomial_label);
        switch (item.source) {
        case PolynomialSource::WITNESS: {
            // add [a]_1, [b]_1, [c]_1 to the group elements' vector
            const auto element = transcript.get_group_element(label);
            // rule out bad points and points at infinity (just to be on the safe side. all-zero witnesses won't be
            // zero-knowledge!)
            if (!element.on_curve() || element.is_point_at_infinity()) {
                throw_or_abort("polynomial commitment to witness is not a valid point.");
            }
            kate_g1_elements.insert({ label, element });
            break;
        }
        case PolynomialSource::SELECTOR: {
            // add [qL]_1, [qR]_1, [qM]_1, [qC]_1, [qO]_1 to the group elements' vector
            const auto element = input_key->constraint_selectors.at(label);
            // selectors can be all zeros so infinity point is valid
            if (!element.on_curve()) {
                throw_or_abort("polynomial commitment to selector is not a valid point.");
            }
            kate_g1_elements.insert({ label, element });
            break;
        }
        case PolynomialSource::PERMUTATION: {

            // add [\sigma_1]_1, [\sigma_2]_1, [\sigma_3]_1 to the group elements' vector
            const auto element = input_key->permutation_selectors.at(label);
            // selectors can be all zeros so infinity point is valid
            if (!element.on_curve()) {
                throw_or_abort("polynomial commitment to permutation selector is not a valid point.");
            }
            kate_g1_elements.insert({ label, element });
            break;
        }
        }

        // We iterate over the polynomials in polynomial_manifest to add their commitments,
        // their scalar multiplicands and their evaluations in the respective vector maps.
        //
        // If we have a polynomial such that `is_linearised` and `use_linearisation` is true
        // and `requires_shifted_evaluation` being false, then the polynomial would either be
        // a selector polynomial or a permutation polynomial. In that case, we do not want them
        // to be included in the batch evaluation or the part of the batch opening commitment.
        //
        bool has_evaluation = !item.is_linearised || !settings::use_linearisation;
        bool has_shifted_evaluation = item.requires_shifted_evaluation;

        fr kate_fr_scalar(0);
        if (has_shifted_evaluation) {

            // compute scalar additively for the batch opening commitment [F]_1
            const auto challenge = transcript.get_challenge_field_element_from_map("nu", poly_label + "_omega");
            const auto separator_challenge = transcript.get_challenge_field_element("separator", 0);
            kate_fr_scalar += (separator_challenge * challenge);

            // compute the batch evaluation scalar additively for the batch evaluation commitment [E]_1
            const auto poly_at_zeta_omega = transcript.get_field_element(poly_label + "_omega");
            batch_eval += separator_challenge * challenge * poly_at_zeta_omega;
        }
        if (has_evaluation) {

            // compute scalar additively for the batch opening commitment [F]_1
            const auto challenge = transcript.get_challenge_field_element_from_map("nu", poly_label);
            kate_fr_scalar += challenge;

            // compute the batch evaluation scalar additively for the batch evaluation commitment [E]_1
            const auto poly_at_zeta = transcript.get_field_element(poly_label);
            batch_eval += challenge * poly_at_zeta;
        }
        kate_fr_elements.insert({ label, kate_fr_scalar });
    }

    const auto zeta = transcript.get_challenge_field_element("z");
    // t_eval is not a part of the transcript when use_linearisation is true. That is why
    // we need to hardcode quotient_challenge
    barretenberg::fr quotient_challenge =
        (settings::use_linearisation) ? 1 : transcript.get_challenge_field_element_from_map("nu", "t");
    barretenberg::polynomial_arithmetic::lagrange_evaluations lagrange_evals =
        barretenberg::polynomial_arithmetic::get_lagrange_evaluations(zeta, input_key->domain);

    // append the commitments to the parts of quotient polynomial and their scalar multiplicands
    fr z_pow_n = zeta.pow(input_key->n);
    fr z_power = settings::use_linearisation ? -lagrange_evals.vanishing_poly : 1;
    for (size_t i = 0; i < settings::program_width; ++i) {
        std::string quotient_label = "T_" + std::to_string(i + 1);
        const auto element = transcript.get_group_element(quotient_label);

        kate_g1_elements.insert({ quotient_label, element });
        kate_fr_elements.insert({ quotient_label, quotient_challenge * z_power });
        z_power *= z_pow_n;
    }

    if (settings::use_linearisation) {
        // add the r0 term to batch_eval
        batch_eval -= r_0;
    } else {
        // add the quotient eval t_eval term to batch evaluation
        const auto quotient_eval = transcript.get_field_element("t");
        batch_eval += (quotient_eval * quotient_challenge);
    }

    // append batch evaluation in the scalar element vector map
    kate_g1_elements.insert({ "BATCH_EVALUATION", g1::affine_one });
    kate_fr_elements.insert({ "BATCH_EVALUATION", -batch_eval });
}

template <typename settings>
void KateCommitmentScheme<settings>::add_opening_evaluations_to_transcript(transcript::StandardTranscript& transcript,
                                                                           std::shared_ptr<proving_key> input_key,
                                                                           bool in_lagrange_form)
{
    // In this function, we compute the evaluations of the polynomials which would be a part of the
    // opening polynomial W_{zeta}(X), viz.
    //     1. a(X), b(X), c(X), S_{sigma1}(X), S_{sigma2}(X), t(X) at zeta
    //     2. z(X) at zeta.omega
    // We add these evaluations to the transcript, which would be used by the prover to compute linearisation
    // polynomial r(X) and the verifier would use them to compute the batch evaluation and partial opening
    // commitments. We refer to these as opening evaluations following the nomenclature in round 4 of the PLONK
    // paper.
    //
    // We also allow this evaluation computation for lagrange (evaluation) forms of polynomials instead of
    // the usual coefficient forms.
    //
    fr zeta = fr::serialize_from_buffer(transcript.get_challenge("z").begin());
    fr shifted_z = zeta * input_key->small_domain.root;
    size_t n = input_key->small_domain.size;

    // Add evaluations for all polynomials in the manifest
    for (size_t i = 0; i < input_key->polynomial_manifest.size(); ++i) {
        const auto& info = input_key->polynomial_manifest[i];
        const std::string poly_label(info.polynomial_label);

        fr* poly = input_key->polynomial_cache.get(poly_label).get_coefficients();

        fr poly_evaluation(0);
        if (!info.is_linearised || !settings::use_linearisation) {
            if (in_lagrange_form) {
                poly_evaluation =
                    polynomial_arithmetic::compute_barycentric_evaluation(poly, n, zeta, input_key->small_domain);
            } else {
                poly_evaluation = polynomial_arithmetic::evaluate(poly, zeta, n);
            }
            transcript.add_element(poly_label, poly_evaluation.to_buffer());
        }
        if (info.requires_shifted_evaluation) {
            if (in_lagrange_form) {
                poly_evaluation =
                    polynomial_arithmetic::compute_barycentric_evaluation(poly, n, zeta, input_key->small_domain);
            } else {
                poly_evaluation = polynomial_arithmetic::evaluate(poly, shifted_z, n);
            }
            transcript.add_element(poly_label + "_omega", poly_evaluation.to_buffer());
        }
    }
}

template class KateCommitmentScheme<unrolled_standard_settings>;
template class KateCommitmentScheme<unrolled_turbo_settings>;
template class KateCommitmentScheme<unrolled_ultra_settings>;
template class KateCommitmentScheme<unrolled_ultra_to_standard_settings>;
template class KateCommitmentScheme<standard_settings>;
template class KateCommitmentScheme<turbo_settings>;
template class KateCommitmentScheme<ultra_settings>;
} // namespace waffle