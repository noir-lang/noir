#pragma once
#include "barretenberg/goblin/translation_evaluations.hpp"
#include "barretenberg/honk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib/honk_recursion/transcript/transcript.hpp"
#include "barretenberg/sumcheck/sumcheck.hpp"
#include "barretenberg/translator_vm/translator_prover.hpp"
#include "barretenberg/translator_vm_recursion/translator_recursive_flavor.hpp"

namespace bb {
template <typename Flavor> class TranslatorRecursiveVerifier_ {
  public:
    using FF = typename Flavor::FF;
    using BF = typename Flavor::BF;
    using NativeBF = typename Flavor::Curve::BaseFieldNative;
    using Commitment = typename Flavor::Commitment;
    using GroupElement = typename Flavor::GroupElement;
    using VerificationKey = typename Flavor::VerificationKey;
    using NativeVerificationKey = typename Flavor::NativeVerificationKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Builder = typename Flavor::CircuitBuilder;
    using RelationSeparator = typename Flavor::RelationSeparator;
    using PairingPoints = std::array<GroupElement, 2>;
    using TranslationEvaluations = TranslationEvaluations_<BF, FF>;
    using Transcript = typename Flavor::Transcript;
    using RelationParams = ::bb::RelationParameters<FF>;

    BF evaluation_input_x = 0;
    BF batching_challenge_v = 0;

    std::shared_ptr<VerificationKey> key;
    std::map<std::string, Commitment> commitments;
    std::shared_ptr<Transcript> transcript;
    std::shared_ptr<VerifierCommitmentKey> pcs_verification_key; // can remove maybe hopefully
    Builder* builder;

    RelationParams relation_parameters;

    TranslatorRecursiveVerifier_(Builder* builder,
                                 const std::shared_ptr<NativeVerificationKey>& native_verifier_key,
                                 const std::shared_ptr<Transcript>& transcript);

    void put_translation_data_in_relation_parameters(const BF& evaluation_input_x,
                                                     const BF& batching_challenge_v,
                                                     const BF& accumulated_result);

    PairingPoints verify_proof(const HonkProof& proof);

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/986): Ensure the translation is also recursively
    // verified somewhere
    bool verify_translation(const TranslationEvaluations& translation_evaluations);
};
} // namespace bb