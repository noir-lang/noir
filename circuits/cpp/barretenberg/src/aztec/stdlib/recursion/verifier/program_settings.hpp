#pragma once

#include <plonk/proof_system/types/program_settings.hpp>
#include <stdlib/types/turbo.hpp>

#include "../transcript/transcript.hpp"

namespace plonk {
namespace stdlib {
namespace recursion {

template <typename Curve> class recursive_turbo_verifier_settings : public waffle::unrolled_turbo_settings {
  public:
    typedef typename Curve::fr_ct fr_ct;
    typedef typename Curve::g1_base_t::affine_element g1_base_t;
    typedef typename Curve::Composer Composer;
    typedef Transcript<Composer> Transcript_pt;
    typedef waffle::VerifierPermutationWidget<fr_ct, g1_base_t, Transcript_pt> PermutationWidget;
    typedef waffle::unrolled_turbo_settings base_settings;

    typedef waffle::VerifierTurboFixedBaseWidget<fr_ct, g1_base_t, Transcript_pt, base_settings> TurboFixedBaseWidget;
    typedef waffle::VerifierTurboArithmeticWidget<fr_ct, g1_base_t, Transcript_pt, base_settings> TurboArithmeticWidget;
    typedef waffle::VerifierTurboRangeWidget<fr_ct, g1_base_t, Transcript_pt, base_settings> TurboRangeWidget;
    typedef waffle::VerifierTurboLogicWidget<fr_ct, g1_base_t, Transcript_pt, base_settings> TurboLogicWidget;

    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake2s;
    static constexpr bool use_linearisation = false;

    static fr_ct append_scalar_multiplication_inputs(typename Transcript_pt::Key* key,
                                                     const fr_ct& alpha_base,
                                                     const Transcript_pt& transcript,
                                                     std::map<std::string, fr_ct>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, scalars, use_linearisation);

        updated_alpha = TurboArithmeticWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, scalars, use_linearisation);

        updated_alpha = TurboFixedBaseWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, scalars, use_linearisation);

        updated_alpha = TurboRangeWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, scalars, use_linearisation);

        updated_alpha = TurboLogicWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, scalars, use_linearisation);
        return updated_alpha;
    }

    static fr_ct compute_quotient_evaluation_contribution(typename Transcript_pt::Key* key,
                                                          const fr_ct& alpha_base,
                                                          const Transcript_pt& transcript,
                                                          fr_ct& r_0)
    {
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, r_0, use_linearisation);
        updated_alpha_base =
            TurboArithmeticWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);
        updated_alpha_base =
            TurboFixedBaseWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);
        updated_alpha_base =
            TurboRangeWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);
        updated_alpha_base =
            TurboLogicWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);
        return updated_alpha_base;
    }
};
} // namespace recursion
} // namespace stdlib
} // namespace plonk
