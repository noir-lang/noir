#pragma once

#include <cstdint>

#include "../../transcript/transcript_wrappers.hpp"
#include "../widgets/arithmetic_widget.hpp"
#include "../widgets/base_widget.hpp"
#include "../widgets/mimc_widget.hpp"
#include "../widgets/turbo_arithmetic_widget.hpp"
#include "../widgets/turbo_fixed_base_widget.hpp"
#include "../widgets/turbo_logic_widget.hpp"
#include "../widgets/turbo_range_widget.hpp"
#include "../widgets/permutation_widget.hpp"

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
    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr size_t program_width = 3;
    static constexpr uint64_t wire_shift_settings = 0b0100;
    static constexpr bool uses_quotient_mid = true;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr bool use_linearisation = true;
};

class unrolled_standard_settings : public settings_base {
  public:
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::Blake2s;
    static constexpr size_t program_width = 3;
    static constexpr uint64_t wire_shift_settings = 0b0100;
    static constexpr bool uses_quotient_mid = true;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr bool use_linearisation = false;
};

class turbo_settings : public settings_base {
  public:
    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr size_t program_width = 4;
    static constexpr uint64_t wire_shift_settings = 0b1111;
    static constexpr bool uses_quotient_mid = false;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr bool use_linearisation = true;
};

class unrolled_turbo_settings : public settings_base {
  public:
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::Blake2s;
    static constexpr size_t program_width = 4;
    static constexpr uint64_t wire_shift_settings = 0b1111;
    static constexpr bool uses_quotient_mid = false;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr bool use_linearisation = false;
};

class standard_verifier_settings : public standard_settings {
  public:
    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr bool use_linearisation = true;
    static VerifierBaseWidget::challenge_coefficients<barretenberg::fr> append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients<barretenberg::fr>& challenge,
        const transcript::StandardTranscript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {
        auto updated_challenge = VerifierPermutationWidget<barretenberg::fr,
                                                           barretenberg::g1::affine_element,
                                                           transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, challenge, transcript, points, scalars, use_linearisation);

        updated_challenge.nu_index += 1;
        return VerifierArithmeticWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);
    }

    static size_t compute_batch_evaluation_contribution(verification_key* key,
                                                        barretenberg::fr& batch_eval,
                                                        const size_t nu_index,
                                                        const transcript::StandardTranscript& transcript)
    {
        auto updated_nu_index = VerifierPermutationWidget<barretenberg::fr,
                                                          barretenberg::g1::affine_element,
                                                          transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, nu_index, transcript, use_linearisation);
        ++updated_nu_index;
        return VerifierArithmeticWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::StandardTranscript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        auto updated_alpha_base = VerifierPermutationWidget<barretenberg::fr,
                                                            barretenberg::g1::affine_element,
                                                            transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = VerifierArithmeticWidget<barretenberg::fr,
                                                      barretenberg::g1::affine_element,
                                                      transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};

class unrolled_standard_verifier_settings : public standard_settings {
  public:
    static constexpr transcript::HashType hash_type = transcript::HashType::Blake2s;
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr bool use_linearisation = false;
    static VerifierBaseWidget::challenge_coefficients<barretenberg::fr> append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients<barretenberg::fr>& challenge,
        const transcript::StandardTranscript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {
        auto updated_challenge = VerifierPermutationWidget<barretenberg::fr,
                                                           barretenberg::g1::affine_element,
                                                           transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, challenge, transcript, points, scalars, use_linearisation);

        updated_challenge.nu_index += 1;

        return VerifierArithmeticWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);
    }

    static size_t compute_batch_evaluation_contribution(verification_key* key,
                                                        barretenberg::fr& batch_eval,
                                                        const size_t nu_index,
                                                        const transcript::StandardTranscript& transcript)
    {
        auto updated_nu_index = VerifierPermutationWidget<barretenberg::fr,
                                                          barretenberg::g1::affine_element,
                                                          transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, nu_index, transcript, use_linearisation);
        ++updated_nu_index;
        return VerifierArithmeticWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::StandardTranscript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        auto updated_alpha_base = VerifierPermutationWidget<barretenberg::fr,
                                                            barretenberg::g1::affine_element,
                                                            transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval, use_linearisation);

        return VerifierArithmeticWidget<barretenberg::fr,
                                        barretenberg::g1::affine_element,
                                        transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
    }
};

