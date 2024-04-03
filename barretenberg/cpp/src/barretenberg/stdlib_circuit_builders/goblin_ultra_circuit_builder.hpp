#pragma once
#include "barretenberg/execution_trace/execution_trace.hpp"
#include "barretenberg/plonk_honk_shared/arithmetization/arithmetization.hpp"
#include "barretenberg/stdlib_circuit_builders/op_queue/ecc_op_queue.hpp"
#include "databus.hpp"
#include "ultra_circuit_builder.hpp"

namespace bb {

using namespace bb;

template <typename FF> class GoblinUltraCircuitBuilder_ : public UltraCircuitBuilder_<UltraHonkArith<FF>> {
  public:
    using Arithmetization = UltraHonkArith<FF>;

    static constexpr std::string_view NAME_STRING = "GoblinUltraArithmetization";
    static constexpr CircuitType CIRCUIT_TYPE = CircuitType::ULTRA;
    static constexpr size_t DEFAULT_NON_NATIVE_FIELD_LIMB_BITS =
        UltraCircuitBuilder_<UltraHonkArith<FF>>::DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;

    // Stores record of ecc operations and performs corresponding native operations internally
    std::shared_ptr<ECCOpQueue> op_queue;

    // Indices for constant variables corresponding to ECCOpQueue op codes
    uint32_t null_op_idx;
    uint32_t add_accum_op_idx;
    uint32_t mul_accum_op_idx;
    uint32_t equality_op_idx;

    // Container for public calldata/returndata
    DataBus databus;

    // Functions for adding ECC op queue "gates"
    ecc_op_tuple queue_ecc_add_accum(const g1::affine_element& point);
    ecc_op_tuple queue_ecc_mul_accum(const g1::affine_element& point, const FF& scalar);
    ecc_op_tuple queue_ecc_eq();

  private:
    void populate_ecc_op_wires(const ecc_op_tuple& in);
    ecc_op_tuple decompose_ecc_operands(uint32_t op, const g1::affine_element& point, const FF& scalar = FF::zero());
    void set_goblin_ecc_op_code_constant_variables();
    uint32_t read_bus_vector(BusVector& bus_vector, const uint32_t& read_idx_witness_idx);
    void create_databus_read_gate(const databus_lookup_gate_<FF>& in);
    void create_calldata_read_gate(const databus_lookup_gate_<FF>& in);
    void create_return_data_read_gate(const databus_lookup_gate_<FF>& in);
    uint32_t append_to_bus_vector(BusVector& bus_vector, const FF& in)
    {
        const uint32_t index = this->add_variable(in);
        bus_vector.append(index);
        return index;
    }

  public:
    GoblinUltraCircuitBuilder_(const size_t size_hint = 0,
                               std::shared_ptr<ECCOpQueue> op_queue_in = std::make_shared<ECCOpQueue>())
        : UltraCircuitBuilder_<UltraHonkArith<FF>>(size_hint)
        , op_queue(op_queue_in)
    {
        // Set indices to constants corresponding to Goblin ECC op codes
        set_goblin_ecc_op_code_constant_variables();
    };
    GoblinUltraCircuitBuilder_(std::shared_ptr<ECCOpQueue> op_queue_in)
        : GoblinUltraCircuitBuilder_(0, op_queue_in)
    {}

    /**
     * @brief Constructor from data generated from ACIR
     *
     * @param op_queue_in Op queue to which goblinized group ops will be added
     * @param witness_values witnesses values known to acir
     * @param public_inputs indices of public inputs in witness array
     * @param varnum number of known witness
     *
     * @note The size of witness_values may be less than varnum. The former is the set of actual witness values known at
     * the time of acir generation. The former may be larger and essentially acounts for placeholders for witnesses that
     * we know will exist but whose values are not known during acir generation. Both are in general less than the total
     * number of variables/witnesses that might be present for a circuit generated from acir, since many gates will
     * depend on the details of the bberg implementation (or more generally on the backend used to process acir).
     */
    GoblinUltraCircuitBuilder_(std::shared_ptr<ECCOpQueue> op_queue_in,
                               auto& witness_values,
                               const std::vector<uint32_t>& public_inputs,
                               size_t varnum)
        : UltraCircuitBuilder_<UltraHonkArith<FF>>(/*size_hint=*/0, witness_values, public_inputs, varnum)
        , op_queue(op_queue_in)
    {
        // Set indices to constants corresponding to Goblin ECC op codes
        set_goblin_ecc_op_code_constant_variables();
    };

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
        auto num_ultra_gates = UltraCircuitBuilder_<UltraHonkArith<FF>>::get_num_gates();
        auto num_goblin_ecc_op_gates = this->blocks.ecc_op.size();
        return num_ultra_gates + num_goblin_ecc_op_gates;
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
        UltraCircuitBuilder_<UltraHonkArith<FF>>::get_num_gates_split_into_components(
            count, rangecount, romcount, ramcount, nnfcount);
        auto num_goblin_ecc_op_gates = this->blocks.ecc_op.size();

        size_t total = count + romcount + ramcount + rangecount + num_goblin_ecc_op_gates;
        std::cout << "gates = " << total << " (arith " << count << ", rom " << romcount << ", ram " << ramcount
                  << ", range " << rangecount << ", non native field gates " << nnfcount << ", goblin ecc op gates "
                  << num_goblin_ecc_op_gates << "), pubinp = " << this->public_inputs.size() << std::endl;
    }

    /**
     * @brief Add a witness variable to the public calldata.
     *
     * */
    uint32_t add_public_calldata(const FF& in) { return append_to_bus_vector(databus.calldata, in); }

    /**
     * @brief Add a witness variable to the public return_data.
     *
     * */
    uint32_t add_public_return_data(const FF& in) { return append_to_bus_vector(databus.return_data, in); }

    /**
     * @brief Read from calldata and create a corresponding databus read gate
     *
     * @param read_idx_witness_idx Witness index for the calldata read index
     * @return uint32_t Witness index for the result of the read
     */
    uint32_t read_calldata(const uint32_t& read_idx_witness_idx)
    {
        uint32_t value_witness_idx = read_bus_vector(databus.calldata, read_idx_witness_idx);
        create_calldata_read_gate({ read_idx_witness_idx, value_witness_idx });
        return value_witness_idx;
    };

    /**
     * @brief Read from return_data and create a corresponding databus read gate
     *
     * @param read_idx_witness_idx Witness index for the return_data read index
     * @return uint32_t Witness index for the result of the read
     */
    uint32_t read_return_data(const uint32_t& read_idx_witness_idx)
    {
        uint32_t value_witness_idx = read_bus_vector(databus.return_data, read_idx_witness_idx);
        create_return_data_read_gate({ read_idx_witness_idx, value_witness_idx });
        return value_witness_idx;
    };

    void create_poseidon2_external_gate(const poseidon2_external_gate_<FF>& in);
    void create_poseidon2_internal_gate(const poseidon2_internal_gate_<FF>& in);
};
using GoblinUltraCircuitBuilder = GoblinUltraCircuitBuilder_<bb::fr>;
} // namespace bb
