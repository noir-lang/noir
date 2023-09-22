#include "goblin_ultra_circuit_builder.hpp"
#include <barretenberg/plonk/proof_system/constants.hpp>
#include <unordered_map>
#include <unordered_set>

using namespace barretenberg;

namespace proof_system {

template <typename FF> void GoblinUltraCircuitBuilder_<FF>::finalize_circuit()
{
    UltraCircuitBuilder_<FF>::finalize_circuit();

    // Set internally the current and previous size of the aggregate op queue transcript
    op_queue->set_size_data();
}

/**
 * @brief Ensure all polynomials have at least one non-zero coefficient to avoid commiting to the zero-polynomial
 *
 * @param in Structure containing variables and witness selectors
 */
// TODO(#423): This function adds valid (but arbitrary) gates to ensure that the circuit which includes
// them will not result in any zero-polynomials. It also ensures that the first coefficient of the wire
// polynomials is zero, which is required for them to be shiftable.
template <typename FF> void GoblinUltraCircuitBuilder_<FF>::add_gates_to_ensure_all_polys_are_non_zero()
{
    UltraCircuitBuilder_<FF>::add_gates_to_ensure_all_polys_are_non_zero();
}

/**
 * @brief Add gates for simple point addition (no mul) and add the raw operation data to the op queue
 *
 * @param point Point to be added into the accumulator
 */
template <typename FF>
ecc_op_tuple GoblinUltraCircuitBuilder_<FF>::queue_ecc_add_accum(const barretenberg::g1::affine_element& point)
{
    // Add raw op to queue
    op_queue->add_accumulate(point);

    // Decompose operation inputs into width-four form and add ecc op gates
    auto op_tuple = decompose_ecc_operands(add_accum_op_idx, point);
    populate_ecc_op_wires(op_tuple);

    return op_tuple;
}

/**
 * @brief Add gates for point mul-then-accumulate and add the raw operation data to the op queue
 *
 * @tparam FF
 * @param point
 * @param scalar The scalar by which point is multiplied prior to being accumulated
 * @return ecc_op_tuple encoding the point and scalar inputs to the mul accum
 */
template <typename FF>
ecc_op_tuple GoblinUltraCircuitBuilder_<FF>::queue_ecc_mul_accum(const barretenberg::g1::affine_element& point,
                                                                 const FF& scalar)
{
    // Add raw op to op queue
    op_queue->mul_accumulate(point, scalar);

    // Decompose operation inputs into width-four form and add ecc op gates
    auto op_tuple = decompose_ecc_operands(mul_accum_op_idx, point, scalar);
    populate_ecc_op_wires(op_tuple);

    return op_tuple;
}

/**
 * @brief Add point equality gates based on the current value of the accumulator internal to the op queue and add the
 * raw operation data to the op queue
 *
 * @return ecc_op_tuple encoding the point to which equality has been asserted
 */
template <typename FF> ecc_op_tuple GoblinUltraCircuitBuilder_<FF>::queue_ecc_eq()
{
    // Add raw op to op queue
    auto point = op_queue->eq();

    // Decompose operation inputs into width-four form and add ecc op gates
    auto op_tuple = decompose_ecc_operands(equality_op_idx, point);
    populate_ecc_op_wires(op_tuple);

    return op_tuple;
}

/**
 * @brief Decompose ecc operands into components, add corresponding variables, return ecc op tuple
 *
 * @param op_idx Index of op code in variables array
 * @param point
 * @param scalar
 * @return ecc_op_tuple Tuple of indices into variables array used to construct pair of ecc op gates
 */
template <typename FF>
ecc_op_tuple GoblinUltraCircuitBuilder_<FF>::decompose_ecc_operands(uint32_t op_idx,
                                                                    const g1::affine_element& point,
                                                                    const FF& scalar)
{
    // Decompose point coordinates (Fq) into hi-lo chunks (Fr)
    const size_t CHUNK_SIZE = 2 * DEFAULT_NON_NATIVE_FIELD_LIMB_BITS;
    auto x_256 = uint256_t(point.x);
    auto y_256 = uint256_t(point.y);
    auto x_lo = FF(x_256.slice(0, CHUNK_SIZE));
    auto x_hi = FF(x_256.slice(CHUNK_SIZE, CHUNK_SIZE * 2));
    auto y_lo = FF(y_256.slice(0, CHUNK_SIZE));
    auto y_hi = FF(y_256.slice(CHUNK_SIZE, CHUNK_SIZE * 2));

    // Split scalar into 128 bit endomorphism scalars
    FF z_1 = 0;
    FF z_2 = 0;
    auto converted = scalar.from_montgomery_form();
    FF::split_into_endomorphism_scalars(converted, z_1, z_2);
    z_1 = z_1.to_montgomery_form();
    z_2 = z_2.to_montgomery_form();

    // Populate ultra ops in OpQueue with the decomposed operands
    op_queue->ultra_ops[0].emplace_back(this->variables[op_idx]);
    op_queue->ultra_ops[1].emplace_back(x_lo);
    op_queue->ultra_ops[2].emplace_back(x_hi);
    op_queue->ultra_ops[3].emplace_back(y_lo);

    op_queue->ultra_ops[0].emplace_back(this->zero_idx);
    op_queue->ultra_ops[1].emplace_back(y_hi);
    op_queue->ultra_ops[2].emplace_back(z_1);
    op_queue->ultra_ops[3].emplace_back(z_2);

    // Add variables for decomposition and get indices needed for op wires
    auto x_lo_idx = this->add_variable(x_lo);
    auto x_hi_idx = this->add_variable(x_hi);
    auto y_lo_idx = this->add_variable(y_lo);
    auto y_hi_idx = this->add_variable(y_hi);
    auto z_1_idx = this->add_variable(z_1);
    auto z_2_idx = this->add_variable(z_2);

    return { op_idx, x_lo_idx, x_hi_idx, y_lo_idx, y_hi_idx, z_1_idx, z_2_idx };
}

/**
 * @brief Add ecc operation to queue
 *
 * @param in Variables array indices corresponding to operation inputs
 * @note We dont explicitly set values for the selectors here since their values are fully determined by
 * num_ecc_op_gates. E.g. in the composer we can reconstruct q_ecc_op as the indicator on the first num_ecc_op_gates
 * indices. All other selectors are simply 0 on this domain.
 */
template <typename FF> void GoblinUltraCircuitBuilder_<FF>::populate_ecc_op_wires(const ecc_op_tuple& in)
{
    ecc_op_wire_1.emplace_back(in.op);
    ecc_op_wire_2.emplace_back(in.x_lo);
    ecc_op_wire_3.emplace_back(in.x_hi);
    ecc_op_wire_4.emplace_back(in.y_lo);

    ecc_op_wire_1.emplace_back(this->zero_idx);
    ecc_op_wire_2.emplace_back(in.y_hi);
    ecc_op_wire_3.emplace_back(in.z_1);
    ecc_op_wire_4.emplace_back(in.z_2);

    num_ecc_op_gates += 2;
};

template class GoblinUltraCircuitBuilder_<barretenberg::fr>;
} // namespace proof_system