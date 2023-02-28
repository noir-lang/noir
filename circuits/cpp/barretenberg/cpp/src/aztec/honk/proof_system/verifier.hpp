#pragma once
#include "../../plonk/proof_system/types/proof.hpp"
#include "./program_settings.hpp"
#include "../../proof_system/verification_key/verification_key.hpp"
#include <transcript/manifest.hpp>
#include <plonk/proof_system/commitment_scheme/commitment_scheme.hpp>
#include "../sumcheck/sumcheck.hpp"
#include "../sumcheck/relations/arithmetic_relation.hpp"
#include "honk/pcs/commitment_key.hpp"
#include "proof_system/flavor/flavor.hpp"
#include <honk/pcs/gemini/gemini.hpp>
#include <honk/pcs/shplonk/shplonk_single.hpp>
#include <honk/pcs/kzg/kzg.hpp>

namespace honk {
template <typename program_settings> class Verifier {

  public:
    Verifier(std::shared_ptr<bonk::verification_key> verifier_key = nullptr,
             const transcript::Manifest& manifest = honk::StandardHonk::create_manifest(0));
    Verifier(Verifier&& other);
    Verifier(const Verifier& other) = delete;
    Verifier& operator=(const Verifier& other) = delete;
    Verifier& operator=(Verifier&& other);

    // TODO(luke): proof is just an std::vector<uint8_t>; probably shouldn't even exist
    // Cody: Idk, what's wrong with an informative alias?
    // An improvement would be to template by flavor and then have proof contain even more info,
    // so it's easy to extract particular elements without looking at the manifest and counting
    // numbers of bytes, for instance.
    bool verify_proof(const plonk::proof& proof);
    transcript::Manifest manifest;

    std::shared_ptr<bonk::verification_key> key;
    std::map<std::string, barretenberg::g1::affine_element> kate_g1_elements;
    std::map<std::string, barretenberg::fr> kate_fr_elements;
    std::shared_ptr<pcs::kzg::VerificationKey> kate_verification_key;
};

extern template class Verifier<honk::standard_verifier_settings>;

typedef Verifier<honk::standard_verifier_settings> StandardVerifier;

} // namespace honk