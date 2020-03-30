#pragma once
#include <ecc/curves/bn254/fr.hpp>
#include <plonk/transcript/transcript.hpp>

namespace waffle {
template <typename Field> struct plonk_linear_terms {
    Field z_1;
    Field sigma_last;
};

// This linearisation trick was originated from Mary Maller and the SONIC paper. When computing Kate commitments to the
// PLONK polynomials, we wish to find the minimum number of polynomial evaluations that the prover must send to the
// verifier. I.e. we want to find the minimum number of polynomial evaluations that are needed, so that the remaining
// polynomial evaluations can be expressed as a linear sum of polynomials. The verifier can derive the prover's
// commitment to this linear polynomial from the original commitments - the prover can provide an evaluation of this
// linear polynomial, instead of the evaluations of its consitutent polynomials. This shaves 6 field elements off of the
// proof size!
template <typename Field, typename Transcript, size_t program_width>
inline plonk_linear_terms<Field> compute_linear_terms(const Transcript& transcript, const Field& l_1)
{
    Field alpha = transcript.get_challenge_field_element("alpha");
    Field alpha_cubed = alpha.sqr() * alpha;
    Field beta = transcript.get_challenge_field_element("beta");
    Field gamma = transcript.get_challenge_field_element("beta", 1);
    Field z = transcript.get_challenge_field_element("z");
    Field z_beta = z * beta;

    std::array<Field, program_width> wire_evaluations;
    for (size_t i = 0; i < program_width; ++i) {
        wire_evaluations[i] = transcript.get_field_element("w_" + std::to_string(i + 1));
    }

    Field z_1_shifted_eval = transcript.get_field_element("z_omega");

    plonk_linear_terms<Field> result;

    Field T0;
    Field z_contribution = Field(1);
    for (size_t i = 0; i < program_width; ++i) {
        Field coset_generator = (i == 0) ? Field(1) : Field::coset_generator(i - 1);
        T0 = z_beta * coset_generator;
        T0 += wire_evaluations[i];
        T0 += gamma;
        z_contribution *= T0;
    }
    result.z_1 = z_contribution * alpha;
    T0 = l_1 * alpha_cubed;
    result.z_1 += T0;

    Field sigma_contribution = Field(1);
    for (size_t i = 0; i < program_width - 1; ++i) {
        Field permutation_evaluation = transcript.get_field_element("sigma_" + std::to_string(i + 1));
        T0 = permutation_evaluation * beta;
        T0 += wire_evaluations[i];
        T0 += gamma;
        sigma_contribution *= T0;
    }
    sigma_contribution *= z_1_shifted_eval;
    result.sigma_last = -(sigma_contribution * alpha);
    result.sigma_last *= beta;

    return result;
}
} // namespace waffle