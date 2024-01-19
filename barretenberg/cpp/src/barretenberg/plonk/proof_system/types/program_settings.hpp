#pragma once

#include <cstdint>

#include "../widgets/random_widgets/permutation_widget.hpp"
#include "../widgets/random_widgets/plookup_widget.hpp"
#include "../widgets/random_widgets/random_widget.hpp"
#include "../widgets/transition_widgets/arithmetic_widget.hpp"
#include "../widgets/transition_widgets/elliptic_widget.hpp"
#include "../widgets/transition_widgets/genperm_sort_widget.hpp"
#include "../widgets/transition_widgets/plookup_arithmetic_widget.hpp"
#include "../widgets/transition_widgets/plookup_auxiliary_widget.hpp"
#include "./prover_settings.hpp"
#include "barretenberg/plonk/transcript/transcript.hpp"

namespace bb::plonk {

class standard_verifier_settings : public standard_settings {
  public:
    typedef bb::fr fr;
    typedef bb::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierArithmeticWidget<fr, g1::affine_element, Transcript, standard_settings> ArithmeticWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;

    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr bool idpolys = false;

    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const Transcript& transcript,
                                                  std::map<std::string, fr>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(key, alpha_base, transcript);

        return ArithmeticWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);
    }

    static bb::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                           const bb::fr& alpha_base,
                                                           const transcript::StandardTranscript& transcript,
                                                           bb::fr& quotient_numerator_eval)
    {
        auto updated_alpha_base =
            VerifierPermutationWidget<bb::fr, bb::g1::affine_element, transcript::StandardTranscript>::
                compute_quotient_evaluation_contribution(key, alpha_base, transcript, quotient_numerator_eval);

        return ArithmeticWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, quotient_numerator_eval);
    }
};

class ultra_verifier_settings : public ultra_settings {
  public:
    typedef bb::fr fr;
    typedef bb::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierPlookupArithmeticWidget<fr, g1::affine_element, Transcript, ultra_settings> PlookupArithmeticWidget;
    typedef VerifierGenPermSortWidget<fr, g1::affine_element, Transcript, ultra_settings> GenPermSortWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;
    typedef VerifierPlookupWidget<fr, g1::affine_element, Transcript> PlookupWidget;
    typedef VerifierEllipticWidget<fr, g1::affine_element, Transcript, ultra_settings> EllipticWidget;
    typedef VerifierPlookupAuxiliaryWidget<fr, g1::affine_element, Transcript, ultra_settings> PlookupAuxiliaryWidget;

    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
    static constexpr bool idpolys = true;

    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const Transcript& transcript,
                                                  std::map<std::string, bb::fr>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(key, alpha_base, transcript);
        updated_alpha = PlookupWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);
        updated_alpha =
            PlookupArithmeticWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);
        updated_alpha = GenPermSortWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);
        updated_alpha = EllipticWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);
        updated_alpha =
            PlookupAuxiliaryWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);

        return updated_alpha;
    }

    static bb::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                           const bb::fr& alpha_base,
                                                           const Transcript& transcript,
                                                           bb::fr& quotient_numerator_eval)
    {
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, quotient_numerator_eval, idpolys);
        updated_alpha_base = PlookupWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, quotient_numerator_eval);
        updated_alpha_base = PlookupArithmeticWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, quotient_numerator_eval);
        updated_alpha_base = GenPermSortWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, quotient_numerator_eval);
        updated_alpha_base = EllipticWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, quotient_numerator_eval);
        updated_alpha_base = PlookupAuxiliaryWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, quotient_numerator_eval);

        return updated_alpha_base;
    }
};

// Only needed because ultra-to-standard recursion requires us to use a Pedersen hash which is common to both Ultra &
// Standard plonk i.e. the non-ultra version.
class ultra_to_standard_verifier_settings : public ultra_verifier_settings {
  public:
    typedef VerifierPlookupArithmeticWidget<fr, g1::affine_element, Transcript, ultra_to_standard_settings>
        PlookupArithmeticWidget;
    typedef VerifierGenPermSortWidget<fr, g1::affine_element, Transcript, ultra_to_standard_settings> GenPermSortWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;
    typedef VerifierPlookupWidget<fr, g1::affine_element, Transcript> PlookupWidget;
    typedef VerifierEllipticWidget<fr, g1::affine_element, Transcript, ultra_to_standard_settings> EllipticWidget;
    typedef VerifierPlookupAuxiliaryWidget<fr, g1::affine_element, Transcript, ultra_to_standard_settings>
        PlookupAuxiliaryWidget;

    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
};

// This is neededed for the Noir backend. The ultra verifier contract uses 32-byte challenges generated with Keccak256.
class ultra_with_keccak_verifier_settings : public ultra_verifier_settings {
  public:
    typedef VerifierPlookupArithmeticWidget<fr, g1::affine_element, Transcript, ultra_with_keccak_settings>
        PlookupArithmeticWidget;
    typedef VerifierGenPermSortWidget<fr, g1::affine_element, Transcript, ultra_with_keccak_settings> GenPermSortWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;
    typedef VerifierPlookupWidget<fr, g1::affine_element, Transcript> PlookupWidget;
    typedef VerifierEllipticWidget<fr, g1::affine_element, Transcript, ultra_with_keccak_settings> EllipticWidget;
    typedef VerifierPlookupAuxiliaryWidget<fr, g1::affine_element, Transcript, ultra_with_keccak_settings>
        PlookupAuxiliaryWidget;

    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
};
} // namespace bb::plonk
