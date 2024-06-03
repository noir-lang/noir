#include "acir_format.hpp"
#include "barretenberg/common/log.hpp"
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
                       AcirFormat const& constraint_system,
                       bool has_valid_witness_assignments,
                       bool honk_recursion)
{
    // Add arithmetic gates
    for (const auto& constraint : constraint_system.poly_triple_constraints) {
        builder.create_poly_gate(constraint);
    }
    for (const auto& constraint : constraint_system.quad_constraints) {
        builder.create_big_mul_gate(constraint);
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

    // Add aes128 constraints
    for (const auto& constraint : constraint_system.aes128_constraints) {
        create_aes128_constraints(builder, constraint);
    }

    // Add sha256 constraints
    for (const auto& constraint : constraint_system.sha256_constraints) {
        create_sha256_constraints(builder, constraint);
    }
    for (const auto& constraint : constraint_system.sha256_compression) {
        create_sha256_compression_constraints(builder, constraint);
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

    // Add blake3 constraints
    for (const auto& constraint : constraint_system.blake3_constraints) {
        create_blake3_constraints(builder, constraint);
    }

    // Add keccak constraints
    for (const auto& constraint : constraint_system.keccak_constraints) {
        create_keccak_constraints(builder, constraint);
    }
    for (const auto& constraint : constraint_system.keccak_permutations) {
        create_keccak_permutations(builder, constraint);
    }

    // Add pedersen constraints
    for (const auto& constraint : constraint_system.pedersen_constraints) {
        create_pedersen_constraint(builder, constraint);
    }

    for (const auto& constraint : constraint_system.pedersen_hash_constraints) {
        create_pedersen_hash_constraint(builder, constraint);
    }

    for (const auto& constraint : constraint_system.poseidon2_constraints) {
        create_poseidon2_permutations(builder, constraint);
    }

    // Add multi scalar mul constraints
    for (const auto& constraint : constraint_system.multi_scalar_mul_constraints) {
        create_multi_scalar_mul_constraint(builder, constraint);
    }

    // Add ec add constraints
    for (const auto& constraint : constraint_system.ec_add_constraints) {
        create_ec_add_constraint(builder, constraint, has_valid_witness_assignments);
    }

    // Add block constraints
    for (const auto& constraint : constraint_system.block_constraints) {
        create_block_constraints(builder, constraint, has_valid_witness_assignments);
    }

    // Add big_int constraints
    DSLBigInts<Builder> dsl_bigints;
    dsl_bigints.set_builder(&builder);
    for (const auto& constraint : constraint_system.bigint_from_le_bytes_constraints) {
        create_bigint_from_le_bytes_constraint(builder, constraint, dsl_bigints);
    }
    for (const auto& constraint : constraint_system.bigint_operations) {
        create_bigint_operations_constraint<Builder>(constraint, dsl_bigints, has_valid_witness_assignments);
    }
    for (const auto& constraint : constraint_system.bigint_to_le_bytes_constraints) {
        create_bigint_to_le_bytes_constraint(builder, constraint, dsl_bigints);
    }

    // RecursionConstraint
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/817): disable these for MegaHonk for now since we're
    // not yet dealing with proper recursion
    if constexpr (IsMegaBuilder<Builder>) {
        if (!constraint_system.recursion_constraints.empty()) {
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
                if (constraint.proof.size() - proof_size_no_pub_inputs !=
                    RecursionConstraint::AGGREGATION_OBJECT_SIZE) {
                    auto error_string = format(
                        "Public inputs are always stripped from proofs unless we have a recursive proof.\n"
                        "Thus, public inputs attached to a proof must match the recursive aggregation object in size "
                        "which is ",
                        RecursionConstraint::AGGREGATION_OBJECT_SIZE);
                    throw_or_abort(error_string);
                }
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

    // HonkRecursionConstraint
    // TODO(https://github.com/AztecProtocol/barretenberg/issues/817): disable these for MegaHonk for now since we're
    // not yet dealing with proper recursion
    if constexpr (IsMegaBuilder<Builder>) {
        if (!constraint_system.honk_recursion_constraints.empty()) {
            info("WARNING: this circuit contains honk_recursion_constraints!");
        }
    } else {
        // These are set and modified whenever we encounter a recursion opcode
        //
        // These should not be set by the caller
        // TODO(https://github.com/AztecProtocol/barretenberg/issues/996): this usage of all zeros is a hack and could
        // use types or enums to properly fix.
        std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> current_aggregation_object = {
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        };

        // Add recursion constraints
        for (auto constraint : constraint_system.honk_recursion_constraints) {
            // A proof passed into the constraint should be stripped of its inner public inputs, but not the nested
            // aggregation object itself. The verifier circuit requires that the indices to a nested proof aggregation
            // state are a circuit constant. The user tells us they how they want these constants set by keeping the
            // nested aggregation object attached to the proof as public inputs.
            std::array<uint32_t, HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE> nested_aggregation_object = {};
            for (size_t i = 0; i < HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE; ++i) {
                // Set the nested aggregation object indices to witness indices from the proof
                nested_aggregation_object[i] =
                    static_cast<uint32_t>(constraint.proof[HonkRecursionConstraint::inner_public_input_offset + i]);
                // Adding the nested aggregation object to the constraint's public inputs
                constraint.public_inputs.emplace_back(nested_aggregation_object[i]);
            }
            // Remove the aggregation object so that they can be handled as normal public inputs
            // in they way that the recursion constraint expects
            constraint.proof.erase(constraint.proof.begin() + HonkRecursionConstraint::inner_public_input_offset,
                                   constraint.proof.begin() +
                                       static_cast<std::ptrdiff_t>(HonkRecursionConstraint::inner_public_input_offset +
                                                                   HonkRecursionConstraint::AGGREGATION_OBJECT_SIZE));
            current_aggregation_object = create_honk_recursion_constraints(builder,
                                                                           constraint,
                                                                           current_aggregation_object,
                                                                           nested_aggregation_object,
                                                                           has_valid_witness_assignments);
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
            std::vector<uint32_t> proof_output_witness_indices(current_aggregation_object.begin(),
                                                               current_aggregation_object.end());
            builder.set_recursive_proof(proof_output_witness_indices);
        } else if (honk_recursion &&
                   builder.is_recursive_circuit) { // Set a default aggregation object if we don't have one.
            // TODO(https://github.com/AztecProtocol/barretenberg/issues/911): These are pairing points extracted from
            // a valid proof. This is a workaround because we can't represent the point at infinity in biggroup yet.
            fq x0("0x031e97a575e9d05a107acb64952ecab75c020998797da7842ab5d6d1986846cf");
            fq y0("0x178cbf4206471d722669117f9758a4c410db10a01750aebb5666547acf8bd5a4");

            fq x1("0x0f94656a2ca489889939f81e9c74027fd51009034b3357f0e91b8a11e7842c38");
            fq y1("0x1b52c2020d7464a0c80c0da527a08193fe27776f50224bd6fb128b46c1ddb67f");
            std::vector<fq> aggregation_object_fq_values = { x0, y0, x1, y1 };
            size_t agg_obj_indices_idx = 0;
            for (fq val : aggregation_object_fq_values) {
                const uint256_t x = val;
                std::array<fr, fq_ct::NUM_LIMBS> val_limbs = {
                    x.slice(0, fq_ct::NUM_LIMB_BITS),
                    x.slice(fq_ct::NUM_LIMB_BITS, fq_ct::NUM_LIMB_BITS * 2),
                    x.slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 3),
                    x.slice(fq_ct::NUM_LIMB_BITS * 3, stdlib::field_conversion::TOTAL_BITS)
                };
                for (size_t i = 0; i < fq_ct::NUM_LIMBS; ++i) {
                    uint32_t idx = builder.add_variable(val_limbs[i]);
                    builder.set_public_input(idx);
                    current_aggregation_object[agg_obj_indices_idx] = idx;
                    agg_obj_indices_idx++;
                }
            }
            // Make sure the verification key records the public input indices of the
            // final recursion output.
            std::vector<uint32_t> proof_output_witness_indices(current_aggregation_object.begin(),
                                                               current_aggregation_object.end());
            builder.set_recursive_proof(proof_output_witness_indices);
        }
    }
}

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
UltraCircuitBuilder create_circuit(const AcirFormat& constraint_system,
                                   size_t size_hint,
                                   WitnessVector const& witness,
                                   bool honk_recursion,
                                   [[maybe_unused]] std::shared_ptr<ECCOpQueue>)
{
    Builder builder{
        size_hint, witness, constraint_system.public_inputs, constraint_system.varnum, constraint_system.recursive
    };

    bool has_valid_witness_assignments = !witness.empty();
    build_constraints(builder, constraint_system, has_valid_witness_assignments, honk_recursion);

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
MegaCircuitBuilder create_circuit(const AcirFormat& constraint_system,
                                  [[maybe_unused]] size_t size_hint,
                                  WitnessVector const& witness,
                                  bool honk_recursion,
                                  std::shared_ptr<ECCOpQueue> op_queue)
{
    // Construct a builder using the witness and public input data from acir and with the goblin-owned op_queue
    auto builder = MegaCircuitBuilder{ op_queue, witness, constraint_system.public_inputs, constraint_system.varnum };

    // Populate constraints in the builder via the data in constraint_system
    bool has_valid_witness_assignments = !witness.empty();
    acir_format::build_constraints(builder, constraint_system, has_valid_witness_assignments, honk_recursion);

    return builder;
};

template void build_constraints<MegaCircuitBuilder>(MegaCircuitBuilder&, AcirFormat const&, bool, bool);

} // namespace acir_format
