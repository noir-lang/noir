#include "goblin_ultra_circuit_builder.hpp"
#include "barretenberg/crypto/poseidon2/poseidon2_params.hpp"
#include "barretenberg/flavor/goblin_ultra.hpp"
#include <barretenberg/plonk/proof_system/constants.hpp>
#include <unordered_map>
#include <unordered_set>

using namespace bb;
using namespace bb::crypto;

namespace bb {

template <typename FF> void GoblinUltraCircuitBuilder_<FF>::finalize_circuit()
{
    UltraCircuitBuilder_<arithmetization::UltraHonk<FF>>::finalize_circuit();
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
    // Most polynomials are handled via the conventional Ultra method
    UltraCircuitBuilder_<arithmetization::UltraHonk<FF>>::add_gates_to_ensure_all_polys_are_non_zero();

    // All that remains is to handle databus related and poseidon2 related polynomials. In what follows we populate the
    // calldata with some mock data then constuct a single calldata read gate

    // Create an arbitrary calldata read gate
    add_public_calldata(FF(25)); // ensure there is at least one entry in calldata
    uint32_t raw_read_idx = 0;   // read first entry in calldata
    auto read_idx = this->add_variable(raw_read_idx);
    FF calldata_value = this->get_variable(public_calldata[raw_read_idx]);
    auto value_idx = this->add_variable(calldata_value);
    create_calldata_lookup_gate({ read_idx, value_idx });
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/821): automate updating of read counts
    calldata_read_counts[raw_read_idx]++;

    // mock gates that use poseidon selectors, with all zeros as input
    this->w_l().emplace_back(this->zero_idx);
    this->w_r().emplace_back(this->zero_idx);
    this->w_o().emplace_back(this->zero_idx);
    this->w_4().emplace_back(this->zero_idx);
    this->q_m().emplace_back(0);
    this->q_1().emplace_back(0);
    this->q_2().emplace_back(0);
    this->q_3().emplace_back(0);
    this->q_c().emplace_back(0);
    this->q_arith().emplace_back(0);
    this->q_4().emplace_back(0);
    this->q_sort().emplace_back(0);
    this->q_lookup_type().emplace_back(0);
    this->q_elliptic().emplace_back(0);
    this->q_aux().emplace_back(0);
    this->q_busread().emplace_back(0);
    this->q_poseidon2_external().emplace_back(1);
    this->q_poseidon2_internal().emplace_back(1);

    ++this->num_gates;

    // second gate that stores the output of all zeros of the poseidon gates
    this->w_l().emplace_back(this->zero_idx);
    this->w_r().emplace_back(this->zero_idx);
    this->w_o().emplace_back(this->zero_idx);
    this->w_4().emplace_back(this->zero_idx);
    this->q_m().emplace_back(0);
    this->q_1().emplace_back(0);
    this->q_2().emplace_back(0);
    this->q_3().emplace_back(0);
    this->q_c().emplace_back(0);
    this->q_arith().emplace_back(0);
    this->q_4().emplace_back(0);
    this->q_sort().emplace_back(0);
    this->q_lookup_type().emplace_back(0);
    this->q_elliptic().emplace_back(0);
    this->q_aux().emplace_back(0);
    this->q_busread().emplace_back(0);
    this->q_poseidon2_external().emplace_back(0);
    this->q_poseidon2_internal().emplace_back(0);

    ++this->num_gates;
}

/**
 * @brief Add gates for simple point addition (no mul) and add the raw operation data to the op queue
 *
 * @param point Point to be added into the accumulator
 */
template <typename FF>
ecc_op_tuple GoblinUltraCircuitBuilder_<FF>::queue_ecc_add_accum(const bb::g1::affine_element& point)
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
ecc_op_tuple GoblinUltraCircuitBuilder_<FF>::queue_ecc_mul_accum(const bb::g1::affine_element& point, const FF& scalar)
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
    ecc_op_wire_1().emplace_back(in.op);
    ecc_op_wire_2().emplace_back(in.x_lo);
    ecc_op_wire_3().emplace_back(in.x_hi);
    ecc_op_wire_4().emplace_back(in.y_lo);

    ecc_op_wire_1().emplace_back(this->zero_idx);
    ecc_op_wire_2().emplace_back(in.y_hi);
    ecc_op_wire_3().emplace_back(in.z_1);
    ecc_op_wire_4().emplace_back(in.z_2);

    num_ecc_op_gates += 2;
};

