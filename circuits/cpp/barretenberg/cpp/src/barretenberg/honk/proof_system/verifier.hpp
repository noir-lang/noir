#pragma once
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "./program_settings.hpp"
#include "barretenberg/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/transcript/manifest.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/commitment_scheme.hpp"
#include "../sumcheck/sumcheck.hpp"
#include "../sumcheck/relations/arithmetic_relation.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/honk/pcs/gemini/gemini.hpp"
#include "barretenberg/honk/pcs/shplonk/shplonk_single.hpp"
#include "barretenberg/honk/pcs/kzg/kzg.hpp"

namespace honk {
template <typename program_settings> class Verifier {

  public:
    Verifier(std::shared_ptr<bonk::verification_key> verifier_key = nullptr,
             const transcript::Manifest& manifest = honk::StandardHonk::create_manifest(0));
    Verifier(Verifier&& other);
    Verifier(const Verifier& other) = delete;
    Verifier& operator=(const Verifier& other) = delete;
    Verifier& operator=(Verifier&& other);

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
