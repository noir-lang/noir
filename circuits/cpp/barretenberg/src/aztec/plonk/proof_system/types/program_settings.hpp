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
#include "../widgets/genperm_sort_widget.hpp"

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
    static constexpr size_t num_shifted_wire_evaluations = 1;
    static constexpr uint64_t wire_shift_settings = 0b0100;
    static constexpr bool uses_quotient_mid = true;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr bool use_linearisation = true;
};

class unrolled_standard_settings : public settings_base {
  public:
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake2s;
    static constexpr size_t program_width = 3;
    static constexpr size_t num_shifted_wire_evaluations = 1;
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
    static constexpr size_t num_shifted_wire_evaluations = 4;
    static constexpr uint64_t wire_shift_settings = 0b1111;
    static constexpr bool uses_quotient_mid = false;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr bool use_linearisation = true;
};

class unrolled_turbo_settings : public settings_base {
  public:
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake2s;
    static constexpr size_t program_width = 4;
    static constexpr size_t num_shifted_wire_evaluations = 4;
    static constexpr uint64_t wire_shift_settings = 0b1111;
    static constexpr bool uses_quotient_mid = false;
    static constexpr uint32_t permutation_shift = 30;
    static constexpr uint32_t permutation_mask = 0xC0000000;
    static constexpr bool use_linearisation = false;
};

class standard_verifier_settings : public standard_settings {
  public:
    typedef barretenberg::fr fr;
    typedef barretenberg::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierArithmeticWidget<fr, g1::affine_element, Transcript> ArithmeticWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;

    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr bool use_linearisation = true;
    static constexpr bool idpolys = false;

    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const Transcript& transcript,
                                                  std::vector<g1::affine_element>& points,
                                                  std::vector<fr>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, points, scalars, use_linearisation);

        return ArithmeticWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
    }

    static void compute_batch_evaluation_contribution(verification_key* key,
                                                      fr& batch_eval,
                                                      const Transcript& transcript)
    {
        PermutationWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        ArithmeticWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const fr& alpha_base,
                                                                     const Transcript& transcript,
                                                                     fr& t_eval)
    {
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = ArithmeticWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};

class unrolled_standard_verifier_settings : public standard_settings {
  public:
    typedef barretenberg::fr fr;
    typedef barretenberg::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierArithmeticWidget<fr, g1::affine_element, Transcript> ArithmeticWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;

    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake2s;
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr bool use_linearisation = false;
    static constexpr bool idpolys = false;
    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const Transcript& transcript,
                                                  std::vector<barretenberg::g1::affine_element>& points,
                                                  std::vector<barretenberg::fr>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, points, scalars, use_linearisation);

        return ArithmeticWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
    }

    static void compute_batch_evaluation_contribution(verification_key* key,
                                                      barretenberg::fr& batch_eval,
                                                      const Transcript& transcript)
    {
        PermutationWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        ArithmeticWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
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

        return ArithmeticWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
    }
};

class mimc_verifier_settings : public standard_settings {
  public:
    typedef barretenberg::fr fr;
    typedef barretenberg::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierArithmeticWidget<fr, g1::affine_element, Transcript> ArithmeticWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;
    typedef VerifierMiMCWidget<fr, g1::affine_element, Transcript> MiMCWidget;

    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr bool use_linearisation = true;
    static constexpr bool idpolys = false;
    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const Transcript& transcript,
                                                  std::vector<barretenberg::g1::affine_element>& points,
                                                  std::vector<barretenberg::fr>& scalars)
    {

        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, points, scalars, use_linearisation);

        updated_alpha = ArithmeticWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        updated_alpha = MiMCWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        return updated_alpha;
    }

    static void compute_batch_evaluation_contribution(verification_key* key,
                                                      fr& batch_eval,
                                                      const Transcript& transcript)
    {
        PermutationWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        ArithmeticWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        MiMCWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
    }

    static fr compute_quotient_evaluation_contribution(verification_key* key,
                                                       const fr& alpha_base,
                                                       const Transcript& transcript,
                                                       fr& t_eval)
    {
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, t_eval, use_linearisation);

        updated_alpha_base = ArithmeticWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = MiMCWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};