class mimc_verifier_settings : public standard_settings {
  public:
    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr bool use_linearisation = true;
    static VerifierBaseWidget::challenge_coefficients<barretenberg::fr> append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients<barretenberg::fr>& challenge,
        const transcript::StandardTranscript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {

        auto updated_challenge = VerifierPermutationWidget<barretenberg::fr,
                                                           barretenberg::g1::affine_element,
                                                           transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, challenge, transcript, points, scalars, use_linearisation);

        updated_challenge.nu_index += 1;

        updated_challenge = VerifierArithmeticWidget<barretenberg::fr,
                                                     barretenberg::g1::affine_element,
                                                     transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);
        updated_challenge =
            VerifierMiMCWidget<barretenberg::fr, barretenberg::g1::affine_element, transcript::StandardTranscript>::
                append_scalar_multiplication_inputs(
                    key, updated_challenge, transcript, points, scalars, use_linearisation);
        return updated_challenge;
    }

    static size_t compute_batch_evaluation_contribution(verification_key* key,
                                                        barretenberg::fr& batch_eval,
                                                        const size_t nu_index,
                                                        const transcript::StandardTranscript& transcript)
    {
        auto updated_nu_index = VerifierPermutationWidget<barretenberg::fr,
                                                          barretenberg::g1::affine_element,
                                                          transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, nu_index, transcript, use_linearisation);
        ++updated_nu_index;
        updated_nu_index = VerifierArithmeticWidget<barretenberg::fr,
                                                    barretenberg::g1::affine_element,
                                                    transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        updated_nu_index =
            VerifierMiMCWidget<barretenberg::fr, barretenberg::g1::affine_element, transcript::StandardTranscript>::
                compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        return updated_nu_index;
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::StandardTranscript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        auto updated_alpha_base = VerifierPermutationWidget<barretenberg::fr,
                                                            barretenberg::g1::affine_element,
                                                            transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval, use_linearisation);

        updated_alpha_base = VerifierArithmeticWidget<barretenberg::fr,
                                                      barretenberg::g1::affine_element,
                                                      transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base =
            VerifierMiMCWidget<barretenberg::fr, barretenberg::g1::affine_element, transcript::StandardTranscript>::
                compute_quotient_evaluation_contribution(
                    key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};

class turbo_verifier_settings : public turbo_settings {
  public:
    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr bool use_linearisation = true;
    static VerifierBaseWidget::challenge_coefficients<barretenberg::fr> append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients<barretenberg::fr>& challenge,
        const transcript::StandardTranscript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {
        auto updated_challenge = VerifierPermutationWidget<barretenberg::fr,
                                                           barretenberg::g1::affine_element,
                                                           transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, challenge, transcript, points, scalars, use_linearisation);

