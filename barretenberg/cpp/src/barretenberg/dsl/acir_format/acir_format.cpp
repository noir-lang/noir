#include "acir_format.hpp"
#include "barretenberg/common/log.hpp"
#include "barretenberg/common/throw_or_abort.hpp"
#include "barretenberg/stdlib/plonk_recursion/aggregation_state/aggregation_state.hpp"
#include "barretenberg/stdlib/primitives/field/field_conversion.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_circuit_builder.hpp"
#include "barretenberg/stdlib_circuit_builders/ultra_circuit_builder.hpp"
#include <cstddef>

namespace acir_format {

using namespace bb;

template class DSLBigInts<UltraCircuitBuilder>;
template class DSLBigInts<MegaCircuitBuilder>;

template <typename Builder>
void build_constraints(Builder& builder,
                       AcirFormat& constraint_system,
                       bool has_valid_witness_assignments,
                       bool honk_recursion,
                       bool collect_gates_per_opcode)
{
    if (collect_gates_per_opcode) {
        constraint_system.gates_per_opcode.resize(constraint_system.num_acir_opcodes, 0);
    }

    GateCounter gate_counter{ &builder, collect_gates_per_opcode };

    // Add arithmetic gates
    for (size_t i = 0; i < constraint_system.poly_triple_constraints.size(); ++i) {
        const auto& constraint = constraint_system.poly_triple_constraints.at(i);
        builder.create_poly_gate(constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.poly_triple_constraints.at(i));
    }

    for (size_t i = 0; i < constraint_system.quad_constraints.size(); ++i) {
        const auto& constraint = constraint_system.quad_constraints.at(i);
        builder.create_big_mul_gate(constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.quad_constraints.at(i));
    }

    // Add logic constraint
    for (size_t i = 0; i < constraint_system.logic_constraints.size(); ++i) {
        const auto& constraint = constraint_system.logic_constraints.at(i);
        create_logic_gate(
            builder, constraint.a, constraint.b, constraint.result, constraint.num_bits, constraint.is_xor_gate);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.logic_constraints.at(i));
    }

