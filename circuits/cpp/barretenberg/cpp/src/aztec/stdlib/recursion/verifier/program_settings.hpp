#pragma once

#include <plonk/proof_system/types/program_settings.hpp>

#include "../transcript/transcript.hpp"

namespace plonk {
namespace stdlib {
namespace recursion {

template <typename Curve> class recursive_ultra_verifier_settings : public waffle::unrolled_ultra_verifier_settings {
  public:
    typedef typename Curve::fr_ct fr_ct;
    typedef typename Curve::g1::affine_element g1;
    typedef typename Curve::Composer Composer;
    typedef plonk::stdlib::recursion::Transcript<Composer> Transcript_pt;
    typedef waffle::VerifierPermutationWidget<fr_ct, g1, Transcript_pt> PermutationWidget;
    typedef waffle::VerifierPlookupWidget<fr_ct, g1, Transcript_pt> PlookupWidget;

    typedef waffle::unrolled_ultra_settings base_settings;

    typedef waffle::VerifierUltraFixedBaseWidget<fr_ct, g1, Transcript_pt, base_settings> UltraFixedBaseWidget;
    typedef waffle::VerifierPlookupArithmeticWidget<fr_ct, g1, Transcript_pt, base_settings> PlookupArithmeticWidget;
    typedef waffle::VerifierTurboLogicWidget<fr_ct, g1, Transcript_pt, base_settings> TurboLogicWidget;
    typedef waffle::VerifierGenPermSortWidget<fr_ct, g1, Transcript_pt, base_settings> GenPermSortWidget;
    typedef waffle::VerifierEllipticWidget<fr_ct, g1, Transcript_pt, base_settings> EllipticWidget;
    typedef waffle::VerifierPlookupAuxiliaryWidget<fr_ct, g1, Transcript_pt, base_settings> PlookupAuxiliaryWidget;

    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PlookupPedersenBlake3s;
    static constexpr bool use_linearisation =
        false; // We don't compute a linearisation polynomial when verifying within a circuit.
    // idpolys is a flag that describes whether we're using Vitalik's trick of using trivial identity permutation
    // polynomials (id_poly = false); OR whether the identity permutation polynomials are circuit-specific and stored in
    // the proving/verification key (id_poly = true).
    static constexpr bool idpolys = true;

    static fr_ct append_scalar_multiplication_inputs(typename Transcript_pt::Key* key,
                                                     const fr_ct& alpha_base,
                                                     const Transcript_pt& transcript,
                                                     std::map<std::string, fr_ct>& scalars)
    {
        auto updated_alpha = PermutationWidget::append_scalar_multiplication_inputs(
            key, alpha_base, transcript, scalars, use_linearisation, idpolys);

        updated_alpha = PlookupWidget::append_scalar_multiplication_inputs(
            key, updated_alpha, transcript, scalars, use_linearisation);

        updated_alpha =
            PlookupArithmeticWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);

        updated_alpha =
            UltraFixedBaseWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);

        updated_alpha = GenPermSortWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);

        updated_alpha = EllipticWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);

        updated_alpha =
            PlookupAuxiliaryWidget::append_scalar_multiplication_inputs(key, updated_alpha, transcript, scalars);

        return updated_alpha;
    }

    static fr_ct compute_quotient_evaluation_contribution(typename Transcript_pt::Key* key,
                                                          const fr_ct& alpha_base,
                                                          const Transcript_pt& transcript,
                                                          fr_ct& r_0)
    {
        auto updated_alpha_base = PermutationWidget::compute_quotient_evaluation_contribution(
            key, alpha_base, transcript, r_0, use_linearisation, idpolys);

        updated_alpha_base = PlookupWidget::compute_quotient_evaluation_contribution(
            key, updated_alpha_base, transcript, r_0, use_linearisation);

        updated_alpha_base =
            PlookupArithmeticWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);

        updated_alpha_base =
            UltraFixedBaseWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);

        updated_alpha_base =
            GenPermSortWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);

        updated_alpha_base =
            EllipticWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);

        updated_alpha_base =
            PlookupAuxiliaryWidget::compute_quotient_evaluation_contribution(key, updated_alpha_base, transcript, r_0);

        return updated_alpha_base;
    }
};

// Only needed because ultra-to-standard recursion requires us to use a Pedersen hash which is common to both Ultra &
// Standard plonk i.e. the non-ultra version.
template <typename Curve>
class recursive_ultra_to_standard_verifier_settings : public recursive_ultra_verifier_settings<Curve> {
  public:
    typedef typename Curve::fr_ct fr_ct;
    typedef typename Curve::g1::affine_element g1;
    typedef typename Curve::Composer Composer;
    typedef plonk::stdlib::recursion::Transcript<Composer> Transcript_pt;
    typedef waffle::VerifierPermutationWidget<fr_ct, g1, Transcript_pt> PermutationWidget;
    typedef waffle::VerifierPlookupWidget<fr_ct, g1, Transcript_pt> PlookupWidget;

    typedef waffle::unrolled_ultra_to_standard_settings base_settings;

    typedef waffle::VerifierUltraFixedBaseWidget<fr_ct, g1, Transcript_pt, base_settings> UltraFixedBaseWidget;
    typedef waffle::VerifierPlookupArithmeticWidget<fr_ct, g1, Transcript_pt, base_settings> PlookupArithmeticWidget;
    typedef waffle::VerifierTurboLogicWidget<fr_ct, g1, Transcript_pt, base_settings> TurboLogicWidget;
    typedef waffle::VerifierGenPermSortWidget<fr_ct, g1, Transcript_pt, base_settings> GenPermSortWidget;
    typedef waffle::VerifierEllipticWidget<fr_ct, g1, Transcript_pt, base_settings> EllipticWidget;
    typedef waffle::VerifierPlookupAuxiliaryWidget<fr_ct, g1, Transcript_pt, base_settings> PlookupAuxiliaryWidget;

    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
};

template <typename Curve> class recursive_turbo_verifier_settings : public waffle::unrolled_turbo_settings {
  public:
    typedef typename Curve::fr_ct fr_ct;
    typedef typename Curve::g1::affine_element g1;
    typedef typename Curve::Composer Composer;
    typedef Transcript<Composer> Transcript_pt;
    typedef waffle::VerifierPermutationWidget<fr_ct, g1, Transcript_pt> PermutationWidget;

    typedef waffle::unrolled_turbo_settings base_settings;

    typedef waffle::VerifierTurboFixedBaseWidget<fr_ct, g1, Transcript_pt, base_settings> TurboFixedBaseWidget;
    typedef waffle::VerifierTurboArithmeticWidget<fr_ct, g1, Transcript_pt, base_settings> TurboArithmeticWidget;
    typedef waffle::VerifierTurboRangeWidget<fr_ct, g1, Transcript_pt, base_settings> TurboRangeWidget;
    typedef waffle::VerifierTurboLogicWidget<fr_ct, g1, Transcript_pt, base_settings> TurboLogicWidget;

    static constexpr size_t num_challenge_bytes = 16;
    static constexpr transcript::HashType hash_type = transcript::HashType::PedersenBlake3s;
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
