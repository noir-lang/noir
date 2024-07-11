#pragma once
#include "aes128_constraint.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/serialize/msgpack.hpp"
#include "bigint_constraint.hpp"
#include "blake2s_constraint.hpp"
#include "blake3_constraint.hpp"
#include "block_constraint.hpp"
#include "ec_operations.hpp"
#include "ecdsa_secp256k1.hpp"
#include "ecdsa_secp256r1.hpp"
#include "honk_recursion_constraint.hpp"
#include "keccak_constraint.hpp"
#include "logic_constraint.hpp"
#include "multi_scalar_mul.hpp"
#include "pedersen.hpp"
#include "poseidon2_constraint.hpp"
#include "range_constraint.hpp"
#include "recursion_constraint.hpp"
#include "schnorr_verify.hpp"
#include "sha256_constraint.hpp"
#include <utility>
#include <vector>

namespace acir_format {

/**
 * @brief Indices of the original opcode that originated each constraint in AcirFormat.
 * @details Contains one array of indices per opcode type. The length of each array is equal to the number of
 * constraints of that type. The relationship between the opcodes and constraints is assumed to be one to one, except
 * for block constraints.
 */
struct AcirFormatOriginalOpcodeIndices {
    std::vector<size_t> logic_constraints;
    std::vector<size_t> range_constraints;
    std::vector<size_t> aes128_constraints;
    std::vector<size_t> sha256_constraints;
    std::vector<size_t> sha256_compression;
    std::vector<size_t> schnorr_constraints;
    std::vector<size_t> ecdsa_k1_constraints;
    std::vector<size_t> ecdsa_r1_constraints;
    std::vector<size_t> blake2s_constraints;
    std::vector<size_t> blake3_constraints;
    std::vector<size_t> keccak_constraints;
    std::vector<size_t> keccak_permutations;
    std::vector<size_t> pedersen_constraints;
    std::vector<size_t> pedersen_hash_constraints;
    std::vector<size_t> poseidon2_constraints;
    std::vector<size_t> multi_scalar_mul_constraints;
    std::vector<size_t> ec_add_constraints;
    std::vector<size_t> recursion_constraints;
    std::vector<size_t> honk_recursion_constraints;
    std::vector<size_t> bigint_from_le_bytes_constraints;
    std::vector<size_t> bigint_to_le_bytes_constraints;
    std::vector<size_t> bigint_operations;
    std::vector<size_t> poly_triple_constraints;
    std::vector<size_t> quad_constraints;
    // Multiple opcode indices per block:
    std::vector<std::vector<size_t>> block_constraints;

    friend bool operator==(AcirFormatOriginalOpcodeIndices const& lhs,
                           AcirFormatOriginalOpcodeIndices const& rhs) = default;
};

struct AcirFormat {
    // The number of witnesses in the circuit
    uint32_t varnum;
    // Specifies whether a prover that produces SNARK recursion friendly proofs should be used.
    // The proof produced when this flag is true should be friendly for recursive verification inside
    // of another SNARK. For example, a recursive friendly proof may use Blake3Pedersen for
    // hashing in its transcript, while we still want a prove that uses Keccak for its transcript in order
    // to be able to verify SNARKs on Ethereum.
    bool recursive;

    uint32_t num_acir_opcodes;

    std::vector<uint32_t> public_inputs;

    std::vector<LogicConstraint> logic_constraints;
    std::vector<RangeConstraint> range_constraints;
    std::vector<AES128Constraint> aes128_constraints;
    std::vector<Sha256Constraint> sha256_constraints;
    std::vector<Sha256Compression> sha256_compression;
    std::vector<SchnorrConstraint> schnorr_constraints;
    std::vector<EcdsaSecp256k1Constraint> ecdsa_k1_constraints;
    std::vector<EcdsaSecp256r1Constraint> ecdsa_r1_constraints;
    std::vector<Blake2sConstraint> blake2s_constraints;
    std::vector<Blake3Constraint> blake3_constraints;
    std::vector<KeccakConstraint> keccak_constraints;
    std::vector<Keccakf1600> keccak_permutations;
    std::vector<PedersenConstraint> pedersen_constraints;
    std::vector<PedersenHashConstraint> pedersen_hash_constraints;
    std::vector<Poseidon2Constraint> poseidon2_constraints;
    std::vector<MultiScalarMul> multi_scalar_mul_constraints;
    std::vector<EcAdd> ec_add_constraints;
    std::vector<RecursionConstraint> recursion_constraints;
    std::vector<HonkRecursionConstraint> honk_recursion_constraints;
    std::vector<BigIntFromLeBytes> bigint_from_le_bytes_constraints;
    std::vector<BigIntToLeBytes> bigint_to_le_bytes_constraints;
    std::vector<BigIntOperation> bigint_operations;

