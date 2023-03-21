#include "verifier_reference_string.hpp"

namespace serialize {

// TODO(AD): After Milestone 1, rewrite this with better injection mechanism.
// Use a pointer to play nicely in case initialization does not occur (i.e. WASM)
static std::shared_ptr<bonk::VerifierReferenceString>* global_verifier_reference_string;

// TODO(AD): After Milestone 1, rewrite this with better injection mechanism.
std::shared_ptr<bonk::VerifierReferenceString> get_global_verifier_reference_string()
{
    return global_verifier_reference_string ? *global_verifier_reference_string
                                            : std::shared_ptr<bonk::VerifierReferenceString>();
}
// TODO(AD): After Milestone 1, rewrite this with better injection mechanism.
void set_global_verifier_reference_string(std::shared_ptr<bonk::VerifierReferenceString> const& vrs)
{
    if (global_verifier_reference_string == nullptr) {
        global_verifier_reference_string = new std::shared_ptr<bonk::VerifierReferenceString>();
    }
    *global_verifier_reference_string = vrs;
}

} // namespace serialize
