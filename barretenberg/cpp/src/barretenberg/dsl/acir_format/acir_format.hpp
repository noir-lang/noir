#pragma once
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/dsl/types.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "blake2s_constraint.hpp"
#include "block_constraint.hpp"
#include "ecdsa_secp256k1.hpp"
#include "ecdsa_secp256r1.hpp"
#include "fixed_base_scalar_mul.hpp"
#include "hash_to_field.hpp"
#include "keccak_constraint.hpp"
#include "logic_constraint.hpp"
#include "pedersen.hpp"
#include "range_constraint.hpp"
#include "recursion_constraint.hpp"
#include "schnorr_verify.hpp"
#include "sha256_constraint.hpp"

namespace acir_format {

struct acir_format {
    // The number of witnesses in the circuit
    uint32_t varnum;

    std::vector<uint32_t> public_inputs;

    std::vector<LogicConstraint> logic_constraints;
    std::vector<RangeConstraint> range_constraints;
    std::vector<Sha256Constraint> sha256_constraints;
    std::vector<SchnorrConstraint> schnorr_constraints;
    std::vector<EcdsaSecp256k1Constraint> ecdsa_k1_constraints;
    std::vector<EcdsaSecp256r1Constraint> ecdsa_r1_constraints;
    std::vector<Blake2sConstraint> blake2s_constraints;
    std::vector<KeccakConstraint> keccak_constraints;
    std::vector<KeccakVarConstraint> keccak_var_constraints;
    std::vector<PedersenConstraint> pedersen_constraints;
    std::vector<HashToFieldConstraint> hash_to_field_constraints;
    std::vector<FixedBaseScalarMul> fixed_base_scalar_mul_constraints;
    std::vector<RecursionConstraint> recursion_constraints;
    // A standard plonk arithmetic constraint, as defined in the poly_triple struct, consists of selector values
    // for q_M,q_L,q_R,q_O,q_C and indices of three variables taking the role of left, right and output wire
    // This could be a large vector so use slab allocator, we don't expect the blackbox implementations to be so large.
    std::vector<poly_triple_<curve::BN254::ScalarField>,
                ContainerSlabAllocator<poly_triple_<curve::BN254::ScalarField>>>
        constraints;
    std::vector<BlockConstraint> block_constraints;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(varnum,
                   public_inputs,
                   logic_constraints,
                   range_constraints,
                   sha256_constraints,
                   schnorr_constraints,
                   ecdsa_k1_constraints,
                   ecdsa_r1_constraints,
                   blake2s_constraints,
                   keccak_constraints,
                   keccak_var_constraints,
                   pedersen_constraints,
                   hash_to_field_constraints,
                   fixed_base_scalar_mul_constraints,
                   recursion_constraints,
                   constraints,
                   block_constraints);

    friend bool operator==(acir_format const& lhs, acir_format const& rhs) = default;
};

using WitnessVector = std::vector<fr, ContainerSlabAllocator<fr>>;

void read_witness(Builder& builder, std::vector<barretenberg::fr> const& witness);

void create_circuit(Builder& builder, const acir_format& constraint_system);

Builder create_circuit(const acir_format& constraint_system, size_t size_hint = 0);

Builder create_circuit_with_witness(const acir_format& constraint_system,
                                    WitnessVector const& witness,
                                    size_t size_hint = 0);

void create_circuit_with_witness(Builder& builder, const acir_format& constraint_system, WitnessVector const& witness);

} // namespace acir_format