class turbo_verifier_settings : public turbo_settings {
  public:
    typedef barretenberg::fr fr;
    typedef barretenberg::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierTurboFixedBaseWidget<fr, g1::affine_element, Transcript> TurboFixedBaseWidget;
    typedef VerifierGenPermSortWidget<fr, g1::affine_element, Transcript> GenPermSortWidget;
    typedef VerifierTurboLogicWidget<fr, g1::affine_element, Transcript> TurboLogicWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;

    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr bool use_linearisation = true;
    static constexpr bool idpolys = false;
    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const transcript::StandardTranscript& transcript,
                                                  std::vector<g1::affine_element>& points,
                                                  std::vector<fr>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, points, scalars, use_linearisation);

        updated_alpha = TurboFixedBaseWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        updated_alpha = GenPermSortWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        updated_alpha = TurboLogicWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        return updated_alpha;
    }

    static void compute_batch_evaluation_contribution(verification_key* key,
                                                      barretenberg::fr& batch_eval,
                                                      const Transcript& transcript)
    {
        PermutationWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        TurboFixedBaseWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        GenPermSortWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        TurboLogicWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const fr& alpha_base,
                                                                     const Transcript& transcript,
                                                                     fr& t_eval)
    {
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, t_eval, use_linearisation);

        updated_alpha_base = TurboFixedBaseWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = GenPermSortWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = TurboLogicWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};

class generalized_permutation_verifier_settings : public turbo_settings {
  public:
    typedef barretenberg::fr fr;
    typedef barretenberg::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierTurboFixedBaseWidget<fr, g1::affine_element, Transcript> TurboFixedBaseWidget;
    typedef VerifierGenPermSortWidget<fr, g1::affine_element, Transcript> GenPermSortWidget;
    typedef VerifierTurboLogicWidget<fr, g1::affine_element, Transcript> TurboLogicWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;

    static constexpr size_t num_challenge_bytes = 32;
    static constexpr transcript::HashType hash_type = transcript::HashType::Keccak256;
    static constexpr bool use_linearisation = true;
    static constexpr bool idpolys = true;
    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const transcript::StandardTranscript& transcript,
                                                  std::vector<g1::affine_element>& points,
                                                  std::vector<fr>& scalars)
    {
        std::cout << "ingenpermsettings" << std::endl;
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, points, scalars, use_linearisation, true);

        updated_alpha = TurboFixedBaseWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        updated_alpha = GenPermSortWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        updated_alpha = TurboLogicWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        return updated_alpha;
    }

    static void compute_batch_evaluation_contribution(verification_key* key,
                                                      barretenberg::fr& batch_eval,
                                                      const Transcript& transcript)
    {
        std::cout << "ingenpermsettings" << std::endl;
        PermutationWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation, true);
        TurboFixedBaseWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        GenPermSortWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        TurboLogicWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const fr& alpha_base,
                                                                     const Transcript& transcript,
                                                                     fr& t_eval)
    {
        std::cout << "ingenpermsettings" << std::endl;
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, t_eval, use_linearisation, true);

        updated_alpha_base = TurboFixedBaseWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = GenPermSortWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = TurboLogicWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};

class unrolled_turbo_verifier_settings : public unrolled_turbo_settings {
  public:
    typedef barretenberg::fr fr;
    typedef barretenberg::g1 g1;
    typedef transcript::StandardTranscript Transcript;
    typedef VerifierTurboFixedBaseWidget<fr, g1::affine_element, Transcript> TurboFixedBaseWidget;
    typedef VerifierGenPermSortWidget<fr, g1::affine_element, Transcript> GenPermSortWidget;
    typedef VerifierTurboLogicWidget<fr, g1::affine_element, Transcript> TurboLogicWidget;
    typedef VerifierPermutationWidget<fr, g1::affine_element, Transcript> PermutationWidget;

    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake2s;
    static constexpr bool use_linearisation = false;
    static constexpr bool idpolys = false;
    static fr append_scalar_multiplication_inputs(verification_key* key,
                                                  const fr& alpha_base,
                                                  const Transcript& transcript,
                                                  std::vector<barretenberg::g1::affine_element>& points,
                                                  std::vector<barretenberg::fr>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, points, scalars, use_linearisation);

        updated_alpha = TurboFixedBaseWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);

        updated_alpha = GenPermSortWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);

        updated_alpha = TurboLogicWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, points, scalars, use_linearisation);
        return updated_alpha;
    }

    static void compute_batch_evaluation_contribution(verification_key* key,
                                                      barretenberg::fr& batch_eval,
                                                      const Transcript& transcript)
    {
        PermutationWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        TurboFixedBaseWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        GenPermSortWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
        TurboLogicWidget::compute_batch_evaluation_contribution(key, batch_eval, transcript, use_linearisation);
    }

    static barretenberg::fr compute_quotient_evaluation_contribution(verification_key* key,
                                                                     const barretenberg::fr& alpha_base,
                                                                     const Transcript& transcript,
                                                                     barretenberg::fr& t_eval)
    {
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = TurboFixedBaseWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = GenPermSortWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = TurboLogicWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};
} // namespace waffle