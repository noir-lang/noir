#include "acir_format.hpp"
#include "barretenberg/common/log.hpp"
#include "barretenberg/dsl/acir_format/pedersen.hpp"
#include "barretenberg/dsl/acir_format/recursion_constraint.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"
#include <cstddef>

namespace acir_format {

/**
 * @brief Populate variables and public_inputs in builder given witness and constraint_system
 * @details This method replaces consecutive calls to add_public_vars then read_witness.
 *
 * @tparam Builder
 * @param builder
 * @param witness
 * @param constraint_system
 */
template <typename Builder>
void populate_variables_and_public_inputs(Builder& builder,
                                          WitnessVector const& witness,
                                          acir_format const& constraint_system)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): Decrement the indices in
    // constraint_system.public_inputs by one to account for the +1 added by default to account for a const zero
    // variable in noir. This entire block can be removed once the +1 is removed from noir.
    const uint32_t pre_applied_noir_offset = 1;
    std::vector<uint32_t> corrected_public_inputs;
    for (const auto& index : constraint_system.public_inputs) {
        corrected_public_inputs.emplace_back(index - pre_applied_noir_offset);
    }

    for (size_t idx = 0; idx < constraint_system.varnum; ++idx) {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/815) why is this needed?
        fr value = idx < witness.size() ? witness[idx] : 0;
        if (std::find(corrected_public_inputs.begin(), corrected_public_inputs.end(), idx) !=
            corrected_public_inputs.end()) {
            builder.add_public_variable(value);
        } else {
            builder.add_variable(value);
        }
    }
}

template <typename Builder> void read_witness(Builder& builder, WitnessVector const& witness)
{
    builder.variables[0] =
        0; // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): This the constant 0 hacked in. Bad.
    for (size_t i = 0; i < witness.size(); ++i) {
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): The i+1 accounts for the fact that 0 is added
        // as a constant in variables in the UCB constructor. "witness" only contains the values that came directly from
        // acir.
        builder.variables[i + 1] = witness[i];
    }
}

// TODO(https://github.com/AztecProtocol/barretenberg/issues/815): This function does two things: 1) emplaces back
// varnum-many 0s into builder.variables (.. why), and (2) populates builder.public_inputs with the correct indices into
// the variables vector (which at this stage will be populated with zeros). The actual entries of the variables vector
// are populated in "read_witness"
template <typename Builder> void add_public_vars(Builder& builder, acir_format const& constraint_system)
{
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/816): i = 1 acounting for const 0 in first position?
    for (size_t i = 1; i < constraint_system.varnum; ++i) {
        // If the index is in the public inputs vector, then we add it as a public input

        if (std::find(constraint_system.public_inputs.begin(), constraint_system.public_inputs.end(), i) !=
            constraint_system.public_inputs.end()) {

            builder.add_public_variable(0);

        } else {
            builder.add_variable(0);
        }
    }
}