template <typename FF> void GoblinUltraCircuitBuilder_<FF>::set_goblin_ecc_op_code_constant_variables()
{
    null_op_idx = this->zero_idx;
    add_accum_op_idx = this->put_constant_variable(FF(EccOpCode::ADD_ACCUM));
    mul_accum_op_idx = this->put_constant_variable(FF(EccOpCode::MUL_ACCUM));
    equality_op_idx = this->put_constant_variable(FF(EccOpCode::EQUALITY));
}

/**
 * @brief Create a calldata lookup/read gate
 *
 * @tparam FF
 * @param databus_lookup_gate_ witness indices corresponding to: calldata index, calldata value
 */
template <typename FF>
void GoblinUltraCircuitBuilder_<FF>::create_calldata_lookup_gate(const databus_lookup_gate_<FF>& in)
{
    this->w_l().emplace_back(in.value);
    this->w_r().emplace_back(in.index);
    q_busread().emplace_back(1);

    // populate all other components with zero
    this->w_o().emplace_back(this->zero_idx);
    this->w_4().emplace_back(this->zero_idx);
    this->q_m().emplace_back(0);
    this->q_1().emplace_back(0);
    this->q_2().emplace_back(0);
    this->q_3().emplace_back(0);
    this->q_c().emplace_back(0);
    this->q_sort().emplace_back(0);
    this->q_arith().emplace_back(0);
    this->q_4().emplace_back(0);
    this->q_lookup_type().emplace_back(0);
    this->q_elliptic().emplace_back(0);
    this->q_aux().emplace_back(0);
    this->q_poseidon2_external().emplace_back(0);
    this->q_poseidon2_internal().emplace_back(0);

    ++this->num_gates;
}

/**
 * @brief Poseidon2 external round gate, activates the q_poseidon2_external selector and relation
 */
template <typename FF>
void GoblinUltraCircuitBuilder_<FF>::create_poseidon2_external_gate(const poseidon2_external_gate_<FF>& in)
{
    this->w_l().emplace_back(in.a);
    this->w_r().emplace_back(in.b);
    this->w_o().emplace_back(in.c);
    this->w_4().emplace_back(in.d);
    this->q_m().emplace_back(0);
    this->q_1().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][0]);
    this->q_2().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][1]);
    this->q_3().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][2]);
    this->q_c().emplace_back(0);
    this->q_arith().emplace_back(0);
    this->q_4().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][3]);
    this->q_sort().emplace_back(0);
    this->q_lookup_type().emplace_back(0);
    this->q_elliptic().emplace_back(0);
    this->q_aux().emplace_back(0);
    this->q_busread().emplace_back(0);
    this->q_poseidon2_external().emplace_back(1);
    this->q_poseidon2_internal().emplace_back(0);
    ++this->num_gates;
}

/**
 * @brief Poseidon2 internal round gate, activates the q_poseidon2_internal selector and relation
 */
template <typename FF>
void GoblinUltraCircuitBuilder_<FF>::create_poseidon2_internal_gate(const poseidon2_internal_gate_<FF>& in)
{
    this->w_l().emplace_back(in.a);
    this->w_r().emplace_back(in.b);
    this->w_o().emplace_back(in.c);
    this->w_4().emplace_back(in.d);
    this->q_m().emplace_back(0);
    this->q_1().emplace_back(Poseidon2Bn254ScalarFieldParams::round_constants[in.round_idx][0]);
    this->q_2().emplace_back(0);
    this->q_3().emplace_back(0);
    this->q_c().emplace_back(0);
    this->q_arith().emplace_back(0);
    this->q_4().emplace_back(0);
    this->q_sort().emplace_back(0);
    this->q_lookup_type().emplace_back(0);
    this->q_elliptic().emplace_back(0);
    this->q_aux().emplace_back(0);
    this->q_busread().emplace_back(0);
    this->q_poseidon2_external().emplace_back(0);
    this->q_poseidon2_internal().emplace_back(1);
    ++this->num_gates;
}

/**
 * @brief Poseidon2 end round gate, needed because poseidon2 rounds compare with shifted wires
 * @details The Poseidon2 permutation is 64 rounds, but needs to be a block of 65 rows, since the result of applying a
 * round of Poseidon2 is stored in the next row (the shifted row). As a result, we need this end row to compare with the
 * result from the 64th round of Poseidon2. Note that it does not activate any selectors since it only serves as a
 * comparison through the shifted wires.
 */
