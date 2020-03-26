#pragma once

#include <plonk/proof_system/types/program_settings.hpp>
#include <stdlib/types/turbo.hpp>

#include "../transcript/transcript.hpp"

namespace plonk {
namespace stdlib {
namespace recursion {

class recursive_turbo_verifier_settings : public waffle::unrolled_turbo_settings {
  public:
    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake2s;
    static constexpr bool use_linearisation = false;
    static waffle::VerifierBaseWidget::challenge_coefficients<field_t<waffle::TurboComposer>>
    append_scalar_multiplication_inputs(
        waffle::verification_key* key,
        const waffle::VerifierBaseWidget::challenge_coefficients<field_t<waffle::TurboComposer>>& challenge,
        const Transcript<waffle::TurboComposer>& transcript,
        std::vector<barretenberg::g1::affine_element>& points,
        std::vector<field_t<waffle::TurboComposer>>& scalars)
    {
        waffle::VerifierBaseWidget::challenge_coefficients<field_t<waffle::TurboComposer>> result =
            waffle::VerifierTurboFixedBaseWidget<field_t<waffle::TurboComposer>,
                                                 barretenberg::g1::affine_element,
                                                 Transcript<waffle::TurboComposer>>::
                append_scalar_multiplication_inputs(key, challenge, transcript, points, scalars, use_linearisation);

        result = waffle::VerifierTurboRangeWidget<field_t<waffle::TurboComposer>,
                                                  barretenberg::g1::affine_element,
                                                  Transcript<waffle::TurboComposer>>::
            append_scalar_multiplication_inputs(key, result, transcript, points, scalars, use_linearisation);

        result = waffle::VerifierTurboLogicWidget<field_t<waffle::TurboComposer>,
                                                  barretenberg::g1::affine_element,
                                                  Transcript<waffle::TurboComposer>>::
            append_scalar_multiplication_inputs(key, result, transcript, points, scalars, use_linearisation);
        return result;
    }

    static size_t compute_batch_evaluation_contribution(waffle::verification_key* key,
                                                        field_t<waffle::TurboComposer>& batch_eval,
                                                        const size_t nu_index,
                                                        const Transcript<waffle::TurboComposer>& transcript)
    {
        size_t updated_nu_index = waffle::VerifierTurboFixedBaseWidget<field_t<waffle::TurboComposer>,
                                                                       barretenberg::g1::affine_element,
                                                                       Transcript<waffle::TurboComposer>>::
            compute_batch_evaluation_contribution(key, batch_eval, nu_index, transcript, use_linearisation);
        updated_nu_index = waffle::VerifierTurboRangeWidget<field_t<waffle::TurboComposer>,
                                                            barretenberg::g1::affine_element,
                                                            Transcript<waffle::TurboComposer>>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        updated_nu_index = waffle::VerifierTurboLogicWidget<field_t<waffle::TurboComposer>,
                                                            barretenberg::g1::affine_element,
                                                            Transcript<waffle::TurboComposer>>::
            compute_batch_evaluation_contribution(key, batch_eval, updated_nu_index, transcript, use_linearisation);
        return updated_nu_index;
    }

    static field_t<waffle::TurboComposer> compute_quotient_evaluation_contribution(
        waffle::verification_key* key,
        const field_t<waffle::TurboComposer>& alpha_base,
        const Transcript<waffle::TurboComposer>& transcript,
        field_t<waffle::TurboComposer>& t_eval)
    {
        field_t<waffle::TurboComposer> updated_alpha_base = waffle::VerifierTurboFixedBaseWidget<
            field_t<waffle::TurboComposer>,
            barretenberg::g1::affine_element,
            Transcript<waffle::TurboComposer>>::compute_quotient_evaluation_contribution(key,
                                                                                         alpha_base,
                                                                                         transcript,
                                                                                         t_eval,
                                                                                         use_linearisation);
        updated_alpha_base = waffle::VerifierTurboRangeWidget<field_t<waffle::TurboComposer>,
                                                              barretenberg::g1::affine_element,
                                                              Transcript<waffle::TurboComposer>>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        updated_alpha_base = waffle::VerifierTurboLogicWidget<field_t<waffle::TurboComposer>,
                                                              barretenberg::g1::affine_element,
                                                              Transcript<waffle::TurboComposer>>::
            compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, t_eval, use_linearisation);
        return updated_alpha_base;
    }
};
} // namespace recursion
} // namespace stdlib
} // namespace plonk