    // Add range constraint
    for (size_t i = 0; i < constraint_system.range_constraints.size(); ++i) {
        const auto& constraint = constraint_system.range_constraints.at(i);
        builder.create_range_constraint(constraint.witness, constraint.num_bits, "");
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.range_constraints.at(i));
    }

    // Add aes128 constraints
    for (size_t i = 0; i < constraint_system.aes128_constraints.size(); ++i) {
        const auto& constraint = constraint_system.aes128_constraints.at(i);
        create_aes128_constraints(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.aes128_constraints.at(i));
    }

    // Add sha256 constraints
    for (size_t i = 0; i < constraint_system.sha256_constraints.size(); ++i) {
        const auto& constraint = constraint_system.sha256_constraints.at(i);
        create_sha256_constraints(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.sha256_constraints.at(i));
    }

    for (size_t i = 0; i < constraint_system.sha256_compression.size(); ++i) {
        const auto& constraint = constraint_system.sha256_compression[i];
        create_sha256_compression_constraints(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.sha256_compression[i]);
    }

    // Add schnorr constraints
    for (size_t i = 0; i < constraint_system.schnorr_constraints.size(); ++i) {
        const auto& constraint = constraint_system.schnorr_constraints.at(i);
        create_schnorr_verify_constraints(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.schnorr_constraints.at(i));
    }

    // Add ECDSA k1 constraints
    for (size_t i = 0; i < constraint_system.ecdsa_k1_constraints.size(); ++i) {
        const auto& constraint = constraint_system.ecdsa_k1_constraints.at(i);
        create_ecdsa_k1_verify_constraints(builder, constraint, has_valid_witness_assignments);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.ecdsa_k1_constraints.at(i));
    }

    // Add ECDSA r1 constraints
    for (size_t i = 0; i < constraint_system.ecdsa_r1_constraints.size(); ++i) {
        const auto& constraint = constraint_system.ecdsa_r1_constraints.at(i);
        create_ecdsa_r1_verify_constraints(builder, constraint, has_valid_witness_assignments);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.ecdsa_r1_constraints.at(i));
    }

    // Add blake2s constraints
    for (size_t i = 0; i < constraint_system.blake2s_constraints.size(); ++i) {
        const auto& constraint = constraint_system.blake2s_constraints.at(i);
        create_blake2s_constraints(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.blake2s_constraints.at(i));
    }

    // Add blake3 constraints
    for (size_t i = 0; i < constraint_system.blake3_constraints.size(); ++i) {
        const auto& constraint = constraint_system.blake3_constraints.at(i);
        create_blake3_constraints(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.blake3_constraints.at(i));
    }

    // Add keccak constraints
    for (size_t i = 0; i < constraint_system.keccak_constraints.size(); ++i) {
        const auto& constraint = constraint_system.keccak_constraints.at(i);
        create_keccak_constraints(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.keccak_constraints.at(i));
    }

    for (size_t i = 0; i < constraint_system.keccak_permutations.size(); ++i) {
        const auto& constraint = constraint_system.keccak_permutations[i];
        create_keccak_permutations(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.keccak_permutations[i]);
    }

    // Add pedersen constraints
    for (size_t i = 0; i < constraint_system.pedersen_constraints.size(); ++i) {
        const auto& constraint = constraint_system.pedersen_constraints.at(i);
        create_pedersen_constraint(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.pedersen_constraints.at(i));
    }

    for (size_t i = 0; i < constraint_system.pedersen_hash_constraints.size(); ++i) {
        const auto& constraint = constraint_system.pedersen_hash_constraints.at(i);
        create_pedersen_hash_constraint(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.pedersen_hash_constraints.at(i));
    }

    for (size_t i = 0; i < constraint_system.poseidon2_constraints.size(); ++i) {
        const auto& constraint = constraint_system.poseidon2_constraints.at(i);
        create_poseidon2_permutations(builder, constraint);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.poseidon2_constraints.at(i));
    }

    // Add multi scalar mul constraints
    for (size_t i = 0; i < constraint_system.multi_scalar_mul_constraints.size(); ++i) {
        const auto& constraint = constraint_system.multi_scalar_mul_constraints.at(i);
        create_multi_scalar_mul_constraint(builder, constraint, has_valid_witness_assignments);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.multi_scalar_mul_constraints.at(i));
    }

    // Add ec add constraints
    for (size_t i = 0; i < constraint_system.ec_add_constraints.size(); ++i) {
        const auto& constraint = constraint_system.ec_add_constraints.at(i);
        create_ec_add_constraint(builder, constraint, has_valid_witness_assignments);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.ec_add_constraints.at(i));
    }

    // Add block constraints
    for (size_t i = 0; i < constraint_system.block_constraints.size(); ++i) {
        const auto& constraint = constraint_system.block_constraints.at(i);
        create_block_constraints(builder, constraint, has_valid_witness_assignments);
        if (collect_gates_per_opcode) {
            size_t avg_gates_per_opcode =
                gate_counter.compute_diff() / constraint_system.original_opcode_indices.block_constraints.at(i).size();
            for (size_t opcode_index : constraint_system.original_opcode_indices.block_constraints.at(i)) {
                constraint_system.gates_per_opcode[opcode_index] = avg_gates_per_opcode;
            }
        }
    }

    // Add big_int constraints
    DSLBigInts<Builder> dsl_bigints;
    dsl_bigints.set_builder(&builder);
    for (size_t i = 0; i < constraint_system.bigint_from_le_bytes_constraints.size(); ++i) {
        const auto& constraint = constraint_system.bigint_from_le_bytes_constraints.at(i);
        create_bigint_from_le_bytes_constraint(builder, constraint, dsl_bigints);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.bigint_from_le_bytes_constraints.at(i));
    }

    for (size_t i = 0; i < constraint_system.bigint_operations.size(); ++i) {
        const auto& constraint = constraint_system.bigint_operations[i];
        create_bigint_operations_constraint<Builder>(constraint, dsl_bigints, has_valid_witness_assignments);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.bigint_operations[i]);
    }

    for (size_t i = 0; i < constraint_system.bigint_to_le_bytes_constraints.size(); ++i) {
        const auto& constraint = constraint_system.bigint_to_le_bytes_constraints.at(i);
        create_bigint_to_le_bytes_constraint(builder, constraint, dsl_bigints);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.bigint_to_le_bytes_constraints.at(i));
    }

    // RecursionConstraints
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/817): disable these for MegaHonk for now since we're
    // not yet dealing with proper recursion
    if constexpr (IsMegaBuilder<Builder>) {
        if (!constraint_system.recursion_constraints.empty()) {
            info("WARNING: this circuit contains unhandled recursion_constraints!");
        }
        if (!constraint_system.honk_recursion_constraints.empty()) {
            info("WARNING: this circuit contains unhandled honk_recursion_constraints!");
        }
    } else {
        process_plonk_recursion_constraints(builder, constraint_system, has_valid_witness_assignments, gate_counter);
        process_honk_recursion_constraints(builder, constraint_system, has_valid_witness_assignments, gate_counter);

        // If the circuit does not itself contain honk recursion constraints but is going to be proven with honk then
        // recursively verified, add a default aggregation object
        if (constraint_system.honk_recursion_constraints.empty() && honk_recursion &&
            builder.is_recursive_circuit) { // Set a default aggregation object if we don't have one.
            AggregationObjectIndices current_aggregation_object =
                stdlib::recursion::init_default_agg_obj_indices<Builder>(builder);
            // Make sure the verification key records the public input indices of the
            // final recursion output.
            builder.add_recursive_proof(current_aggregation_object);
        }
    }
}

