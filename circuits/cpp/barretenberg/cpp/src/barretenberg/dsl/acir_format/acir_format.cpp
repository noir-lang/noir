#include "acir_format.hpp"
#include "barretenberg/common/log.hpp"

namespace acir_format {

void read_witness(Composer& composer, WitnessVector const& witness)
{
    composer.variables[0] = 0;
    for (size_t i = 0; i < witness.size(); ++i) {
        composer.variables[i + 1] = witness[i];
    }
}

void create_circuit(Composer& composer, acir_format const& constraint_system)
{
    if (constraint_system.public_inputs.size() > constraint_system.varnum) {
        info("create_circuit: too many public inputs!");
    }

    for (size_t i = 1; i < constraint_system.varnum; ++i) {
        // If the index is in the public inputs vector, then we add it as a public input

        if (std::find(constraint_system.public_inputs.begin(), constraint_system.public_inputs.end(), i) !=
            constraint_system.public_inputs.end()) {
            composer.add_public_variable(0);
        } else {
            composer.add_variable(0);
        }
    }

    // Add arithmetic gates
    for (const auto& constraint : constraint_system.constraints) {
        composer.create_poly_gate(constraint);
    }

    // Add and constraint
    for (const auto& constraint : constraint_system.logic_constraints) {
        create_logic_gate(
            composer, constraint.a, constraint.b, constraint.result, constraint.num_bits, constraint.is_xor_gate);
    }

    // Add range constraint
    for (const auto& constraint : constraint_system.range_constraints) {
        composer.create_range_constraint(constraint.witness, constraint.num_bits, "");
    }

    // Add sha256 constraints
    for (const auto& constraint : constraint_system.sha256_constraints) {
        create_sha256_constraints(composer, constraint);
    }

    // Add schnorr constraints
    for (const auto& constraint : constraint_system.schnorr_constraints) {
        create_schnorr_verify_constraints(composer, constraint);
    }

    // Add ECDSA constraints
    for (const auto& constraint : constraint_system.ecdsa_constraints) {
        create_ecdsa_verify_constraints(composer, constraint, false);
    }

    // Add blake2s constraints
    for (const auto& constraint : constraint_system.blake2s_constraints) {
        create_blake2s_constraints(composer, constraint);
    }

    // Add keccak constraints
    for (const auto& constraint : constraint_system.keccak_constraints) {
        create_keccak_constraints(composer, constraint);
    }
    for (const auto& constraint : constraint_system.keccak_var_constraints) {
        create_keccak_var_constraints(composer, constraint);
    }

    // Add pedersen constraints
    for (const auto& constraint : constraint_system.pedersen_constraints) {
        create_pedersen_constraint(composer, constraint);
    }

    // Add fixed base scalar mul constraints
    for (const auto& constraint : constraint_system.fixed_base_scalar_mul_constraints) {
        create_fixed_base_constraint(composer, constraint);
    }

    // Add hash to field constraints
    for (const auto& constraint : constraint_system.hash_to_field_constraints) {
        create_hash_to_field_constraints(composer, constraint);
    }

    // Add block constraints
    for (const auto& constraint : constraint_system.block_constraints) {
        create_block_constraints(composer, constraint, false);
    }

    // Add recursion constraints
    for (size_t i = 0; i < constraint_system.recursion_constraints.size(); ++i) {
        auto& constraint = constraint_system.recursion_constraints[i];
        create_recursion_constraints(composer, constraint);

        // make sure the verification key records the public input indices of the final recursion output
        // (N.B. up to the ACIR description to make sure that the final output aggregation object wires are public
        // inputs!)
        if (i == constraint_system.recursion_constraints.size() - 1) {
            std::vector<uint32_t> proof_output_witness_indices(constraint.output_aggregation_object.begin(),
                                                               constraint.output_aggregation_object.end());
            composer.set_recursive_proof(proof_output_witness_indices);
        }
    }
}

Composer create_circuit(const acir_format& constraint_system,
                        std::shared_ptr<barretenberg::srs::factories::CrsFactory> const& crs_factory,
                        size_t size_hint)
{
    Composer composer(crs_factory, size_hint);
    create_circuit(composer, constraint_system);
    return composer;
}

Composer create_circuit_with_witness(acir_format const& constraint_system,
                                     WitnessVector const& witness,
                                     std::shared_ptr<barretenberg::srs::factories::CrsFactory> const& crs_factory,
                                     size_t size_hint)
{
    Composer composer(crs_factory, size_hint);
    create_circuit_with_witness(composer, constraint_system, witness);
    return composer;
}

Composer create_circuit_with_witness(const acir_format& constraint_system, WitnessVector const& witness)
{
    auto composer = Composer();
    create_circuit_with_witness(composer, constraint_system, witness);
    return composer;
}

void create_circuit_with_witness(Composer& composer, acir_format const& constraint_system, WitnessVector const& witness)
{
    if (constraint_system.public_inputs.size() > constraint_system.varnum) {
        info("create_circuit_with_witness: too many public inputs!");
    }

    for (size_t i = 1; i < constraint_system.varnum; ++i) {
        // If the index is in the public inputs vector, then we add it as a public input

        if (std::find(constraint_system.public_inputs.begin(), constraint_system.public_inputs.end(), i) !=
            constraint_system.public_inputs.end()) {

            composer.add_public_variable(0);

        } else {
            composer.add_variable(0);
        }
    }

    read_witness(composer, witness);

    // Add arithmetic gates
    for (const auto& constraint : constraint_system.constraints) {
        composer.create_poly_gate(constraint);
    }

    // Add logic constraint
    for (const auto& constraint : constraint_system.logic_constraints) {
        create_logic_gate(
            composer, constraint.a, constraint.b, constraint.result, constraint.num_bits, constraint.is_xor_gate);
    }

    // Add range constraint
    for (const auto& constraint : constraint_system.range_constraints) {
        composer.create_range_constraint(constraint.witness, constraint.num_bits, "");
    }

    // Add sha256 constraints
    for (const auto& constraint : constraint_system.sha256_constraints) {
        create_sha256_constraints(composer, constraint);
    }

    // Add schnorr constraints
    for (const auto& constraint : constraint_system.schnorr_constraints) {
        create_schnorr_verify_constraints(composer, constraint);
    }

    // Add ECDSA constraints
    for (const auto& constraint : constraint_system.ecdsa_constraints) {
        create_ecdsa_verify_constraints(composer, constraint);
    }

    // Add blake2s constraints
    for (const auto& constraint : constraint_system.blake2s_constraints) {
        create_blake2s_constraints(composer, constraint);
    }

    // Add keccak constraints
    for (const auto& constraint : constraint_system.keccak_constraints) {
        create_keccak_constraints(composer, constraint);
    }
    for (const auto& constraint : constraint_system.keccak_var_constraints) {
        create_keccak_var_constraints(composer, constraint);
    }

    // Add pedersen constraints
    for (const auto& constraint : constraint_system.pedersen_constraints) {
        create_pedersen_constraint(composer, constraint);
    }

    // Add fixed base scalar mul constraints
    for (const auto& constraint : constraint_system.fixed_base_scalar_mul_constraints) {
        create_fixed_base_constraint(composer, constraint);
    }

    // Add hash to field constraints
    for (const auto& constraint : constraint_system.hash_to_field_constraints) {
        create_hash_to_field_constraints(composer, constraint);
    }

    // Add block constraints
    for (const auto& constraint : constraint_system.block_constraints) {
        create_block_constraints(composer, constraint);
    }

    // Add recursion constraints
    for (size_t i = 0; i < constraint_system.recursion_constraints.size(); ++i) {
        auto& constraint = constraint_system.recursion_constraints[i];
        create_recursion_constraints(composer, constraint, true);

        // make sure the verification key records the public input indices of the final recursion output
        // (N.B. up to the ACIR description to make sure that the final output aggregation object wires are public
        // inputs!)
        if (i == constraint_system.recursion_constraints.size() - 1) {
            std::vector<uint32_t> proof_output_witness_indices(constraint.output_aggregation_object.begin(),
                                                               constraint.output_aggregation_object.end());
            composer.set_recursive_proof(proof_output_witness_indices);
        }
    }
}

} // namespace acir_format
