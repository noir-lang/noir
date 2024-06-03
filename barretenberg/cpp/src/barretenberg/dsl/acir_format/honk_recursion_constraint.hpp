#pragma once
#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include <vector>

namespace acir_format {
using Builder = bb::UltraCircuitBuilder;

using namespace bb;
using fq_ct = stdlib::bigfield<Builder, Bn254FqParams>;

/**
 * @brief HonkRecursionConstraint struct contains information required to recursively verify a proof!
 *
 * @details The recursive verifier algorithm produces an 'aggregation object' representing 2 G1 points, expressed as 16
 * witness values. The smart contract Verifier must be aware of this aggregation object in order to complete the full
 * recursive verification. If the circuit verifies more than 1 proof, the recursion algorithm will update a pre-existing
 * aggregation object (`input_aggregation_object`).
 *
 * @details We currently require that the inner circuit being verified only has a single public input. If more are
 * required, the outer circuit can hash them down to 1 input.
 *
 * @param verification_key_data The inner circuit vkey. Is converted into circuit witness values (internal to the
 * backend)
 * @param proof The honk proof. Is converted into circuit witness values (internal to the backend)
 * @param is_aggregation_object_nonzero A flag to tell us whether the circuit has already recursively verified proofs
 * (and therefore an aggregation object is present)
 * @param public_input The index of the single public input
 * @param input_aggregation_object Witness indices of pre-existing aggregation object (if it exists)
 * @param output_aggregation_object Witness indices of the aggregation object produced by recursive verification
 * @param nested_aggregation_object Public input indices of an aggregation object inside the proof.
 *
 * @note If input_aggregation_object witness indices are all zero, we interpret this to mean that the inner proof does
 * NOT contain a previously recursively verified proof
 * @note nested_aggregation_object is used for cases where the proof being verified contains an aggregation object in
 * its public inputs! If this is the case, we record the public input locations in `nested_aggregation_object`. If the
 * inner proof is of a circuit that does not have a nested aggregation object, these values are all zero.
 *
 * To outline the interaction between the input_aggergation_object and the nested_aggregation_object take the following
 * example: If we have a circuit that verifies 2 proofs A and B, the recursion constraint for B will have an
 * input_aggregation_object that points to the aggregation output produced by verifying A. If circuit B also verifies a
 * proof, in the above example the recursion constraint for verifying B will have a nested object that describes the
 * aggregation object in B’s public inputs as well as an input aggregation object that points to the object produced by
 * the previous recursion constraint in the circuit (the one that verifies A)
 *
 * TODO(https://github.com/AztecProtocol/barretenberg/issues/996): Update these comments for Honk.
 */
struct HonkRecursionConstraint {
    // In Honk, the proof starts with circuit_size, num_public_inputs, and pub_input_offset. We use this offset to keep
    // track of where the public inputs start.
    static constexpr size_t inner_public_input_offset = 3;
    // An aggregation state is represented by two G1 affine elements. Each G1 point has
    // two field element coordinates (x, y). Thus, four field elements
    static constexpr size_t NUM_AGGREGATION_ELEMENTS = 4;
    // Four limbs are used when simulating a non-native field using the bigfield class
    static constexpr size_t AGGREGATION_OBJECT_SIZE = NUM_AGGREGATION_ELEMENTS * fq_ct::NUM_LIMBS; // 16 field elements
    std::vector<uint32_t> key;
    std::vector<uint32_t> proof;
    std::vector<uint32_t> public_inputs;

    friend bool operator==(HonkRecursionConstraint const& lhs, HonkRecursionConstraint const& rhs) = default;
};

std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> create_honk_recursion_constraints(
    Builder& builder,
    const HonkRecursionConstraint& input,
    std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> input_aggregation_object,
    std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> nested_aggregation_object,
    bool has_valid_witness_assignments = false);

} // namespace acir_format