void process_plonk_recursion_constraints(Builder& builder,
                                         AcirFormat& constraint_system,
                                         bool has_valid_witness_assignments,
                                         GateCounter<Builder>& gate_counter)
{

    // These are set and modified whenever we encounter a recursion opcode
    //
    // These should not be set by the caller
    // TODO(maxim): Check if this is always the case. ie I won't receive a proof that will set the first
    // TODO(maxim): input_aggregation_object to be non-zero.
    // TODO(maxim): if not, we can add input_aggregation_object to the proof too for all recursive proofs
    // TODO(maxim): This might be the case for proof trees where the proofs are created on different machines
    AggregationObjectIndices current_input_aggregation_object = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };
    AggregationObjectIndices current_output_aggregation_object = { 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 };

    // Get the size of proof with no public inputs prepended to it
    // This is used while processing recursion constraints to determine whether
    // the proof we are verifying contains a recursive proof itself
    auto proof_size_no_pub_inputs = recursion_proof_size_without_public_inputs();

    // Add recursion constraints
    for (size_t constraint_idx = 0; constraint_idx < constraint_system.recursion_constraints.size(); ++constraint_idx) {
        auto constraint = constraint_system.recursion_constraints[constraint_idx];

        // A proof passed into the constraint should be stripped of its public inputs, except in the case where a
        // proof contains an aggregation object itself. We refer to this as the `nested_aggregation_object`. The
        // verifier circuit requires that the indices to a nested proof aggregation state are a circuit constant.
        // The user tells us they how they want these constants set by keeping the nested aggregation object
        // attached to the proof as public inputs. As this is the only object that can prepended to the proof if the
        // proof is above the expected size (with public inputs stripped)
        AggregationObjectPubInputIndices nested_aggregation_object = {};
        // If the proof has public inputs attached to it, we should handle setting the nested aggregation object
        if (constraint.proof.size() > proof_size_no_pub_inputs) {
            // The public inputs attached to a proof should match the aggregation object in size
            if (constraint.proof.size() - proof_size_no_pub_inputs != bb::AGGREGATION_OBJECT_SIZE) {
                auto error_string = format(
                    "Public inputs are always stripped from proofs unless we have a recursive proof.\n"
                    "Thus, public inputs attached to a proof must match the recursive aggregation object in size "
                    "which is ",
                    bb::AGGREGATION_OBJECT_SIZE);
                throw_or_abort(error_string);
            }
            for (size_t i = 0; i < bb::AGGREGATION_OBJECT_SIZE; ++i) {
                // Set the nested aggregation object indices to the current size of the public inputs
                // This way we know that the nested aggregation object indices will always be the last
                // indices of the public inputs
                nested_aggregation_object[i] = static_cast<uint32_t>(constraint.public_inputs.size());
                // Attach the nested aggregation object to the end of the public inputs to fill in
                // the slot where the nested aggregation object index will point into
                constraint.public_inputs.emplace_back(constraint.proof[i]);
            }
            // Remove the aggregation object so that they can be handled as normal public inputs
            // in the way that the recursion constraint expects
            constraint.proof.erase(constraint.proof.begin(),
                                   constraint.proof.begin() + static_cast<std::ptrdiff_t>(bb::AGGREGATION_OBJECT_SIZE));
        }

        current_output_aggregation_object = create_recursion_constraints(builder,
                                                                         constraint,
                                                                         current_input_aggregation_object,
                                                                         nested_aggregation_object,
                                                                         has_valid_witness_assignments);
        current_input_aggregation_object = current_output_aggregation_object;
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.recursion_constraints[constraint_idx]);
    }

    // Now that the circuit has been completely built, we add the output aggregation as public
    // inputs.
    if (!constraint_system.recursion_constraints.empty()) {

        // First add the output aggregation object as public inputs
        // Set the indices as public inputs because they are no longer being
        // created in ACIR
        for (const auto& idx : current_output_aggregation_object) {
            builder.set_public_input(idx);
        }

        // Make sure the verification key records the public input indices of the
        // final recursion output.
        builder.set_recursive_proof(current_output_aggregation_object);
    }
};

