#include "honk_recursion_constraint.hpp"
#include "barretenberg/stdlib/primitives/bigfield/constants.hpp"
#include "recursion_constraint.hpp"

namespace acir_format {

using namespace bb;
using namespace bb::stdlib::recursion::honk;

std::array<bn254::Group, 2> agg_points_from_witness_indicies(
    Builder& builder, const std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE>& obj_witness_indices)
{
    std::array<bn254::BaseField, 4> aggregation_elements;
    for (size_t i = 0; i < 4; ++i) {
        aggregation_elements[i] =
            bn254::BaseField(field_ct::from_witness_index(&builder, obj_witness_indices[4 * i]),
                             field_ct::from_witness_index(&builder, obj_witness_indices[4 * i + 1]),
                             field_ct::from_witness_index(&builder, obj_witness_indices[4 * i + 2]),
                             field_ct::from_witness_index(&builder, obj_witness_indices[4 * i + 3]));
        aggregation_elements[i].assert_is_in_field();
    }

    return { bn254::Group(aggregation_elements[0], aggregation_elements[1]),
             bn254::Group(aggregation_elements[2], aggregation_elements[3]) };
}

/**
 * @brief Add constraints required to recursively verify an UltraHonk proof
 *
 * @param builder
 * @param input
 * @param input_aggregation_object. The aggregation object coming from previous Honk recursion constraints.
 * @param nested_aggregation_object. The aggregation object coming from the inner proof.
 * @param has_valid_witness_assignment. Do we have witnesses or are we just generating keys?
 *
 * @note We currently only support HonkRecursionConstraint where inner_proof_contains_recursive_proof = false.
 *       We would either need a separate ACIR opcode where inner_proof_contains_recursive_proof = true,
 *       or we need non-witness data to be provided as metadata in the ACIR opcode
 */
std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> create_honk_recursion_constraints(
    Builder& builder,
    const HonkRecursionConstraint& input,
    std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> input_aggregation_object,
    std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> nested_aggregation_object,
    bool has_valid_witness_assignments)
{
    using Flavor = UltraRecursiveFlavor_<Builder>;
    using RecursiveVerificationKey = Flavor::VerificationKey;
    using RecursiveVerifier = UltraRecursiveVerifier_<Flavor>;

    // Ignore the case of invalid witness assignments for now.
    static_cast<void>(has_valid_witness_assignments);

    // Construct aggregation points from the nested aggregation witness indices
    std::array<bn254::Group, 2> nested_aggregation_points =
        agg_points_from_witness_indicies(builder, nested_aggregation_object);

    // Construct an in-circuit representation of the verification key.
    // For now, the v-key is a circuit constant and is fixed for the circuit.
    // (We may need a separate recursion opcode for this to vary, or add more config witnesses to this opcode)
    const auto& aggregation_input = input_aggregation_object;
    aggregation_state_ct cur_aggregation_object;
    cur_aggregation_object.P0 = nested_aggregation_points[0];
    cur_aggregation_object.P1 = nested_aggregation_points[1];
    cur_aggregation_object.has_data = true; // the nested aggregation object always exists

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/995): generate this challenge properly.
    field_ct recursion_separator = bb::stdlib::witness_t<Builder>(&builder, 2);

    // If we have previously recursively verified proofs, `previous_aggregation_object_nonzero = true`
    // For now this is a complile-time constant i.e. whether this is true/false is fixed for the circuit!
    bool previous_aggregation_indices_all_zero = true;
    for (const auto& idx : aggregation_input) {
        previous_aggregation_indices_all_zero &= (idx == 0);
    }