template <typename FF> void GoblinUltraCircuitBuilder_<FF>::create_poseidon2_end_gate(const poseidon2_end_gate_<FF>& in)
{
    this->w_l().emplace_back(in.a);
    this->w_r().emplace_back(in.b);
    this->w_o().emplace_back(in.c);
    this->w_4().emplace_back(in.d);
    this->q_m().emplace_back(0);
    this->q_1().emplace_back(0);
    this->q_2().emplace_back(0);
    this->q_3().emplace_back(0);
    this->q_c().emplace_back(0);
    this->q_arith().emplace_back(0);
    this->q_4().emplace_back(0);
    this->q_sort().emplace_back(0);
    this->q_lookup_type().emplace_back(0);
    this->q_elliptic().emplace_back(0);
    this->q_aux().emplace_back(0);
    this->q_busread().emplace_back(0);
    this->q_poseidon2_external().emplace_back(0);
    this->q_poseidon2_internal().emplace_back(0);
    ++this->num_gates;
}

template <typename FF>
inline FF GoblinUltraCircuitBuilder_<FF>::compute_poseidon2_external_identity(FF q_poseidon2_external_value,
                                                                              FF q_1_value,
                                                                              FF q_2_value,
                                                                              FF q_3_value,
                                                                              FF q_4_value,
                                                                              FF w_1_value,
                                                                              FF w_2_value,
                                                                              FF w_3_value,
                                                                              FF w_4_value,
                                                                              FF w_1_shifted_value,
                                                                              FF w_2_shifted_value,
                                                                              FF w_3_shifted_value,
                                                                              FF w_4_shifted_value,
                                                                              FF alpha_base,
                                                                              FF alpha) const
{
    // Power of alpha to separate individual sub-relations
    // TODO(kesha): This is a repeated computation which can be efficiently optimized
    const FF alpha_a = alpha_base;
    const FF alpha_b = alpha_a * alpha;
    const FF alpha_c = alpha_b * alpha;
    const FF alpha_d = alpha_c * alpha;

    FF s1 = w_1_value + q_1_value;
    FF s2 = w_2_value + q_2_value;
    FF s3 = w_3_value + q_3_value;
    FF s4 = w_4_value + q_4_value;

    FF u1 = s1 * s1;
    u1 *= u1;
    u1 *= s1;
    FF u2 = s2 * s2;
    u2 *= u2;
    u2 *= s2;
    FF u3 = s3 * s3;
    u3 *= u3;
    u3 *= s3;
    FF u4 = s4 * s4;
    u4 *= u4;
    u4 *= s4;

    auto t0 = u1 + u2;
    auto t1 = u3 + u4;
    auto t2 = u2 + u2;
    t2 += t1;
    auto t3 = u4 + u4;
    t3 += t0;
    auto v4 = t1 + t1;
    v4 += v4;
    v4 += t3;
    auto v2 = t0 + t0;
    v2 += v2;
    v2 += t2;
    auto v1 = t3 + v2;
    auto v3 = t2 + v4;

    return q_poseidon2_external_value * (alpha_a * (v1 - w_1_shifted_value) + alpha_b * (v2 - w_2_shifted_value) +
                                         alpha_c * (v3 - w_3_shifted_value) + alpha_d * (v4 - w_4_shifted_value));
}

template <typename FF>
inline FF GoblinUltraCircuitBuilder_<FF>::compute_poseidon2_internal_identity(FF q_poseidon2_internal_value,
                                                                              FF q_1_value,
                                                                              FF w_1_value,
                                                                              FF w_2_value,
                                                                              FF w_3_value,
                                                                              FF w_4_value,
                                                                              FF w_1_shifted_value,
                                                                              FF w_2_shifted_value,
                                                                              FF w_3_shifted_value,
                                                                              FF w_4_shifted_value,
                                                                              FF alpha_base,
                                                                              FF alpha) const
{
    // Power of alpha to separate individual sub-relations
    // TODO(kesha): This is a repeated computation which can be efficiently optimized
    const FF alpha_a = alpha_base;
    const FF alpha_b = alpha_a * alpha;
    const FF alpha_c = alpha_b * alpha;
    const FF alpha_d = alpha_c * alpha;

    auto s1 = w_1_value + q_1_value;

    auto u1 = s1 * s1;
    u1 *= u1;
    u1 *= s1;

    auto sum = u1 + w_2_value + w_3_value + w_4_value;
    auto v1 = u1 * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[0];
    v1 += sum;
    auto v2 = w_2_value * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[1];
    v2 += sum;
    auto v3 = w_3_value * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[2];
    v3 += sum;
    auto v4 = w_4_value * crypto::Poseidon2Bn254ScalarFieldParams::internal_matrix_diagonal[3];
    v4 += sum;

    return q_poseidon2_internal_value * (alpha_a * (v1 - w_1_shifted_value) + alpha_b * (v2 - w_2_shifted_value) +
                                         alpha_c * (v3 - w_3_shifted_value) + alpha_d * (v4 - w_4_shifted_value));
}

