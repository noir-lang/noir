#pragma once
#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/goblin/translation_evaluations.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"

namespace bb::honk {
class GoblinTranslatorVerifier {
  public:
    using Flavor = honk::flavor::GoblinTranslator;
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using Commitment = typename Flavor::Commitment;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using TranslationEvaluations = bb::TranslationEvaluations;
    using Transcript = typename Flavor::Transcript;

    BF evaluation_input_x = 0;
    BF batching_challenge_v = 0;
    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::map<std::string, FF> pcs_fr_elements;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key;
    std::shared_ptr<Transcript> transcript;
    RelationParameters<FF> relation_parameters;

    GoblinTranslatorVerifier(const std::shared_ptr<VerificationKey>& verifier_key = nullptr,
                             const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());
    GoblinTranslatorVerifier(GoblinTranslatorVerifier&& other) noexcept;
    GoblinTranslatorVerifier(const GoblinTranslatorVerifier& other) = delete;
    GoblinTranslatorVerifier& operator=(const GoblinTranslatorVerifier& other) = delete;
    GoblinTranslatorVerifier& operator=(GoblinTranslatorVerifier&& other) noexcept;
    ~GoblinTranslatorVerifier() = default;

    void put_translation_data_in_relation_parameters(const uint256_t& evaluation_input_x,
                                                     const BF& batching_challenge_v,
                                                     const uint256_t& accumulated_result);
    bool verify_proof(const plonk::proof& proof);
    bool verify_translation(const TranslationEvaluations& translation_evaluations);
};
} // namespace bb::honk
