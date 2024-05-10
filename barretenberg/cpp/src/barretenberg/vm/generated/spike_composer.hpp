

#pragma once

#include "barretenberg/plonk_honk_shared/composer/composer_lib.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/vm/generated/spike_circuit_builder.hpp"
#include "barretenberg/vm/generated/spike_prover.hpp"
#include "barretenberg/vm/generated/spike_verifier.hpp"

namespace bb {
class SpikeComposer {
  public:
    using Flavor = SpikeFlavor;
    using CircuitConstructor = SpikeCircuitBuilder;
    using ProvingKey = Flavor::ProvingKey;
    using VerificationKey = Flavor::VerificationKey;
    using PCS = Flavor::PCS;
    using CommitmentKey = Flavor::CommitmentKey;
    using VerifierCommitmentKey = Flavor::VerifierCommitmentKey;

    // TODO: which of these will we really need
    static constexpr std::string_view NAME_STRING = "Spike";
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

    SpikeComposer() { crs_factory_ = bb::srs::get_bn254_crs_factory(); }

    SpikeComposer(std::shared_ptr<ProvingKey> p_key, std::shared_ptr<VerificationKey> v_key)
        : proving_key(std::move(p_key))
        , verification_key(std::move(v_key))
    {}

    SpikeComposer(SpikeComposer&& other) noexcept = default;
    SpikeComposer(SpikeComposer const& other) noexcept = default;
    SpikeComposer& operator=(SpikeComposer&& other) noexcept = default;
    SpikeComposer& operator=(SpikeComposer const& other) noexcept = default;
    ~SpikeComposer() = default;

    std::shared_ptr<ProvingKey> compute_proving_key(CircuitConstructor& circuit_constructor);
    std::shared_ptr<VerificationKey> compute_verification_key(CircuitConstructor& circuit_constructor);

    void compute_witness(CircuitConstructor& circuit_constructor);

    SpikeProver create_prover(CircuitConstructor& circuit_constructor);
    SpikeVerifier create_verifier(CircuitConstructor& circuit_constructor);

    void add_table_column_selector_poly_to_proving_key(bb::polynomial& small, const std::string& tag);

    void compute_commitment_key(size_t circuit_size)
    {
        proving_key->commitment_key = std::make_shared<CommitmentKey>(circuit_size);
    };
};

} // namespace bb
