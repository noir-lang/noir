#pragma once
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "./program_settings.hpp"
#include "barretenberg/plonk/proof_system/verification_key/verification_key.hpp"
#include "barretenberg/transcript/manifest.hpp"
#include "barretenberg/plonk/proof_system/commitment_scheme/commitment_scheme.hpp"
#include "../sumcheck/sumcheck.hpp"
#include "../sumcheck/relations/arithmetic_relation.hpp"
#include "barretenberg/honk/pcs/commitment_key.hpp"
#include "barretenberg/honk/flavor/flavor.hpp"
#include "barretenberg/honk/pcs/gemini/gemini.hpp"
#include "barretenberg/honk/pcs/shplonk/shplonk_single.hpp"
#include "barretenberg/honk/pcs/kzg/kzg.hpp"

namespace proof_system::honk {
template <typename program_settings> class Verifier {

  public:
    Verifier(std::shared_ptr<plonk::verification_key> verifier_key = nullptr);
    Verifier(Verifier&& other);
    Verifier(const Verifier& other) = delete;
    Verifier& operator=(const Verifier& other) = delete;
    Verifier& operator=(Verifier&& other);

    bool verify_proof(const plonk::proof& proof);

    std::shared_ptr<plonk::verification_key> key;
    std::map<std::string, barretenberg::g1::affine_element> kate_g1_elements;
    std::map<std::string, barretenberg::fr> kate_fr_elements;
    std::shared_ptr<pcs::kzg::VerificationKey> kate_verification_key;
    VerifierTranscript<typename program_settings::fr> transcript;
};

extern template class Verifier<honk::standard_verifier_settings>;

typedef Verifier<honk::standard_verifier_settings> StandardVerifier;

} // namespace proof_system::honk