template <typename FF> bool GoblinUltraCircuitBuilder_<FF>::check_circuit()
{
    bool result = true;
    if (!UltraCircuitBuilder_<arithmetization::UltraHonk<FF>>::check_circuit()) {
        return false;
    }

    const FF poseidon2_external_base = FF::random_element();
    const FF poseidon2_internal_base = FF::random_element();
    const FF alpha = FF::random_element();

    // For each gate
    for (size_t i = 0; i < this->num_gates; i++) {
        FF q_poseidon2_external_value;
        FF q_poseidon2_internal_value;
        FF q_1_value;
        FF q_2_value;
        FF q_3_value;
        FF q_4_value;
        FF w_1_value;
        FF w_2_value;
        FF w_3_value;
        FF w_4_value;
        // Get the values of selectors and wires and update tag products along the way
        q_poseidon2_external_value = this->q_poseidon2_external()[i];
        q_poseidon2_internal_value = this->q_poseidon2_internal()[i];
        q_1_value = this->q_1()[i];
        q_2_value = this->q_2()[i];
        q_3_value = this->q_3()[i];
        q_4_value = this->q_4()[i];
        w_1_value = this->get_variable(this->w_l()[i]);
        w_2_value = this->get_variable(this->w_r()[i]);
        w_3_value = this->get_variable(this->w_o()[i]);
        w_4_value = this->get_variable(this->w_4()[i]);
        FF w_1_shifted_value;
        FF w_2_shifted_value;
        FF w_3_shifted_value;
        FF w_4_shifted_value;
        if (i < (this->num_gates - 1)) {
            w_1_shifted_value = this->get_variable(this->w_l()[i + 1]);
            w_2_shifted_value = this->get_variable(this->w_r()[i + 1]);
            w_3_shifted_value = this->get_variable(this->w_o()[i + 1]);
            w_4_shifted_value = this->get_variable(this->w_4()[i + 1]);
        } else {
            w_1_shifted_value = FF::zero();
            w_2_shifted_value = FF::zero();
            w_3_shifted_value = FF::zero();
            w_4_shifted_value = FF::zero();
        }
        if (!compute_poseidon2_external_identity(q_poseidon2_external_value,
                                                 q_1_value,
                                                 q_2_value,
                                                 q_3_value,
                                                 q_4_value,
                                                 w_1_value,
                                                 w_2_value,
                                                 w_3_value,
                                                 w_4_value,
                                                 w_1_shifted_value,
                                                 w_2_shifted_value,
                                                 w_3_shifted_value,
                                                 w_4_shifted_value,
                                                 poseidon2_external_base,
                                                 alpha)
                 .is_zero()) {
#ifndef FUZZING
            info("Poseidon2External identity fails at gate ", i);
#endif
            result = false;
            break;
        }
        if (!compute_poseidon2_internal_identity(q_poseidon2_internal_value,
                                                 q_1_value,
                                                 w_1_value,
                                                 w_2_value,
                                                 w_3_value,
                                                 w_4_value,
                                                 w_1_shifted_value,
                                                 w_2_shifted_value,
                                                 w_3_shifted_value,
                                                 w_4_shifted_value,
                                                 poseidon2_internal_base,
                                                 alpha)
                 .is_zero()) {
#ifndef FUZZING
            info("Poseidon2Internal identity fails at gate ", i);
#endif
            result = false;
            break;
        }
    }
    return result;
}

template class GoblinUltraCircuitBuilder_<bb::fr>;
} // namespace bb