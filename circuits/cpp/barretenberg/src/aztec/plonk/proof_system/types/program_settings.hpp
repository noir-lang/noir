#pragma once

#include <cstdint>

#include "../widgets/base_widget.hpp"
#include "../widgets/arithmetic_widget.hpp"
#include "../widgets/mimc_widget.hpp"
#include "../widgets/turbo_arithmetic_widget.hpp"
#include "../widgets/turbo_fixed_base_widget.hpp"
#include "../widgets/turbo_logic_widget.hpp"
#include "../widgets/turbo_range_widget.hpp"

namespace waffle {
class settings_base {
  public:
    static constexpr bool requires_shifted_wire(const uint64_t wire_shift_settings, const uint64_t wire_index)
    {
        return (((wire_shift_settings >> (wire_index)) & 1UL) == 1UL);
    }
};

class standard_settings : public settings_base {
  public:
    static constexpr size_t program_width = 3;
    static constexpr uint64_t wire_shift_settings = 0b0100;
    static constexpr bool uses_quotient_mid = true;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
};

class turbo_settings : public settings_base {
  public:
    static constexpr size_t program_width = 4;
    static constexpr uint64_t wire_shift_settings = 0b1111;
    static constexpr bool uses_quotient_mid = false;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
};

class standard_verifier_settings : public standard_settings {
  public:
    static VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {
        return VerifierArithmeticWidget::append_scalar_multiplication_inputs(
            key, challenge, transcript, points, scalars);
    }

    static barretenberg::fr compute_batch_evaluation_contribution(verification_key* key,
                                                                  barretenberg::fr& batch_eval,
                                                                  const barretenberg::fr& nu_base,
                                                                  const transcript::Transcript& transcript)
    {
        return VerifierArithmeticWidget::compute_batch_evaluation_contribution(key, batch_eval, nu_base, transcript);
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::Transcript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        return VerifierArithmeticWidget::compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval);
    }
};

class mimc_verifier_settings : public standard_settings {
  public:
    static VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {

        VerifierBaseWidget::challenge_coefficients result =
            VerifierArithmeticWidget::append_scalar_multiplication_inputs(key, challenge, transcript, points, scalars);
        result = VerifierMiMCWidget::append_scalar_multiplication_inputs(key, result, transcript, points, scalars);
        return result;
    }

    static barretenberg::fr compute_batch_evaluation_contribution(verification_key* key,
                                                                  barretenberg::fr& batch_eval,
                                                                  const barretenberg::fr& nu_base,
                                                                  const transcript::Transcript& transcript)
    {
        barretenberg::fr updated_nu_base =
            VerifierArithmeticWidget::compute_batch_evaluation_contribution(key, batch_eval, nu_base, transcript);
        updated_nu_base =
            VerifierMiMCWidget::compute_batch_evaluation_contribution(key, batch_eval, updated_nu_base, transcript);
        return updated_nu_base;
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::Transcript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        barretenberg::fr updated_alpha_base =
            VerifierArithmeticWidget::compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval);
        updated_alpha_base =
            VerifierMiMCWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval);
        return updated_alpha_base;
    }
};

class turbo_verifier_settings : public turbo_settings {
  public:
    static VerifierBaseWidget::challenge_coefficients append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients& challenge,
        const transcript::Transcript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {
        VerifierBaseWidget::challenge_coefficients result =
            VerifierTurboFixedBaseWidget::append_scalar_multiplication_inputs(
                key, challenge, transcript, points, scalars);
        result =
            VerifierTurboRangeWidget::append_scalar_multiplication_inputs(key, result, transcript, points, scalars);
        result =
            VerifierTurboLogicWidget::append_scalar_multiplication_inputs(key, result, transcript, points, scalars);
        return result;
    }

    static barretenberg::fr compute_batch_evaluation_contribution(verification_key* key,
                                                                  barretenberg::fr& batch_eval,
                                                                  const barretenberg::fr& nu_base,
                                                                  const transcript::Transcript& transcript)
    {
        barretenberg::fr updated_nu_base =
            VerifierTurboFixedBaseWidget::compute_batch_evaluation_contribution(key, batch_eval, nu_base, transcript);
        updated_nu_base = VerifierTurboRangeWidget::compute_batch_evaluation_contribution(
            key, batch_eval, updated_nu_base, transcript);
        updated_nu_base = VerifierTurboLogicWidget::compute_batch_evaluation_contribution(
            key, batch_eval, updated_nu_base, transcript);

        return updated_nu_base;
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::Transcript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        barretenberg::fr updated_alpha_base =
            VerifierTurboFixedBaseWidget::compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval);
        updated_alpha_base = VerifierTurboRangeWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval);
        updated_alpha_base = VerifierTurboLogicWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval);
        return updated_alpha_base;
    }
};
} // namespace waffle