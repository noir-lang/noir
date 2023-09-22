#pragma once
#include "barretenberg/plonk/proof_system/constants.hpp"
#include "barretenberg/plonk/proof_system/types/polynomial_manifest.hpp"
#include "barretenberg/plonk/proof_system/types/prover_settings.hpp"
#include "barretenberg/polynomials/polynomial.hpp"
#include "barretenberg/proof_system/arithmetization/arithmetization.hpp"
#include "barretenberg/proof_system/op_queue/ecc_op_queue.hpp"
#include "barretenberg/proof_system/plookup_tables/plookup_tables.hpp"
#include "barretenberg/proof_system/plookup_tables/types.hpp"
#include "barretenberg/proof_system/types/merkle_hash_type.hpp"
#include "barretenberg/proof_system/types/pedersen_commitment_type.hpp"
#include "ultra_circuit_builder.hpp"
#include <optional>

namespace proof_system {

using namespace barretenberg;

template <typename FF> class GoblinUltraCircuitBuilder_ : public UltraCircuitBuilder_<FF> {
  public:
    static constexpr std::string_view NAME_STRING = "GoblinUltraArithmetization";
    static constexpr CircuitType CIRCUIT_TYPE = CircuitType::ULTRA;
    static constexpr size_t DEFAULT_NON_NATIVE_FIELD_LIMB_BITS =
        UltraCircuitBuilder_<FF>::DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;

    size_t num_ecc_op_gates = 0; // number of ecc op "gates" (rows); these are placed at the start of the circuit

    // Stores record of ecc operations and performs corresponding native operations internally
    std::shared_ptr<ECCOpQueue> op_queue;

    // Indices for constant variables corresponding to ECCOpQueue op codes
    uint32_t null_op_idx;
    uint32_t add_accum_op_idx;
    uint32_t mul_accum_op_idx;
    uint32_t equality_op_idx;

    using WireVector = std::vector<uint32_t, ContainerSlabAllocator<uint32_t>>;

    // Wires storing ecc op queue data; values are indices into the variables array
    std::array<WireVector, arithmetization::Ultra<FF>::NUM_WIRES> ecc_op_wires;

    WireVector& ecc_op_wire_1 = std::get<0>(ecc_op_wires);
    WireVector& ecc_op_wire_2 = std::get<1>(ecc_op_wires);
    WireVector& ecc_op_wire_3 = std::get<2>(ecc_op_wires);
    WireVector& ecc_op_wire_4 = std::get<3>(ecc_op_wires);

    // Functions for adding ECC op queue "gates"
    ecc_op_tuple queue_ecc_add_accum(const g1::affine_element& point);
    ecc_op_tuple queue_ecc_mul_accum(const g1::affine_element& point, const FF& scalar);
    ecc_op_tuple queue_ecc_eq();

  private:
    void populate_ecc_op_wires(const ecc_op_tuple& in);
    ecc_op_tuple decompose_ecc_operands(uint32_t op, const g1::affine_element& point, const FF& scalar = FF::zero());

  public:
    GoblinUltraCircuitBuilder_(const size_t size_hint = 0,
                               std::shared_ptr<ECCOpQueue> op_queue_in = std::make_shared<ECCOpQueue>())
        : UltraCircuitBuilder_<FF>(size_hint)
        , op_queue(op_queue_in)
    {
        // Set indices to constants corresponding to Goblin ECC op codes
        null_op_idx = this->zero_idx;
        add_accum_op_idx = this->put_constant_variable(FF(EccOpCode::ADD_ACCUM));
        mul_accum_op_idx = this->put_constant_variable(FF(EccOpCode::MUL_ACCUM));
        equality_op_idx = this->put_constant_variable(FF(EccOpCode::EQUALITY));
    };
    GoblinUltraCircuitBuilder_(std::shared_ptr<ECCOpQueue> op_queue_in)
        : GoblinUltraCircuitBuilder_(0, op_queue_in)
    {}

    void finalize_circuit();
    void add_gates_to_ensure_all_polys_are_non_zero();

    size_t get_num_constant_gates() const override { return 0; }

    /**
     * @brief Get the final number of gates in a circuit, which consists of the sum of:
     * 1) Current number number of actual gates
     * 2) Number of public inputs, as we'll need to add a gate for each of them
     * 3) Number of Rom array-associated gates
     * 4) Number of range-list associated gates
     * 5) Number of non-native field multiplication gates.
     *
     * @return size_t
     */
    size_t get_num_gates() const override
    {
        auto num_ultra_gates = UltraCircuitBuilder_<FF>::get_num_gates();
        return num_ultra_gates + num_ecc_op_gates;
    }

    /**x
     * @brief Print the number and composition of gates in the circuit
     *
     */
    virtual void print_num_gates() const override
    {
        size_t count = 0;
        size_t rangecount = 0;
        size_t romcount = 0;
        size_t ramcount = 0;
        size_t nnfcount = 0;
        UltraCircuitBuilder_<FF>::get_num_gates_split_into_components(count, rangecount, romcount, ramcount, nnfcount);

        size_t total = count + romcount + ramcount + rangecount + num_ecc_op_gates;
        std::cout << "gates = " << total << " (arith " << count << ", rom " << romcount << ", ram " << ramcount
                  << ", range " << rangecount << ", non native field gates " << nnfcount << ", goblin ecc op gates "
                  << num_ecc_op_gates << "), pubinp = " << this->public_inputs.size() << std::endl;
    }
};
extern template class GoblinUltraCircuitBuilder_<barretenberg::fr>;
using GoblinUltraCircuitBuilder = GoblinUltraCircuitBuilder_<barretenberg::fr>;
} // namespace proof_system
