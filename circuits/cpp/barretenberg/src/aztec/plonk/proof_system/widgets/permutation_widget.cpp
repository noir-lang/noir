#include "permutation_widget.hpp"
#include "../proving_key/proving_key.hpp"
#include <plonk/transcript/transcript.hpp>
#include <polynomials/iterate_over_domain.hpp>

using namespace barretenberg;

namespace waffle {
template <size_t program_width>
ProverPermutationWidget<program_width>::ProverPermutationWidget(proving_key* input_key, program_witness* input_witness)
    : ProverBaseWidget(input_key, input_witness)
{}

template <size_t program_width>
ProverPermutationWidget<program_width>::ProverPermutationWidget(const ProverPermutationWidget& other)
    : ProverBaseWidget(other)
{}

template <size_t program_width>
ProverPermutationWidget<program_width>::ProverPermutationWidget(ProverPermutationWidget&& other)
    : ProverBaseWidget(other)
{}

template <size_t program_width>
ProverPermutationWidget<program_width>& ProverPermutationWidget<program_width>::operator=(
    const ProverPermutationWidget& other)
{
    ProverBaseWidget::operator=(other);
    return *this;
}

template <size_t program_width>
ProverPermutationWidget<program_width>& ProverPermutationWidget<program_width>::operator=(
    ProverPermutationWidget&& other)
{
    ProverBaseWidget::operator=(other);
    return *this;
}

template <size_t program_width>
void ProverPermutationWidget<program_width>::compute_round_commitments(const transcript::Transcript&, const size_t)
{}

template <size_t program_width>
fr ProverPermutationWidget<program_width>::compute_quotient_contribution(const fr& alpha_base,
                                                                         const transcript::Transcript&)
{
    return alpha_base;
}

template <size_t program_width>
fr ProverPermutationWidget<program_width>::compute_linear_contribution(const fr& alpha_base,
                                                                       const transcript::Transcript&,
                                                                       polynomial&)
{

    return alpha_base;
}

template <size_t program_width>
size_t ProverPermutationWidget<program_width>::compute_opening_poly_contribution(
    const size_t nu_index, const transcript::Transcript&, barretenberg::fr*, barretenberg::fr*, const bool)
{
    return nu_index;
}

template <size_t program_width>
void ProverPermutationWidget<program_width>::compute_transcript_elements(transcript::Transcript&, const bool)
{
    return;
}

template class ProverPermutationWidget<3>;
template class ProverPermutationWidget<4>;

// ###

template <typename Field, typename Group, typename Transcript>
VerifierPermutationWidget<Field, Group, Transcript>::VerifierPermutationWidget()
{}

template <typename Field, typename Group, typename Transcript>
Field VerifierPermutationWidget<Field, Group, Transcript>::compute_quotient_evaluation_contribution(
    verification_key*, const Field& alpha_base, const Transcript&, Field&, const bool)
{
    return alpha_base;
}

template <typename Field, typename Group, typename Transcript>
size_t VerifierPermutationWidget<Field, Group, Transcript>::compute_batch_evaluation_contribution(
    verification_key*, Field&, const size_t nu_index, const Transcript&, const bool)
{
    return nu_index;
};

template <typename Field, typename Group, typename Transcript>
VerifierBaseWidget::challenge_coefficients<Field> VerifierPermutationWidget<Field, Group, Transcript>::
    append_scalar_multiplication_inputs(verification_key*,
                                        const VerifierBaseWidget::challenge_coefficients<Field>& challenge,
                                        const Transcript&,
                                        std::vector<Group>&,
                                        std::vector<Field>&,
                                        const bool)
{
    return challenge;
}

template class VerifierPermutationWidget<barretenberg::fr,
                                         barretenberg::g1::affine_element,
                                         transcript::StandardTranscript>;

} // namespace waffle