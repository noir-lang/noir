#pragma once
#include "../types/plonk_proof.hpp"
#include "../types/program_settings.hpp"
#include "../widgets/random_widgets/random_widget.hpp"
#include <transcript/manifest.hpp>
#include <plonk/proof_system/commitment_scheme/commitment_scheme.hpp>

namespace waffle {
template <typename program_settings> class VerifierBase {

  public:
    VerifierBase(std::shared_ptr<verification_key> verifier_key = nullptr,
                 const transcript::Manifest& manifest = transcript::Manifest());
    VerifierBase(VerifierBase&& other);
    VerifierBase(const VerifierBase& other) = delete;
    VerifierBase& operator=(const VerifierBase& other) = delete;
    VerifierBase& operator=(VerifierBase&& other);

    bool validate_commitments();
    bool validate_scalars();

    bool verify_proof(const waffle::plonk_proof& proof);
    transcript::Manifest manifest;

    std::shared_ptr<verification_key> key;
    std::map<std::string, barretenberg::g1::affine_element> kate_g1_elements;
    std::map<std::string, barretenberg::fr> kate_fr_elements;
    std::unique_ptr<CommitmentScheme> commitment_scheme;
};

extern template class VerifierBase<unrolled_standard_verifier_settings>;
extern template class VerifierBase<unrolled_turbo_verifier_settings>;
extern template class VerifierBase<unrolled_ultra_verifier_settings>;
extern template class VerifierBase<unrolled_ultra_to_standard_verifier_settings>;

extern template class VerifierBase<standard_verifier_settings>;
extern template class VerifierBase<turbo_verifier_settings>;
extern template class VerifierBase<ultra_verifier_settings>;
extern template class VerifierBase<generalized_permutation_verifier_settings>;

typedef VerifierBase<unrolled_standard_verifier_settings> UnrolledVerifier;
typedef VerifierBase<unrolled_turbo_verifier_settings> UnrolledTurboVerifier;
typedef VerifierBase<unrolled_ultra_verifier_settings> UnrolledUltraVerifier;
typedef VerifierBase<unrolled_ultra_to_standard_verifier_settings> UnrolledUltraToStandardVerifier;

typedef VerifierBase<standard_verifier_settings> Verifier;
typedef VerifierBase<turbo_verifier_settings> TurboVerifier;
typedef VerifierBase<ultra_verifier_settings> UltraVerifier;
typedef VerifierBase<generalized_permutation_verifier_settings> GenPermVerifier;

} // namespace waffle