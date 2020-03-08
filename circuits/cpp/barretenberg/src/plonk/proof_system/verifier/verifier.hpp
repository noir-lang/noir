#pragma once
#include <plonk/transcript/manifest.hpp>
#include "../verification_key/verification_key.hpp"
#include "../types/plonk_proof.hpp"
#include "../types/program_settings.hpp"
#include "../widgets/base_widget.hpp"

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

    std::vector<std::unique_ptr<VerifierBaseWidget>> verifier_widgets;

    transcript::Manifest manifest;

    std::shared_ptr<verification_key> key;
};

extern template class VerifierBase<standard_settings>;
extern template class VerifierBase<turbo_settings>;

typedef VerifierBase<standard_settings> Verifier;
typedef VerifierBase<turbo_settings> TurboVerifier;

} // namespace waffle