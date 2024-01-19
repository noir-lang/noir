#pragma once

#include "barretenberg/eccvm/eccvm_prover.hpp"
#include "barretenberg/eccvm/eccvm_verifier.hpp"
#include "barretenberg/proof_system/circuit_builder/eccvm/eccvm_circuit_builder.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"
#include "barretenberg/srs/global_crs.hpp"

namespace bb::honk {
template <ECCVMFlavor Flavor> class ECCVMComposer_ {
  public:
    using FF = typename Flavor::FF;
    using CircuitConstructor = ECCVMCircuitBuilder<Flavor>;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCS = typename Flavor::PCS;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;
    using Transcript = typename Flavor::Transcript;

    static constexpr std::string_view NAME_STRING = "ECCVM";
    static constexpr size_t NUM_RESERVED_GATES = 0; // equal to the number of multilinear evaluations leaked
    static constexpr size_t NUM_WIRES = CircuitConstructor::NUM_WIRES;
    std::shared_ptr<ProvingKey> proving_key;
    std::shared_ptr<VerificationKey> verification_key;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<bb::srs::factories::CrsFactory<typename Flavor::Curve>> crs_factory_;

    // The commitment key is passed to the prover but also used herein to compute the verfication key commitments
    std::shared_ptr<CommitmentKey> commitment_key;

    std::vector<uint32_t> recursive_proof_public_input_indices;
    bool contains_recursive_proof = false;
    bool computed_witness = false;
    ECCVMComposer_()
        requires(std::same_as<Flavor, honk::flavor::ECCVM>)
    {
        crs_factory_ = bb::srs::get_grumpkin_crs_factory();
    };

    explicit ECCVMComposer_(std::shared_ptr<bb::srs::factories::CrsFactory<typename Flavor::Curve>> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    ECCVMComposer_(std::shared_ptr<ProvingKey> p_key, std::shared_ptr<VerificationKey> v_key)
        : proving_key(std::move(p_key))
        , verification_key(std::move(v_key))
    {}

    ECCVMComposer_(ECCVMComposer_&& other) noexcept = default;
    ECCVMComposer_(ECCVMComposer_ const& other) noexcept = default;
    ECCVMComposer_& operator=(ECCVMComposer_&& other) noexcept = default;
    ECCVMComposer_& operator=(ECCVMComposer_ const& other) noexcept = default;
    ~ECCVMComposer_() = default;

    std::shared_ptr<ProvingKey> compute_proving_key(CircuitConstructor& circuit_constructor);
    std::shared_ptr<VerificationKey> compute_verification_key(CircuitConstructor& circuit_constructor);

    void compute_witness(CircuitConstructor& circuit_constructor);

    ECCVMProver_<Flavor> create_prover(CircuitConstructor& circuit_constructor,
                                       const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());
    ECCVMVerifier_<Flavor> create_verifier(
        CircuitConstructor& circuit_constructor,
        const std::shared_ptr<Transcript>& transcript = std::make_shared<Transcript>());

    void add_table_column_selector_poly_to_proving_key(bb::polynomial& small, const std::string& tag);

    void compute_commitment_key(size_t circuit_size)
    {
        commitment_key = std::make_shared<CommitmentKey>(circuit_size, crs_factory_);
    };
};

// TODO(#532): this pattern is weird; is this not instantiating the templates?
using ECCVMComposer = ECCVMComposer_<honk::flavor::ECCVM>;

} // namespace bb::honk
