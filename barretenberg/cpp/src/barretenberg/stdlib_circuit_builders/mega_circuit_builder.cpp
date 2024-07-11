#include "mega_circuit_builder.hpp"
#include "barretenberg/crypto/poseidon2/poseidon2_params.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include <barretenberg/plonk/proof_system/constants.hpp>
#include <unordered_map>
#include <unordered_set>

using namespace bb;
using namespace bb::crypto;

namespace bb {

template <typename FF> void MegaCircuitBuilder_<FF>::finalize_circuit()
{
    // All of the gates involved in finalization are part of the Ultra arithmetization
    UltraCircuitBuilder_<MegaArith<FF>>::finalize_circuit();
}

/**
 * @brief Ensure all polynomials have at least one non-zero coefficient to avoid commiting to the zero-polynomial.
 *        This only adds gates for the Goblin polynomials. Most polynomials are handled via the Ultra method,
 *        which should be done by a separate call to the Ultra builder's non zero polynomial gates method.
 *
 * @param in Structure containing variables and witness selectors
 */
// TODO(#423): This function adds valid (but arbitrary) gates to ensure that the circuit which includes
// them will not result in any zero-polynomials. It also ensures that the first coefficient of the wire
// polynomials is zero, which is required for them to be shiftable.
template <typename FF> void MegaCircuitBuilder_<FF>::add_gates_to_ensure_all_polys_are_non_zero()
{
    // Most polynomials are handled via the conventional Ultra method
    UltraCircuitBuilder_<MegaArith<FF>>::add_gates_to_ensure_all_polys_are_non_zero();

    // All that remains is to handle databus related and poseidon2 related polynomials. In what follows we populate the
    // calldata with some mock data then constuct a single calldata read gate

    // Create an arbitrary calldata read gate
    add_public_calldata(this->add_variable(25)); // ensure there is at least one entry in calldata
    auto raw_read_idx = static_cast<uint32_t>(get_calldata().size()) - 1; // read data that was just added
    auto read_idx = this->add_variable(raw_read_idx);
    read_calldata(read_idx);

    // Create an arbitrary return data read gate
    add_public_return_data(this->add_variable(17)); // ensure there is at least one entry in return data
    raw_read_idx = static_cast<uint32_t>(get_return_data().size()) - 1; // read data that was just added
    read_idx = this->add_variable(raw_read_idx);
    read_return_data(read_idx);

    // mock a poseidon external gate, with all zeros as input
    this->blocks.poseidon_external.populate_wires(this->zero_idx, this->zero_idx, this->zero_idx, this->zero_idx);
    this->blocks.poseidon_external.q_m().emplace_back(0);
    this->blocks.poseidon_external.q_1().emplace_back(0);
    this->blocks.poseidon_external.q_2().emplace_back(0);
    this->blocks.poseidon_external.q_3().emplace_back(0);
    this->blocks.poseidon_external.q_c().emplace_back(0);
    this->blocks.poseidon_external.q_arith().emplace_back(0);
    this->blocks.poseidon_external.q_4().emplace_back(0);
    this->blocks.poseidon_external.q_delta_range().emplace_back(0);
    this->blocks.poseidon_external.q_lookup_type().emplace_back(0);
    this->blocks.poseidon_external.q_elliptic().emplace_back(0);
    this->blocks.poseidon_external.q_aux().emplace_back(0);
    this->blocks.poseidon_external.q_busread().emplace_back(0);
    this->blocks.poseidon_external.q_poseidon2_external().emplace_back(1);
    this->blocks.poseidon_external.q_poseidon2_internal().emplace_back(0);
    this->check_selector_length_consistency();
    ++this->num_gates;

    // dummy gate to be read into by previous poseidon external gate via shifts
    this->create_dummy_gate(
        this->blocks.poseidon_external, this->zero_idx, this->zero_idx, this->zero_idx, this->zero_idx);

    // mock a poseidon internal gate, with all zeros as input
    this->blocks.poseidon_internal.populate_wires(this->zero_idx, this->zero_idx, this->zero_idx, this->zero_idx);
    this->blocks.poseidon_internal.q_m().emplace_back(0);
    this->blocks.poseidon_internal.q_1().emplace_back(0);
    this->blocks.poseidon_internal.q_2().emplace_back(0);
    this->blocks.poseidon_internal.q_3().emplace_back(0);
    this->blocks.poseidon_internal.q_c().emplace_back(0);
    this->blocks.poseidon_internal.q_arith().emplace_back(0);
    this->blocks.poseidon_internal.q_4().emplace_back(0);
    this->blocks.poseidon_internal.q_delta_range().emplace_back(0);
    this->blocks.poseidon_internal.q_lookup_type().emplace_back(0);
    this->blocks.poseidon_internal.q_elliptic().emplace_back(0);
    this->blocks.poseidon_internal.q_aux().emplace_back(0);
    this->blocks.poseidon_internal.q_busread().emplace_back(0);
    this->blocks.poseidon_internal.q_poseidon2_external().emplace_back(0);
    this->blocks.poseidon_internal.q_poseidon2_internal().emplace_back(1);
    this->check_selector_length_consistency();
    ++this->num_gates;

    // dummy gate to be read into by previous poseidon internal gate via shifts
    this->create_dummy_gate(
        this->blocks.poseidon_internal, this->zero_idx, this->zero_idx, this->zero_idx, this->zero_idx);

    // add dummy mul accum op and an equality op
    this->queue_ecc_mul_accum(bb::g1::affine_element::one(), 2);
    this->queue_ecc_eq();
}

/**
 * @brief Add simple point addition operation to the op queue and add corresponding gates
 *
 * @param point Point to be added into the accumulator
 */
template <typename FF> ecc_op_tuple MegaCircuitBuilder_<FF>::queue_ecc_add_accum(const bb::g1::affine_element& point)
{
    // Add the operation to the op queue
    auto ultra_op = op_queue->add_accumulate(point);

    // Add corresponding gates for the operation
    ecc_op_tuple op_tuple = populate_ecc_op_wires(ultra_op);
    return op_tuple;
}

/**
 * @brief Add point mul-then-accumulate operation to the op queue and add corresponding gates
 *
 * @tparam FF
 * @param point
 * @param scalar The scalar by which point is multiplied prior to being accumulated
 * @return ecc_op_tuple encoding the point and scalar inputs to the mul accum
 */
template <typename FF>
ecc_op_tuple MegaCircuitBuilder_<FF>::queue_ecc_mul_accum(const bb::g1::affine_element& point, const FF& scalar)
{
    // Add the operation to the op queue
    auto ultra_op = op_queue->mul_accumulate(point, scalar);

    // Add corresponding gates for the operation
    ecc_op_tuple op_tuple = populate_ecc_op_wires(ultra_op);
    return op_tuple;
}

/**
 * @brief Add point equality operation to the op queue based on the value of the internal accumulator and add
 * corresponding gates
 *
 * @return ecc_op_tuple encoding the point to which equality has been asserted
 */
template <typename FF> ecc_op_tuple MegaCircuitBuilder_<FF>::queue_ecc_eq()
{
    // Add the operation to the op queue
    auto ultra_op = op_queue->eq_and_reset();

    // Add corresponding gates for the operation
    ecc_op_tuple op_tuple = populate_ecc_op_wires(ultra_op);
    op_tuple.return_is_infinity = ultra_op.return_is_infinity;
    return op_tuple;
}

/**
 * @brief Add goblin ecc op gates for a single operation
 *
 * @param ultra_op Operation data expressed in the ultra format
 * @note All selectors are set to 0 since the ecc op selector is derived later based on the block size/location.
 */
template <typename FF> ecc_op_tuple MegaCircuitBuilder_<FF>::populate_ecc_op_wires(const UltraOp& ultra_op)
{
    ecc_op_tuple op_tuple;
    op_tuple.op = get_ecc_op_idx(ultra_op.op_code);
    op_tuple.x_lo = this->add_variable(ultra_op.x_lo);
    op_tuple.x_hi = this->add_variable(ultra_op.x_hi);
    op_tuple.y_lo = this->add_variable(ultra_op.y_lo);
    op_tuple.y_hi = this->add_variable(ultra_op.y_hi);
    op_tuple.z_1 = this->add_variable(ultra_op.z_1);
    op_tuple.z_2 = this->add_variable(ultra_op.z_2);

    this->blocks.ecc_op.populate_wires(op_tuple.op, op_tuple.x_lo, op_tuple.x_hi, op_tuple.y_lo);
    for (auto& selector : this->blocks.ecc_op.selectors) {
        selector.emplace_back(0);
    }

    this->blocks.ecc_op.populate_wires(this->zero_idx, op_tuple.y_hi, op_tuple.z_1, op_tuple.z_2);
    for (auto& selector : this->blocks.ecc_op.selectors) {
        selector.emplace_back(0);
    }

    return op_tuple;
};

template <typename FF> void MegaCircuitBuilder_<FF>::set_goblin_ecc_op_code_constant_variables()
{
    null_op_idx = this->zero_idx;
    add_accum_op_idx = this->put_constant_variable(FF(EccOpCode::ADD_ACCUM));
    mul_accum_op_idx = this->put_constant_variable(FF(EccOpCode::MUL_ACCUM));
    equality_op_idx = this->put_constant_variable(FF(EccOpCode::EQUALITY));
}

/**
 * @brief Read from a databus column
 * @details Creates a databus lookup gate based on the input index and read result
 *
 * @tparam FF
 * @param read_idx_witness_idx Variable index of the read index
 * @return uint32_t Variable index of the result of the read
 */
template <typename FF>
uint32_t MegaCircuitBuilder_<FF>::read_bus_vector(BusId bus_idx, const uint32_t& read_idx_witness_idx)
{
    auto& bus_vector = databus[static_cast<size_t>(bus_idx)];
    // Get the raw index into the databus column
    const uint32_t read_idx = static_cast<uint32_t>(uint256_t(this->get_variable(read_idx_witness_idx)));

    ASSERT(read_idx < bus_vector.size()); // Ensure that the read index is valid
    // NOTE(https://github.com/AztecProtocol/barretenberg/issues/937): Multiple reads at same index is not supported.
    ASSERT(bus_vector.get_read_count(read_idx) < 1);

    // Create a variable corresponding to the result of the read. Note that we do not in general connect reads from
    // databus via copy constraints (i.e. we create a unique variable for the result of each read)
    FF value = this->get_variable(bus_vector[read_idx]);
    uint32_t value_witness_idx = this->add_variable(value);

    create_databus_read_gate({ read_idx_witness_idx, value_witness_idx }, bus_idx);
    bus_vector.increment_read_count(read_idx);

    return value_witness_idx;
}

/**
 * @brief Create a databus lookup/read gate
 *
 * @tparam FF
 * @param databus_lookup_gate_ witness indices corresponding to: read index, result value
 */
template <typename FF>
void MegaCircuitBuilder_<FF>::create_databus_read_gate(const databus_lookup_gate_<FF>& in, const BusId bus_idx)
{
    auto& block = this->blocks.busread;
    block.populate_wires(in.value, in.index, this->zero_idx, this->zero_idx);
    apply_databus_selectors(bus_idx);

    this->check_selector_length_consistency();
    ++this->num_gates;
}

template <typename FF> void MegaCircuitBuilder_<FF>::apply_databus_selectors(const BusId bus_idx)
{
    auto& block = this->blocks.busread;
    switch (bus_idx) {
    case BusId::CALLDATA: {
        block.q_1().emplace_back(1);
        block.q_2().emplace_back(0);
        break;
    }
    case BusId::RETURNDATA: {
        block.q_1().emplace_back(0);
        block.q_2().emplace_back(1);
        break;
    }
    }
    block.q_busread().emplace_back(1);
    block.q_m().emplace_back(0);
    block.q_3().emplace_back(0);
    block.q_c().emplace_back(0);
    block.q_delta_range().emplace_back(0);
    block.q_arith().emplace_back(0);
    block.q_4().emplace_back(0);
    block.q_lookup_type().emplace_back(0);
    block.q_elliptic().emplace_back(0);
    block.q_aux().emplace_back(0);
    block.q_poseidon2_external().emplace_back(0);
    block.q_poseidon2_internal().emplace_back(0);
}

/**
 * @brief Poseidon2 external round gate, activates the q_poseidon2_external selector and relation
 */
template <typename FF>
void MegaCircuitBuilder_<FF>::create_poseidon2_external_gate(const poseidon2_external_gate_<FF>& in)
{
    auto& block = this->blocks.poseidon_external;
    block.populate_wires(in.a, in.b, in.c, in.d);
    block.q_m().emplace_back(0);
    block.q_1().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][0]);
    block.q_2().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][1]);
    block.q_3().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][2]);
    block.q_c().emplace_back(0);
    block.q_arith().emplace_back(0);
    block.q_4().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][3]);
    block.q_delta_range().emplace_back(0);
    block.q_lookup_type().emplace_back(0);
    block.q_elliptic().emplace_back(0);
    block.q_aux().emplace_back(0);
    block.q_busread().emplace_back(0);
    block.q_poseidon2_external().emplace_back(1);
    block.q_poseidon2_internal().emplace_back(0);
    this->check_selector_length_consistency();
    ++this->num_gates;
}

/**
 * @brief Poseidon2 internal round gate, activates the q_poseidon2_internal selector and relation
 */
template <typename FF>
void MegaCircuitBuilder_<FF>::create_poseidon2_internal_gate(const poseidon2_internal_gate_<FF>& in)
{
    auto& block = this->blocks.poseidon_internal;
    block.populate_wires(in.a, in.b, in.c, in.d);
    block.q_m().emplace_back(0);
    block.q_1().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][0]);
    block.q_2().emplace_back(0);
    block.q_3().emplace_back(0);
    block.q_c().emplace_back(0);
    block.q_arith().emplace_back(0);
    block.q_4().emplace_back(0);
    block.q_delta_range().emplace_back(0);
    block.q_lookup_type().emplace_back(0);
    block.q_elliptic().emplace_back(0);
    block.q_aux().emplace_back(0);
    block.q_busread().emplace_back(0);
    block.q_poseidon2_external().emplace_back(0);
    block.q_poseidon2_internal().emplace_back(1);
    this->check_selector_length_consistency();
    ++this->num_gates;
}

template class MegaCircuitBuilder_<bb::fr>;
} // namespace bb