#include "acir_format.hpp"
#include "barretenberg/common/log.hpp"

namespace acir_format {

void read_witness(Builder& builder, WitnessVector const& witness)
{
    builder.variables[0] = 0;
    for (size_t i = 0; i < witness.size(); ++i) {
        builder.variables[i + 1] = witness[i];
    }
}

void create_circuit(Builder& builder, acir_format const& constraint_system)
{
    if (constraint_system.public_inputs.size() > constraint_system.varnum) {
        info("create_circuit: too many public inputs!");
    }

    for (size_t i = 1; i < constraint_system.varnum; ++i) {
        // If the index is in the public inputs vector, then we add it as a public input

        if (std::find(constraint_system.public_inputs.begin(), constraint_system.public_inputs.end(), i) !=
            constraint_system.public_inputs.end()) {
            builder.add_public_variable(0);
        } else {
            builder.add_variable(0);
        }
    }

    // Add arithmetic gates
    for (const auto& constraint : constraint_system.constraints) {
        builder.create_poly_gate(constraint);
    }

    // Add and constraint
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

    // Add ECDSA K1 constraints
    for (const auto& constraint : constraint_system.ecdsa_k1_constraints) {
        create_ecdsa_k1_verify_constraints(builder, constraint, false);
    }

    // Add ECDSA R1 constraints
    for (const auto& constraint : constraint_system.ecdsa_r1_constraints) {
        create_ecdsa_r1_verify_constraints(builder, constraint, false);
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

    // Add fixed base scalar mul constraints
    for (const auto& constraint : constraint_system.fixed_base_scalar_mul_constraints) {
        create_fixed_base_constraint(builder, constraint);
    }

    // Add hash to field constraints
    for (const auto& constraint : constraint_system.hash_to_field_constraints) {
        create_hash_to_field_constraints(builder, constraint);
    }

    // Add block constraints
    for (const auto& constraint : constraint_system.block_constraints) {
        create_block_constraints(builder, constraint, false);
    }

    // Add recursion constraints
    for (size_t i = 0; i < constraint_system.recursion_constraints.size(); ++i) {
        auto& constraint = constraint_system.recursion_constraints[i];
        create_recursion_constraints(builder, constraint);

        // make sure the verification key records the public input indices of the final recursion output
        // (N.B. up to the ACIR description to make sure that the final output aggregation object wires are public
        // inputs!)
        if (i == constraint_system.recursion_constraints.size() - 1) {
            std::vector<uint32_t> proof_output_witness_indices(constraint.output_aggregation_object.begin(),
                                                               constraint.output_aggregation_object.end());
            builder.set_recursive_proof(proof_output_witness_indices);
        }
    }
}

Builder create_circuit(const acir_format& constraint_system, size_t size_hint)
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

void create_circuit_with_witness(Builder& builder, acir_format const& constraint_system, WitnessVector const& witness)
{
    if (constraint_system.public_inputs.size() > constraint_system.varnum) {
        info("create_circuit_with_witness: too many public inputs!");
    }

    for (size_t i = 1; i < constraint_system.varnum; ++i) {
        // If the index is in the public inputs vector, then we add it as a public input

        if (std::find(constraint_system.public_inputs.begin(), constraint_system.public_inputs.end(), i) !=
            constraint_system.public_inputs.end()) {

            builder.add_public_variable(0);

        } else {
            builder.add_variable(0);
        }
    }

    read_witness(builder, witness);

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
        create_ecdsa_k1_verify_constraints(builder, constraint);
    }

    // Add ECDSA r1 constraints
    for (const auto& constraint : constraint_system.ecdsa_r1_constraints) {
        create_ecdsa_r1_verify_constraints(builder, constraint);
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

    // Add fixed base scalar mul constraints
    for (const auto& constraint : constraint_system.fixed_base_scalar_mul_constraints) {
        create_fixed_base_constraint(builder, constraint);
    }

    // Add hash to field constraints
    for (const auto& constraint : constraint_system.hash_to_field_constraints) {
        create_hash_to_field_constraints(builder, constraint);
    }

    // Add block constraints
    for (const auto& constraint : constraint_system.block_constraints) {
        create_block_constraints(builder, constraint);
    }

    // Add recursion constraints
    for (size_t i = 0; i < constraint_system.recursion_constraints.size(); ++i) {
        auto& constraint = constraint_system.recursion_constraints[i];
        create_recursion_constraints(builder, constraint, true);

        // make sure the verification key records the public input indices of the final recursion output
        // (N.B. up to the ACIR description to make sure that the final output aggregation object wires are public
        // inputs!)
        if (i == constraint_system.recursion_constraints.size() - 1) {
            std::vector<uint32_t> proof_output_witness_indices(constraint.output_aggregation_object.begin(),
                                                               constraint.output_aggregation_object.end());
            builder.set_recursive_proof(proof_output_witness_indices);
        }
    }
}

} // namespace acir_format
