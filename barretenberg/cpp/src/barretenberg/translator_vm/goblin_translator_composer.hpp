#pragma once

#include "barretenberg/flavor/goblin_translator.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/translator_vm/goblin_translator_prover.hpp"
#include "barretenberg/translator_vm/goblin_translator_verifier.hpp"

namespace bb::honk {
class GoblinTranslatorComposer {
  public:
    using Flavor = honk::flavor::GoblinTranslator;
    using Curve = typename Flavor::Curve;
    using CircuitBuilder = typename Flavor::CircuitBuilder;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCS = typename Flavor::PCS;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Polynomial = typename Flavor::Polynomial;
    using Transcript = BaseTranscript;
    static constexpr size_t MINI_CIRCUIT_SIZE = Flavor::MINI_CIRCUIT_SIZE;

    static constexpr std::string_view NAME_STRING = "GoblinTranslator";
    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    std::shared_ptr<ProvingKey> proving_key;
    std::shared_ptr<VerificationKey> verification_key;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<bb::srs::factories::CrsFactory<Curve>> crs_factory_;

    // The commitment key is passed to the prover but also used herein to compute the verfication key commitments
    std::shared_ptr<CommitmentKey> commitment_key;

    bool computed_witness = false;
    size_t total_num_gates = 0;          // num_gates (already include zero row offset) (used to compute dyadic size)
    size_t dyadic_circuit_size = 0;      // final power-of-2 circuit size
    size_t mini_circuit_dyadic_size = 0; // The size of the small circuit that contains non-range constraint relations

    // We only need the standard crs factory. GoblinTranslator is not supposed to be used with Grumpkin
    GoblinTranslatorComposer() { crs_factory_ = bb::srs::get_crs_factory(); }

    GoblinTranslatorComposer(std::shared_ptr<ProvingKey> p_key, std::shared_ptr<VerificationKey> v_key)
        : proving_key(std::move(p_key))
        , verification_key(std::move(v_key))
    {}

    std::shared_ptr<ProvingKey> compute_proving_key(const CircuitBuilder& circuit_builder);
    std::shared_ptr<VerificationKey> compute_verification_key(const CircuitBuilder& circuit_builder);

    void compute_circuit_size_parameters(CircuitBuilder& circuit_builder);

    void compute_witness(CircuitBuilder& circuit_builder);

    GoblinTranslatorProver create_prover(
        CircuitBuilder& circuit_builder,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());
    GoblinTranslatorVerifier create_verifier(
        const CircuitBuilder& circuit_builder,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    std::shared_ptr<CommitmentKey> compute_commitment_key(size_t circuit_size)
    {
        if (commitment_key) {
            return commitment_key;
        }

        commitment_key = std::make_shared<CommitmentKey>(circuit_size, crs_factory_);
        return commitment_key;
    };
};
} // namespace bb::honk