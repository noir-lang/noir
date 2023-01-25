#pragma once
#include "../../plonk/proof_system/types/plonk_proof.hpp"
#include "./program_settings.hpp"
#include "../../proof_system/verification_key/verification_key.hpp"
#include <transcript/manifest.hpp>
#include <plonk/proof_system/commitment_scheme/commitment_scheme.hpp>
#include "../sumcheck/polynomials/multivariates.hpp"
#include "../sumcheck/sumcheck.hpp"
#include "../sumcheck/relations/arithmetic_relation.hpp"
#include "proof_system/flavor/flavor.hpp"

namespace honk {
template <typename program_settings> class Verifier {

  public:
    Verifier(std::shared_ptr<waffle::verification_key> verifier_key = nullptr,
             const transcript::Manifest& manifest = honk::StandardHonk::create_unrolled_manifest(0));
    Verifier(Verifier&& other);
    Verifier(const Verifier& other) = delete;
    Verifier& operator=(const Verifier& other) = delete;
    Verifier& operator=(Verifier&& other);

    // TODO: plonk_proof is just an std::vector<uint8_t>; probably shouldn't even exist
    bool verify_proof(const waffle::plonk_proof& proof);
    transcript::Manifest manifest;

    std::shared_ptr<waffle::verification_key> key;
    std::map<std::string, barretenberg::g1::affine_element> kate_g1_elements;
    std::map<std::string, barretenberg::fr> kate_fr_elements;
    std::unique_ptr<waffle::CommitmentScheme> commitment_scheme;
};

extern template class Verifier<waffle::standard_verifier_settings>;

typedef Verifier<honk::standard_verifier_settings> StandardVerifier;
typedef Verifier<honk::standard_verifier_settings> StandardUnrolledVerifier;

} // namespace honk