    // Aggregate the aggregation object if it exists. It exists if we have previously verified proofs, i.e. if this is
    // not the first recursion constraint.
    if (!previous_aggregation_indices_all_zero) {
        std::array<bn254::Group, 2> inner_agg_points = agg_points_from_witness_indicies(builder, aggregation_input);
        // If we have a previous aggregation object, aggregate it into the current aggregation object.
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/995): Verify that using challenge and challenge
        // squared is safe.
        cur_aggregation_object.P0 += inner_agg_points[0] * recursion_separator;
        cur_aggregation_object.P1 += inner_agg_points[1] * recursion_separator;
        recursion_separator =
            recursion_separator *
            recursion_separator; // update the challenge to be challenge squared for the next aggregation
    }

    std::vector<field_ct> key_fields;
    key_fields.reserve(input.key.size());
    for (const auto& idx : input.key) {
        auto field = field_ct::from_witness_index(&builder, idx);
        key_fields.emplace_back(field);
    }

    std::vector<field_ct> proof_fields;
    // Insert the public inputs in the middle the proof fields after 'inner_public_input_offset' because this is how the
    // core barretenberg library processes proofs (with the public inputs starting at the third element and not
    // separate from the rest of the proof)
    proof_fields.reserve(input.proof.size() + input.public_inputs.size());
    size_t i = 0;
    for (const auto& idx : input.proof) {
        auto field = field_ct::from_witness_index(&builder, idx);
        proof_fields.emplace_back(field);
        i++;
        if (i == HonkRecursionConstraint::inner_public_input_offset) {
            for (const auto& idx : input.public_inputs) {
                auto field = field_ct::from_witness_index(&builder, idx);
                proof_fields.emplace_back(field);
            }
        }
    }

    // Recursively verify the proof
    auto vkey = std::make_shared<RecursiveVerificationKey>(builder, key_fields);
    RecursiveVerifier verifier(&builder, vkey);
    std::array<typename Flavor::GroupElement, 2> pairing_points = verifier.verify_proof(proof_fields);

    // Aggregate the current aggregation object with these pairing points from verify_proof
    cur_aggregation_object.P0 += pairing_points[0] * recursion_separator;
    cur_aggregation_object.P1 += pairing_points[1] * recursion_separator;

    std::vector<uint32_t> proof_witness_indices = {
        cur_aggregation_object.P0.x.binary_basis_limbs[0].element.normalize().witness_index,
        cur_aggregation_object.P0.x.binary_basis_limbs[1].element.normalize().witness_index,
        cur_aggregation_object.P0.x.binary_basis_limbs[2].element.normalize().witness_index,
        cur_aggregation_object.P0.x.binary_basis_limbs[3].element.normalize().witness_index,
        cur_aggregation_object.P0.y.binary_basis_limbs[0].element.normalize().witness_index,
        cur_aggregation_object.P0.y.binary_basis_limbs[1].element.normalize().witness_index,
        cur_aggregation_object.P0.y.binary_basis_limbs[2].element.normalize().witness_index,
        cur_aggregation_object.P0.y.binary_basis_limbs[3].element.normalize().witness_index,
        cur_aggregation_object.P1.x.binary_basis_limbs[0].element.normalize().witness_index,
        cur_aggregation_object.P1.x.binary_basis_limbs[1].element.normalize().witness_index,
        cur_aggregation_object.P1.x.binary_basis_limbs[2].element.normalize().witness_index,
        cur_aggregation_object.P1.x.binary_basis_limbs[3].element.normalize().witness_index,
        cur_aggregation_object.P1.y.binary_basis_limbs[0].element.normalize().witness_index,
        cur_aggregation_object.P1.y.binary_basis_limbs[1].element.normalize().witness_index,
        cur_aggregation_object.P1.y.binary_basis_limbs[2].element.normalize().witness_index,
        cur_aggregation_object.P1.y.binary_basis_limbs[3].element.normalize().witness_index,
    };
    auto result = cur_aggregation_object;
    result.proof_witness_indices = proof_witness_indices;

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/996): investigate whether assert_equal on public inputs
    // is important, like what the plonk recursion constraint does.

    // We want to return an array, so just copy the vector into the array
    ASSERT(result.proof_witness_indices.size() == HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE);
    std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> resulting_output_aggregation_object;
    std::copy(result.proof_witness_indices.begin(),
              result.proof_witness_indices.begin() + HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE,
              resulting_output_aggregation_object.begin());

    return resulting_output_aggregation_object;
}

} // namespace acir_format
