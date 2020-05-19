#pragma once
#include <memory>

namespace waffle {
struct verification_key;
struct proving_key;
class VerifierReferenceString;

namespace standard_composer {

std::shared_ptr<verification_key> compute_verification_key(std::shared_ptr<proving_key> const& circuit_proving_key,
                                                           std::shared_ptr<VerifierReferenceString> const& vrs);

}
} // namespace waffle