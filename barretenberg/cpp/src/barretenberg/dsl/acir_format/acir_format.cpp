#include "acir_format.hpp"
#include "barretenberg/common/log.hpp"
#include "barretenberg/dsl/acir_format/pedersen.hpp"
#include "barretenberg/proof_system/circuit_builder/ultra_circuit_builder.hpp"

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

    // Add hash to field constraints
    for (const auto& constraint : constraint_system.hash_to_field_constraints) {
        create_hash_to_field_constraints(builder, constraint);
    }

    // Add block constraints
    for (const auto& constraint : constraint_system.block_constraints) {
        create_block_constraints(builder, constraint, has_valid_witness_assignments);
    }

    // Add recursion constraints
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/817): disable these for UGH for now since we're not yet
    // dealing with proper recursion
    if constexpr (IsGoblinBuilder<Builder>) {
        info("WARNING: this circuit contains recursion_constraints!");
    } else {
        for (size_t i = 0; i < constraint_system.recursion_constraints.size(); ++i) {
            auto& constraint = constraint_system.recursion_constraints[i];
            create_recursion_constraints(builder, constraint, has_valid_witness_assignments);

            // make sure the verification key records the public input indices of the final recursion output (N.B. up to
            // the ACIR description to make sure that the final output aggregation object wires are public inputs!)
            if (i == constraint_system.recursion_constraints.size() - 1) {
                std::vector<uint32_t> proof_output_witness_indices(constraint.output_aggregation_object.begin(),
                                                                   constraint.output_aggregation_object.end());
                builder.set_recursive_proof(proof_output_witness_indices);
            }
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

/**
 * @brief Apply an offset to the indices stored in the wires
 * @details This method is needed due to the following: Noir constructs "wires" as indices into a "witness" vector. This
 * is analogous to the wires and variables vectors in bberg builders. Were it not for the addition of constant variables
 * in the constructors of a builder (e.g. zero), we would simply have noir.wires = builder.wires and noir.witness =
 * builder.variables. To account for k-many constant variables in the first entries of the variables array, we have
 * something like variables = variables.append(noir.witness). Accordingly, the indices in noir.wires have to be
 * incremented to account for the offset at which noir.wires was placed into variables.
 *
 * @tparam Builder
 * @param builder
 */
template <typename Builder> void apply_wire_index_offset(Builder& builder)
{
    // For now, noir has a hard coded witness index offset = 1. Once this is removed, this pre-applied offset goes away
    const uint32_t pre_applied_noir_offset = 1;
    auto offset = static_cast<uint32_t>(builder.num_vars_added_in_constructor - pre_applied_noir_offset);
    info("Applying offset = ", offset);

    // Apply the offset to the indices stored the wires that were generated from acir. (Do not apply the offset to those
    // values that were added in the builder constructor).
    size_t start_index = builder.num_vars_added_in_constructor;
    for (auto& wire : builder.wires) {
        for (size_t idx = start_index; idx < wire.size(); ++idx) {
            wire[idx] += offset;
        }
    }
}

template UltraCircuitBuilder create_circuit<UltraCircuitBuilder>(const acir_format& constraint_system,
                                                                 size_t size_hint);
template void create_circuit_with_witness<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                               acir_format const& constraint_system,
                                                               WitnessVector const& witness);
template void create_circuit_with_witness<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                     acir_format const& constraint_system,
                                                                     WitnessVector const& witness);
template void apply_wire_index_offset<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder);
template void apply_wire_index_offset<UltraCircuitBuilder>(UltraCircuitBuilder& builder);

} // namespace acir_format
