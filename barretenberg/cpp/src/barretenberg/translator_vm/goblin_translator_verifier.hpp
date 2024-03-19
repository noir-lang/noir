#pragma once
#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/goblin/translation_evaluations.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/translator_vm/goblin_translator_prover.hpp"

namespace bb {
class GoblinTranslatorVerifier {
  public:
    using Flavor = GoblinTranslatorFlavor;
    using CircuitBuilder = typename Flavor::CircuitBuilder;
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using Commitment = typename Flavor::Commitment;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using TranslationEvaluations = bb::TranslationEvaluations;
    using Transcript = typename Flavor::Transcript;

    BF evaluation_input_x = 0;
    BF batching_challenge_v = 0;

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::map<std::string, FF> pcs_fr_elements;
    std::shared_ptr<Transcript> transcript;
    RelationParameters<FF> relation_parameters;

    GoblinTranslatorVerifier(const std::shared_ptr<VerificationKey>& verifier_key = nullptr,
                             const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    GoblinTranslatorVerifier(const std::shared_ptr<ProvingKey>& proving_key,
                             const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    void put_translation_data_in_relation_parameters(const uint256_t& evaluation_input_x,
                                                     const BF& batching_challenge_v,
                                                     const uint256_t& accumulated_result);
    bool verify_proof(const HonkProof& proof);
    bool verify_translation(const TranslationEvaluations& translation_evaluations);
};
} // namespace bb