template <typename Builder>
void build_constraints(Builder& builder, acir_format const& constraint_system, bool has_valid_witness_assignments)
{
    // Add arithmetic gates
    for (const auto& constraint : constraint_system.constraints) {
        builder.create_poly_gate(constraint);
    }

    // Add logic constraint
    for (const auto& constraint : constraint_system.logic_constraints) {
        create_logic_gate(
            builder, constraint.a, constraint.b, constraint.result, constraint.num_bits, constraint.is_xor_gate);
    }

    // Add range constraint
    for (const auto& constraint : constraint_system.range_constraints) {
        builder.create_range_constraint(constraint.witness, constraint.num_bits, "");
    }

    // Add sha256 constraints
    for (const auto& constraint : constraint_system.sha256_constraints) {
        create_sha256_constraints(builder, constraint);
    }

    // Add schnorr constraints
    for (const auto& constraint : constraint_system.schnorr_constraints) {
        create_schnorr_verify_constraints(builder, constraint);
    }

    // Add ECDSA k1 constraints
    for (const auto& constraint : constraint_system.ecdsa_k1_constraints) {
        create_ecdsa_k1_verify_constraints(builder, constraint, has_valid_witness_assignments);
    }

    // Add ECDSA r1 constraints
    for (const auto& constraint : constraint_system.ecdsa_r1_constraints) {
        create_ecdsa_r1_verify_constraints(builder, constraint, has_valid_witness_assignments);
    }

    // Add blake2s constraints
    for (const auto& constraint : constraint_system.blake2s_constraints) {
        create_blake2s_constraints(builder, constraint);
    }

    // Add keccak constraints
    for (const auto& constraint : constraint_system.keccak_constraints) {
        create_keccak_constraints(builder, constraint);
    }
    for (const auto& constraint : constraint_system.keccak_var_constraints) {
        create_keccak_var_constraints(builder, constraint);
    }

    // Add pedersen constraints
    for (const auto& constraint : constraint_system.pedersen_constraints) {
        create_pedersen_constraint(builder, constraint);
    }

    for (const auto& constraint : constraint_system.pedersen_hash_constraints) {
        create_pedersen_hash_constraint(builder, constraint);
    }

    // Add fixed base scalar mul constraints
    for (const auto& constraint : constraint_system.fixed_base_scalar_mul_constraints) {
        create_fixed_base_constraint(builder, constraint);
    }

    // Add block constraints
    for (const auto& constraint : constraint_system.block_constraints) {
        create_block_constraints(builder, constraint, has_valid_witness_assignments);
    }

    // TODO(https://github.com/AztecProtocol/barretenberg/issues/817): disable these for UGH for now since we're not yet
    // dealing with proper recursion
    if constexpr (IsGoblinBuilder<Builder>) {
        if (constraint_system.recursion_constraints.size() > 0) {
            info("WARNING: this circuit contains recursion_constraints!");
        }
    } else {
        // These are set and modified whenever we encounter a recursion opcode
        //
        // These should not be set by the caller
        // TODO(maxim): Check if this is always the case. ie I won't receive a proof that will set the first
        // TODO(maxim): input_aggregation_object to be non-zero.
        // TODO(maxim): if not, we can add input_aggregation_object to the proof too for all recursive proofs
        // TODO(maxim): This might be the case for proof trees where the proofs are created on different machines
        std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> current_input_aggregation_object = {
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        };
        std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> current_output_aggregation_object = {
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        };

        // Get the size of proof with no public inputs prepended to it
        // This is used while processing recursion constraints to determine whether
        // the proof we are verifying contains a recursive proof itself
        auto proof_size_no_pub_inputs = recursion_proof_size_without_public_inputs();

        // Add recursion constraints
        for (auto constraint : constraint_system.recursion_constraints) {
            // A proof passed into the constraint should be stripped of its public inputs, except in the case where a
            // proof contains an aggregation object itself. We refer to this as the `nested_aggregation_object`. The
            // verifier circuit requires that the indices to a nested proof aggregation state are a circuit constant.
            // The user tells us they how they want these constants set by keeping the nested aggregation object
            // attached to the proof as public inputs. As this is the only object that can prepended to the proof if the
            // proof is above the expected size (with public inputs stripped)
            std::array<uint32_t, RecursionConstraint::AGGREGATION_OBJECT_SIZE> nested_aggregation_object = {};
            // If the proof has public inputs attached to it, we should handle setting the nested aggregation object
            if (constraint.proof.size() > proof_size_no_pub_inputs) {
                // The public inputs attached to a proof should match the aggregation object in size
                ASSERT(constraint.proof.size() - proof_size_no_pub_inputs ==
                       RecursionConstraint::AGGREGATION_OBJECT_SIZE);
                for (size_t i = 0; i < RecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
                    // Set the nested aggregation object indices to the current size of the public inputs
                    // This way we know that the nested aggregation object indices will always be the last
                    // indices of the public inputs
                    nested_aggregation_object[i] = static_cast<uint32_t>(constraint.public_inputs.size());
                    // Attach the nested aggregation object to the end of the public inputs to fill in
                    // the slot where the nested aggregation object index will point into
                    constraint.public_inputs.emplace_back(constraint.proof[i]);
                }
                // Remove the aggregation object so that they can be handled as normal public inputs
                // in they way taht the recursion constraint expects
                constraint.proof.erase(constraint.proof.begin(),
                                       constraint.proof.begin() +
                                           static_cast<std::ptrdiff_t>(RecursionConstraint::AGGREGATION_OBJECT_SIZE));
            }

            current_output_aggregation_object = create_recursion_constraints(builder,
                                                                             constraint,
                                                                             current_input_aggregation_object,
                                                                             nested_aggregation_object,
                                                                             has_valid_witness_assignments);
            current_input_aggregation_object = current_output_aggregation_object;
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
            std::vector<uint32_t> proof_output_witness_indices(current_output_aggregation_object.begin(),
                                                               current_output_aggregation_object.end());
            builder.set_recursive_proof(proof_output_witness_indices);
        }
    }
}

template <typename Builder> void create_circuit(Builder& builder, acir_format const& constraint_system)
{
    if (constraint_system.public_inputs.size() > constraint_system.varnum) {
        info("create_circuit: too many public inputs!");
    }

    add_public_vars(builder, constraint_system);
    build_constraints(builder, constraint_system, false);
}

template <typename Builder> Builder create_circuit(const acir_format& constraint_system, size_t size_hint)
{
    Builder builder(size_hint);
    create_circuit(builder, constraint_system);
    return builder;
}

Builder create_circuit_with_witness(acir_format const& constraint_system,
                                    WitnessVector const& witness,
                                    size_t size_hint)
{
    Builder builder(size_hint);
    create_circuit_with_witness(builder, constraint_system, witness);
    return builder;
}

template <typename Builder>
void create_circuit_with_witness(Builder& builder, acir_format const& constraint_system, WitnessVector const& witness)
{
    if (constraint_system.public_inputs.size() > constraint_system.varnum) {
        info("create_circuit_with_witness: too many public inputs!");
    }

    // Populate builder.variables and buider.public_inputs
    populate_variables_and_public_inputs(builder, witness, constraint_system);

    build_constraints(builder, constraint_system, true);
}

template UltraCircuitBuilder create_circuit<UltraCircuitBuilder>(const acir_format& constraint_system,
                                                                 size_t size_hint);
template void create_circuit_with_witness<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                               acir_format const& constraint_system,
                                                               WitnessVector const& witness);
template void create_circuit_with_witness<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                     acir_format const& constraint_system,
                                                                     WitnessVector const& witness);
template void build_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder&, acir_format const&, bool);

} // namespace acir_format
