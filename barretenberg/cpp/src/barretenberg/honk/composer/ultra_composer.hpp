#pragma once

#include "barretenberg/honk/proof_system/ultra_prover.hpp"
#include "barretenberg/honk/proof_system/ultra_verifier.hpp"
#include "barretenberg/proof_system/composer/composer_lib.hpp"
#include "barretenberg/proof_system/flavor/flavor.hpp"
#include "barretenberg/srs/factories/file_crs_factory.hpp"

#include <cstddef>
#include <memory>
#include <utility>
#include <vector>

namespace proof_system::honk {
template <UltraFlavor Flavor> class UltraComposer_ {
  public:
    using CircuitBuilder = typename Flavor::CircuitBuilder;
    using ProvingKey = typename Flavor::ProvingKey;
    using VerificationKey = typename Flavor::VerificationKey;
    using PCS = typename Flavor::PCS;
    using CommitmentKey = typename Flavor::CommitmentKey;
    using VerifierCommitmentKey = typename Flavor::VerifierCommitmentKey;

    // offset due to placing zero wires at the start of execution trace
    static constexpr size_t num_zero_rows = Flavor::has_zero_row ? 1 : 0;

    static constexpr std::string_view NAME_STRING = "UltraHonk";
    static constexpr size_t NUM_WIRES = CircuitBuilder::NUM_WIRES;
    std::shared_ptr<ProvingKey> proving_key;
    std::shared_ptr<VerificationKey> verification_key;

    // The crs_factory holds the path to the srs and exposes methods to extract the srs elements
    std::shared_ptr<srs::factories::CrsFactory<typename Flavor::Curve>> crs_factory_;

    // The commitment key is passed to the prover but also used herein to compute the verfication key commitments
    std::shared_ptr<CommitmentKey> commitment_key;

    std::vector<uint32_t> recursive_proof_public_input_indices;
    bool contains_recursive_proof = false;
    bool computed_witness = false;
    size_t total_num_gates = 0; // num_gates + num_pub_inputs + tables + zero_row_offset (used to compute dyadic size)
    size_t dyadic_circuit_size = 0; // final power-of-2 circuit size
    size_t lookups_size = 0;        // total number of lookup gates
    size_t tables_size = 0;         // total number of table entries
    size_t num_public_inputs = 0;
    size_t num_ecc_op_gates = 0;

    UltraComposer_() { crs_factory_ = barretenberg::srs::get_crs_factory(); }

    explicit UltraComposer_(std::shared_ptr<srs::factories::CrsFactory<typename Flavor::Curve>> crs_factory)
        : crs_factory_(std::move(crs_factory))
    {}

    UltraComposer_(std::shared_ptr<ProvingKey> p_key, std::shared_ptr<VerificationKey> v_key)
        : proving_key(std::move(p_key))
        , verification_key(std::move(v_key))
    {}

    UltraComposer_(UltraComposer_&& other) noexcept = default;
    UltraComposer_(UltraComposer_ const& other) noexcept = default;
    UltraComposer_& operator=(UltraComposer_&& other) noexcept = default;
    UltraComposer_& operator=(UltraComposer_ const& other) noexcept = default;
    ~UltraComposer_() = default;

    std::shared_ptr<ProvingKey> compute_proving_key(const CircuitBuilder& circuit_constructor);
    std::shared_ptr<VerificationKey> compute_verification_key(const CircuitBuilder& circuit_constructor);

    void compute_circuit_size_parameters(CircuitBuilder& circuit_constructor);

    void compute_witness(CircuitBuilder& circuit_constructor);

    void construct_ecc_op_wire_polynomials(auto&);

    UltraProver_<Flavor> create_prover(CircuitBuilder& circuit_constructor);
    UltraVerifier_<Flavor> create_verifier(const CircuitBuilder& circuit_constructor);

    void add_table_column_selector_poly_to_proving_key(polynomial& small, const std::string& tag);

    void compute_commitment_key(size_t circuit_size)
    {
        commitment_key = std::make_shared<CommitmentKey>(circuit_size, crs_factory_);
    };
};
extern template class UltraComposer_<honk::flavor::Ultra>;
// TODO: the UltraGrumpkin flavor still works on BN254 because plookup needs to be templated to be able to construct
// Grumpkin circuits.
extern template class UltraComposer_<honk::flavor::UltraGrumpkin>;
extern template class UltraComposer_<honk::flavor::GoblinUltra>;
// TODO(#532): this pattern is weird; is this not instantiating the templates?
using UltraComposer = UltraComposer_<honk::flavor::Ultra>;
using UltraGrumpkinComposer = UltraComposer_<honk::flavor::UltraGrumpkin>;
using GoblinUltraComposer = UltraComposer_<honk::flavor::GoblinUltra>;
} // namespace proof_system::honk
