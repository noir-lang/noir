

#pragma once

#include "barretenberg/proof_system/circuit_builder/generated/Toy_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/vm/generated/Toy_prover.hpp"
#include "barretenberg/vm/generated/Toy_verifier.hpp"

namespace bb {
class ToyComposer {
  public:
    using Flavor = ToyFlavor;
    using CircuitConstructor = ToyCircuitBuilder;
    using ProvingKey = Flavor::ProvingKey;
    using VerificationKey = Flavor::VerificationKey;
    using PCS = Flavor::PCS;
    using CommitmentKey = Flavor::CommitmentKey;
    using VerifierCommitmentKey = Flavor::VerifierCommitmentKey;

    // TODO: which of these will we really need
    static constexpr std::string_view NAME_STRING = "Toy";
    static constexpr size_t NUM_RESERVED_GATES = 0;
    static constexpr size_t NUM_WIRES = Flavor::NUM_WIRES;

    std::shared_ptr<ProvingKey> proving_key;
    std::shared_ptr<VerificationKey> verification_key;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<bb::srs::factories::CrsFactory<Flavor::Curve>> crs_factory_;

    // The commitment key is passed to the prover but also used herein to compute the verfication key commitments
    std::shared_ptr<CommitmentKey> commitment_key;

    std::vector<uint32_t> recursive_proof_public_input_indices;
    bool contains_recursive_proof = false;
    bool computed_witness = false;

    ToyComposer() { crs_factory_ = bb::srs::get_crs_factory(); }

    ToyComposer(std::shared_ptr<ProvingKey> p_key, std::shared_ptr<VerificationKey> v_key)
        : proving_key(std::move(p_key))
        , verification_key(std::move(v_key))
    {}

    ToyComposer(ToyComposer&& other) noexcept = default;
    ToyComposer(ToyComposer const& other) noexcept = default;
    ToyComposer& operator=(ToyComposer&& other) noexcept = default;
    ToyComposer& operator=(ToyComposer const& other) noexcept = default;
    ~ToyComposer() = default;

    std::shared_ptr<ProvingKey> compute_proving_key(CircuitConstructor& circuit_constructor);
    std::shared_ptr<VerificationKey> compute_verification_key(CircuitConstructor& circuit_constructor);

    void compute_witness(CircuitConstructor& circuit_constructor);

    ToyProver create_prover(CircuitConstructor& circuit_constructor);
    ToyVerifier create_verifier(CircuitConstructor& circuit_constructor);

    void add_table_column_selector_poly_to_proving_key(bb::polynomial& small, const std::string& tag);

    void compute_commitment_key(size_t circuit_size)
    {
        commitment_key = std::make_shared<CommitmentKey>(circuit_size, crs_factory_);
    };
};

} // namespace bb