void process_honk_recursion_constraints(Builder& builder,
                                        AcirFormat& constraint_system,
                                        bool has_valid_witness_assignments,
                                        GateCounter<Builder>& gate_counter)
{
    AggregationObjectIndices current_aggregation_object =
        stdlib::recursion::init_default_agg_obj_indices<Builder>(builder);

    // Add recursion constraints
    for (size_t i = 0; i < constraint_system.honk_recursion_constraints.size(); ++i) {
        auto& constraint = constraint_system.honk_recursion_constraints.at(i);
        // A proof passed into the constraint should be stripped of its inner public inputs, but not the nested
        // aggregation object itself. The verifier circuit requires that the indices to a nested proof aggregation
        // state are a circuit constant. The user tells us they how they want these constants set by keeping the
        // nested aggregation object attached to the proof as public inputs.
        for (size_t i = 0; i < bb::AGGREGATION_OBJECT_SIZE; ++i) {
            // Adding the nested aggregation object to the constraint's public inputs
            constraint.public_inputs.emplace_back(constraint.proof[HONK_RECURSION_PUBLIC_INPUT_OFFSET + i]);
        }
        // Remove the aggregation object so that they can be handled as normal public inputs
        // in they way that the recursion constraint expects
        constraint.proof.erase(
            constraint.proof.begin() + HONK_RECURSION_PUBLIC_INPUT_OFFSET,
            constraint.proof.begin() +
                static_cast<std::ptrdiff_t>(HONK_RECURSION_PUBLIC_INPUT_OFFSET + bb::AGGREGATION_OBJECT_SIZE));
        current_aggregation_object = create_honk_recursion_constraints(
            builder, constraint, current_aggregation_object, has_valid_witness_assignments);
        gate_counter.track_diff(constraint_system.gates_per_opcode,
                                constraint_system.original_opcode_indices.honk_recursion_constraints.at(i));
    }

    // Now that the circuit has been completely built, we add the output aggregation as public
    // inputs.
    if (!constraint_system.honk_recursion_constraints.empty()) {

        // First add the output aggregation object as public inputs
        // Set the indices as public inputs because they are no longer being
        // created in ACIR
        for (const auto& idx : current_aggregation_object) {
            builder.set_public_input(idx);
        }

        // Make sure the verification key records the public input indices of the
        // final recursion output.
        builder.set_recursive_proof(current_aggregation_object);
    }
};

/**
 * @brief Specialization for creating Ultra circuit from acir constraints and optionally a witness
 *
 * @tparam Builder
 * @param constraint_system
 * @param size_hint
 * @param witness
 * @return Builder
 */
template <>
UltraCircuitBuilder create_circuit(AcirFormat& constraint_system,
                                   size_t size_hint,
                                   WitnessVector const& witness,
                                   bool honk_recursion,
                                   [[maybe_unused]] std::shared_ptr<ECCOpQueue>,
                                   bool collect_gates_per_opcode)
{
    Builder builder{
        size_hint, witness, constraint_system.public_inputs, constraint_system.varnum, constraint_system.recursive
    };

    bool has_valid_witness_assignments = !witness.empty();
    build_constraints(
        builder, constraint_system, has_valid_witness_assignments, honk_recursion, collect_gates_per_opcode);

    return builder;
};

/**
 * @brief Specialization for creating Mega circuit from acir constraints and optionally a witness
 *
 * @tparam Builder
 * @param constraint_system
 * @param size_hint
 * @param witness
 * @return Builder
 */
template <>
MegaCircuitBuilder create_circuit(AcirFormat& constraint_system,
                                  [[maybe_unused]] size_t size_hint,
                                  WitnessVector const& witness,
                                  bool honk_recursion,
                                  std::shared_ptr<ECCOpQueue> op_queue,
                                  bool collect_gates_per_opcode)
{
    // Construct a builder using the witness and public input data from acir and with the goblin-owned op_queue
    auto builder = MegaCircuitBuilder{ op_queue, witness, constraint_system.public_inputs, constraint_system.varnum };

    // Populate constraints in the builder via the data in constraint_system
    bool has_valid_witness_assignments = !witness.empty();
    acir_format::build_constraints(
        builder, constraint_system, has_valid_witness_assignments, honk_recursion, collect_gates_per_opcode);

    return builder;
};

template void build_constraints<MegaCircuitBuilder>(MegaCircuitBuilder&, AcirFormat&, bool, bool, bool);

} // namespace acir_format
