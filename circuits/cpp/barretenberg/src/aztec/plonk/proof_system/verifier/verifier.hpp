#pragma once
#include "../types/plonk_proof.hpp"
#include "../types/program_settings.hpp"
#include "../verification_key/verification_key.hpp"
#include "../widgets/base_widget.hpp"
#include <plonk/transcript/manifest.hpp>

namespace waffle {
template <typename program_settings> class VerifierBase {

  public:
    VerifierBase(std::shared_ptr<verification_key> verifier_key = nullptr,
                 const transcript::Manifest& manifest = transcript::Manifest({}));
    VerifierBase(VerifierBase&& other);
    VerifierBase(const VerifierBase& other) = delete;
    VerifierBase& operator=(const VerifierBase& other) = delete;
    VerifierBase& operator=(VerifierBase&& other);

    bool verify_proof(const waffle::plonk_proof& proof);

    transcript::Manifest manifest;

    std::shared_ptr<verification_key> key;
};

extern template class VerifierBase<unrolled_standard_verifier_settings>;
extern template class VerifierBase<unrolled_turbo_verifier_settings>;
extern template class VerifierBase<standard_verifier_settings>;
extern template class VerifierBase<unrolled_plookup_verifier_settings>;
extern template class VerifierBase<mimc_verifier_settings>;
extern template class VerifierBase<turbo_verifier_settings>;
extern template class VerifierBase<plookup_verifier_settings>;

typedef VerifierBase<unrolled_standard_verifier_settings> UnrolledVerifier;
typedef VerifierBase<unrolled_turbo_verifier_settings> UnrolledTurboVerifier;
typedef VerifierBase<standard_verifier_settings> Verifier;
typedef VerifierBase<mimc_verifier_settings> MiMCVerifier;
typedef VerifierBase<turbo_verifier_settings> TurboVerifier;
typedef VerifierBase<plookup_verifier_settings> PLookupVerifier;
typedef VerifierBase<unrolled_plookup_verifier_settings> UnrolledPLookupVerifier;

} // namespace waffle