        updated_challenge.nu_index += 4;
        updated_challenge = VerifierTurboFixedBaseWidget<barretenberg::fr,
                                                         barretenberg::g1::affine_element,
                                                         transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);
        updated_challenge = VerifierTurboRangeWidget<barretenberg::fr,
                                                     barretenberg::g1::affine_element,
                                                     transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);
        updated_challenge = VerifierTurboLogicWidget<barretenberg::fr,
                                                     barretenberg::g1::affine_element,
                                                     transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);
        return updated_challenge;
    }

    static size_t compute_batch_evaluation_contribution(verification_key* key,
                                                        barretenberg::fr& batch_eval,
                                                        const size_t nu_index,
                                                        const transcript::StandardTranscript& transcript)
    {
        auto updated_nu_index = VerifierPermutationWidget<barretenberg::fr,
                                                          barretenberg::g1::affine_element,
                                                          transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, nu_index, transcript, use_linearisation);
        updated_nu_index += 4;
        updated_nu_index = VerifierTurboFixedBaseWidget<barretenberg::fr,
                                                        barretenberg::g1::affine_element,
                                                        transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        updated_nu_index = VerifierTurboRangeWidget<barretenberg::fr,
                                                    barretenberg::g1::affine_element,
                                                    transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        updated_nu_index = VerifierTurboLogicWidget<barretenberg::fr,
                                                    barretenberg::g1::affine_element,
                                                    transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);

        return updated_nu_index;
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::StandardTranscript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        auto updated_alpha_base = VerifierPermutationWidget<barretenberg::fr,
                                                            barretenberg::g1::affine_element,
                                                            transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval, use_linearisation);

        updated_alpha_base = VerifierTurboFixedBaseWidget<barretenberg::fr,
                                                          barretenberg::g1::affine_element,
                                                          transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = VerifierTurboRangeWidget<barretenberg::fr,
                                                      barretenberg::g1::affine_element,
                                                      transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = VerifierTurboLogicWidget<barretenberg::fr,
                                                      barretenberg::g1::affine_element,
                                                      transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};

class unrolled_turbo_verifier_settings : public unrolled_turbo_settings {
  public:
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::Blake2s;
    static constexpr bool use_linearisation = false;
    static VerifierBaseWidget::challenge_coefficients<barretenberg::fr> append_scalar_multiplication_inputs(
        verification_key* key,
        const VerifierBaseWidget::challenge_coefficients<barretenberg::fr>& challenge,
        const transcript::StandardTranscript& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<barretenberg::fr>& scalars)
    {
        auto updated_challenge = VerifierPermutationWidget<barretenberg::fr,
                                                           barretenberg::g1::affine_element,
                                                           transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, challenge, transcript, points, scalars, use_linearisation);

        updated_challenge.nu_index += 4;

        updated_challenge = VerifierTurboFixedBaseWidget<barretenberg::fr,
                                                         barretenberg::g1::affine_element,
                                                         transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);

        updated_challenge = VerifierTurboRangeWidget<barretenberg::fr,
                                                     barretenberg::g1::affine_element,
                                                     transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);

        updated_challenge = VerifierTurboLogicWidget<barretenberg::fr,
                                                     barretenberg::g1::affine_element,
                                                     transcript::StandardTranscript>::
            append_scalar_multiplication_inputs(key, updated_challenge, transcript, points, scalars, use_linearisation);
        return updated_challenge;
    }

    static size_t compute_batch_evaluation_contribution(verification_key* key,
                                                        barretenberg::fr& batch_eval,
                                                        const size_t nu_index,
                                                        const transcript::StandardTranscript& transcript)
    {
        auto updated_nu_index = VerifierPermutationWidget<barretenberg::fr,
                                                          barretenberg::g1::affine_element,
                                                          transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, nu_index, transcript, use_linearisation);
        updated_nu_index += 4;
        updated_nu_index = VerifierTurboFixedBaseWidget<barretenberg::fr,
                                                        barretenberg::g1::affine_element,
                                                        transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        updated_nu_index = VerifierTurboRangeWidget<barretenberg::fr,
                                                    barretenberg::g1::affine_element,
                                                    transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        updated_nu_index = VerifierTurboLogicWidget<barretenberg::fr,
                                                    barretenberg::g1::affine_element,
                                                    transcript::StandardTranscript>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        return updated_nu_index;
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const transcript::StandardTranscript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        auto updated_alpha_base = VerifierPermutationWidget<barretenberg::fr,
                                                            barretenberg::g1::affine_element,
                                                            transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, alpha_base, transcript, t_eval, use_linearisation);

        updated_alpha_base = VerifierTurboFixedBaseWidget<barretenberg::fr,
                                                          barretenberg::g1::affine_element,
                                                          transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = VerifierTurboRangeWidget<barretenberg::fr,
                                                      barretenberg::g1::affine_element,
                                                      transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = VerifierTurboLogicWidget<barretenberg::fr,
                                                      barretenberg::g1::affine_element,
                                                      transcript::StandardTranscript>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};
} // namespace waffle