    // A standard plonk arithmetic constraint, as defined in the poly_triple struct, consists of selector values
    // for q_M,q_L,q_R,q_O,q_C and indices of three variables taking the role of left, right and output wire
    // This could be a large vector so use slab allocator, we don't expect the blackbox implementations to be so large.
    std::vector<bb::poly_triple_<bb::curve::BN254::ScalarField>,
                bb::ContainerSlabAllocator<bb::poly_triple_<bb::curve::BN254::ScalarField>>>
        poly_triple_constraints;
    std::vector<bb::mul_quad_<bb::curve::BN254::ScalarField>,
                bb::ContainerSlabAllocator<bb::mul_quad_<bb::curve::BN254::ScalarField>>>
        quad_constraints;
    std::vector<BlockConstraint> block_constraints;

    // Number of gates added to the circuit per original opcode.
    // Has length equal to num_acir_opcodes.
    std::vector<size_t> gates_per_opcode = {};

    // Indices of the original opcode that originated each constraint in AcirFormat.
    AcirFormatOriginalOpcodeIndices original_opcode_indices;

    // For serialization, update with any new fields
    MSGPACK_FIELDS(varnum,
                   public_inputs,
                   logic_constraints,
                   range_constraints,
                   aes128_constraints,
                   sha256_constraints,
                   sha256_compression,
                   schnorr_constraints,
                   ecdsa_k1_constraints,
                   ecdsa_r1_constraints,
                   blake2s_constraints,
                   blake3_constraints,
                   keccak_constraints,
                   keccak_permutations,
                   pedersen_constraints,
                   pedersen_hash_constraints,
                   poseidon2_constraints,
                   multi_scalar_mul_constraints,
                   ec_add_constraints,
                   recursion_constraints,
                   honk_recursion_constraints,
                   poly_triple_constraints,
                   block_constraints,
                   bigint_from_le_bytes_constraints,
                   bigint_to_le_bytes_constraints,
                   bigint_operations);

    friend bool operator==(AcirFormat const& lhs, AcirFormat const& rhs) = default;
};

using WitnessVector = std::vector<bb::fr, bb::ContainerSlabAllocator<bb::fr>>;
using WitnessVectorStack = std::vector<std::pair<uint32_t, WitnessVector>>;

struct AcirProgram {
    AcirFormat constraints;
    WitnessVector witness;
};

/**
 * @brief Storage for constaint_systems/witnesses for a stack of acir programs
 * @details In general the number of items in the witness stack will be equal or greater than the number of constraint
 * systems because the program may consist of multiple calls to the same function.
 *
 */
struct AcirProgramStack {
    std::vector<AcirFormat> constraint_systems;
    WitnessVectorStack witness_stack;

    AcirProgramStack(const std::vector<AcirFormat>& constraint_systems_in, const WitnessVectorStack& witness_stack_in)
        : constraint_systems(constraint_systems_in)
        , witness_stack(witness_stack_in)
    {}

    size_t size() const { return witness_stack.size(); }
    bool empty() const { return witness_stack.empty(); }

    AcirProgram back()
    {
        auto witness_stack_item = witness_stack.back();
        auto witness = witness_stack_item.second;
        auto constraint_system = constraint_systems[witness_stack_item.first];

        return { constraint_system, witness };
    }

    void pop_back() { witness_stack.pop_back(); }
};

template <typename Builder = bb::UltraCircuitBuilder>
Builder create_circuit(AcirFormat& constraint_system,
                       size_t size_hint = 0,
                       WitnessVector const& witness = {},
                       bool honk_recursion = false,
                       std::shared_ptr<bb::ECCOpQueue> op_queue = std::make_shared<bb::ECCOpQueue>(),
                       bool collect_gates_per_opcode = false);

template <typename Builder>
void build_constraints(
    Builder& builder,
    AcirFormat& constraint_system,
    bool has_valid_witness_assignments,
    bool honk_recursion = false,
    bool collect_gates_per_opcode = false); // honk_recursion means we will honk to recursively verify this
                                            // circuit. This distinction is needed to not add the default
                                            // aggregation object when we're not using the honk RV.

} // namespace acir_format
