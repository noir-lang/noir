#include "honk_recursion_constraint.hpp"
#include "acir_format.hpp"
#include "acir_format_mocks.hpp"
#include "barretenberg/sumcheck/instance/prover_instance.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"
#include "barretenberg/ultra_honk/ultra_verifier.hpp"
#include "proof_surgeon.hpp"

#include <gtest/gtest.h>
#include <vector>

using namespace acir_format;
using namespace bb;

class AcirHonkRecursionConstraint : public ::testing::Test {

  public:
    using ProverInstance = ProverInstance_<UltraFlavor>;
    using Prover = bb::UltraProver;
    using VerificationKey = UltraFlavor::VerificationKey;
    using Verifier = bb::UltraVerifier;

    Builder create_inner_circuit()
    {
        /**
         * constraints produced by Noir program:
         * fn main(x : u32, y : pub u32) {
         * let z = x ^ y;
         *
         * constrain z != 10;
         * }
         **/
        RangeConstraint range_a{
            .witness = 0,
            .num_bits = 32,
        };
        RangeConstraint range_b{
            .witness = 1,
            .num_bits = 32,
        };

        LogicConstraint logic_constraint{
            .a = WitnessOrConstant<bb::fr>::from_index(0),
            .b = WitnessOrConstant<bb::fr>::from_index(1),
            .result = 2,
            .num_bits = 32,
            .is_xor_gate = 1,
        };
        poly_triple expr_a{
            .a = 2,
            .b = 3,
            .c = 0,
            .q_m = 0,
            .q_l = 1,
            .q_r = -1,
            .q_o = 0,
            .q_c = -10,
        };
        poly_triple expr_b{
            .a = 3,
            .b = 4,
            .c = 5,
            .q_m = 1,
            .q_l = 0,
            .q_r = 0,
            .q_o = -1,
            .q_c = 0,
        };
        poly_triple expr_c{
            .a = 3,
            .b = 5,
            .c = 3,
            .q_m = 1,
            .q_l = 0,
            .q_r = 0,
            .q_o = -1,
            .q_c = 0,

        };
        poly_triple expr_d{
            .a = 5,
            .b = 0,
            .c = 0,
            .q_m = 0,
            .q_l = -1,
            .q_r = 0,
            .q_o = 0,
            .q_c = 1,
        };

        AcirFormat constraint_system{
            .varnum = 6,
            .recursive = true,
            .num_acir_opcodes = 7,
            .public_inputs = { 1, 2 },
            .logic_constraints = { logic_constraint },
            .range_constraints = { range_a, range_b },
            .aes128_constraints = {},
            .sha256_constraints = {},
            .sha256_compression = {},
            .schnorr_constraints = {},
            .ecdsa_k1_constraints = {},
            .ecdsa_r1_constraints = {},
            .blake2s_constraints = {},
            .blake3_constraints = {},
            .keccak_constraints = {},
            .keccak_permutations = {},
            .pedersen_constraints = {},
            .pedersen_hash_constraints = {},
            .poseidon2_constraints = {},
            .multi_scalar_mul_constraints = {},
            .ec_add_constraints = {},
            .recursion_constraints = {},
            .honk_recursion_constraints = {},
            .bigint_from_le_bytes_constraints = {},
            .bigint_to_le_bytes_constraints = {},
            .bigint_operations = {},
            .poly_triple_constraints = { expr_a, expr_b, expr_c, expr_d },
            .quad_constraints = {},
            .block_constraints = {},
            .original_opcode_indices = create_empty_original_opcode_indices(),
        };
        mock_opcode_indices(constraint_system);

        uint256_t inverse_of_five = fr(5).invert();
        WitnessVector witness{
            5, 10, 15, 5, inverse_of_five, 1,
        };
        auto builder = create_circuit(constraint_system, /*size_hint*/ 0, witness, /*honk recursion*/ true);

        return builder;
    }

    /**
     * @brief Create a circuit that recursively verifies one or more inner circuits
     *
     * @param inner_circuits
     * @return Composer
     */
    Builder create_outer_circuit(std::vector<Builder>& inner_circuits)
    {
        std::vector<RecursionConstraint> honk_recursion_constraints;

        SlabVector<fr> witness;

        for (auto& inner_circuit : inner_circuits) {

            auto instance = std::make_shared<ProverInstance>(inner_circuit);
            Prover prover(instance);
            auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
            Verifier verifier(verification_key);
            auto inner_proof = prover.construct_proof();

            std::vector<bb::fr> key_witnesses = verification_key->to_field_elements();
            std::vector<fr> proof_witnesses = inner_proof;
            const size_t num_public_inputs = inner_circuit.get_public_inputs().size();

            auto [key_indices, proof_indices, inner_public_inputs] = ProofSurgeon::populate_recursion_witness_data(
                witness, proof_witnesses, key_witnesses, num_public_inputs);

            RecursionConstraint honk_recursion_constraint{
                .key = key_indices,
                .proof = proof_indices,
                .public_inputs = inner_public_inputs,
                .key_hash = 0, // not used
                .proof_type = HONK_RECURSION,
            };
            honk_recursion_constraints.push_back(honk_recursion_constraint);
        }

        std::vector<size_t> honk_recursion_opcode_indices(honk_recursion_constraints.size());
        std::iota(honk_recursion_opcode_indices.begin(), honk_recursion_opcode_indices.end(), 0);

        AcirFormat constraint_system{
            .varnum = static_cast<uint32_t>(witness.size()),
            .recursive = false,
            .num_acir_opcodes = static_cast<uint32_t>(honk_recursion_constraints.size()),
            .public_inputs = {},
            .logic_constraints = {},
            .range_constraints = {},
            .aes128_constraints = {},
            .sha256_constraints = {},
            .sha256_compression = {},
            .schnorr_constraints = {},
            .ecdsa_k1_constraints = {},
            .ecdsa_r1_constraints = {},
            .blake2s_constraints = {},
            .blake3_constraints = {},
            .keccak_constraints = {},
            .keccak_permutations = {},
            .pedersen_constraints = {},
            .pedersen_hash_constraints = {},
            .poseidon2_constraints = {},
            .multi_scalar_mul_constraints = {},
            .ec_add_constraints = {},
            .recursion_constraints = {},
            .honk_recursion_constraints = honk_recursion_constraints,
            .bigint_from_le_bytes_constraints = {},
            .bigint_to_le_bytes_constraints = {},
            .bigint_operations = {},
            .poly_triple_constraints = {},
            .quad_constraints = {},
            .block_constraints = {},
            .original_opcode_indices = create_empty_original_opcode_indices(),
        };
        mock_opcode_indices(constraint_system);
        auto outer_circuit = create_circuit(constraint_system, /*size_hint*/ 0, witness, /*honk recursion*/ true);

        return outer_circuit;
    }

  protected:
    static void SetUpTestSuite() { bb::srs::init_crs_factory("../srs_db/ignition"); }
};

TEST_F(AcirHonkRecursionConstraint, TestBasicSingleHonkRecursionConstraint)
{
    std::vector<Builder> layer_1_circuits;
    layer_1_circuits.push_back(create_inner_circuit());

    auto layer_2_circuit = create_outer_circuit(layer_1_circuits);

    info("circuit gates = ", layer_2_circuit.get_num_gates());

    auto instance = std::make_shared<ProverInstance>(layer_2_circuit);
    Prover prover(instance);
    info("prover gates = ", instance->proving_key.circuit_size);
    auto proof = prover.construct_proof();
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirHonkRecursionConstraint, TestBasicDoubleHonkRecursionConstraints)
{
    std::vector<Builder> layer_1_circuits;
    layer_1_circuits.push_back(create_inner_circuit());

    layer_1_circuits.push_back(create_inner_circuit());

    auto layer_2_circuit = create_outer_circuit(layer_1_circuits);

    info("circuit gates = ", layer_2_circuit.get_num_gates());

    auto instance = std::make_shared<ProverInstance>(layer_2_circuit);
    Prover prover(instance);
    info("prover gates = ", instance->proving_key.circuit_size);
    auto proof = prover.construct_proof();
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirHonkRecursionConstraint, TestOneOuterRecursiveCircuit)
{
    /**
     * We want to test the following:
     * 1. circuit that verifies a proof of another circuit
     * 2. the above, but the inner circuit contains a recursive proof output that we have to aggregate
     * 3. the above, but the outer circuit verifies 2 proofs, the aggregation outputs from the 2 proofs (+ the recursive
     * proof output from 2) are aggregated together
     *
     * A = basic circuit
     * B = circuit that verifies proof of A
     * C = circuit that verifies proof of B and a proof of A
     *
     * Layer 1 = proof of A
     * Layer 2 = verifies proof of A and proof of B
     * Layer 3 = verifies proof of C
     *
     * Attempt at a visual graphic
     * ===========================
     *
     *     C
     *     ^
     *     |
     *     | - B
     *     ^   ^
     *     |   |
     *     |    -A
     *     |
     *      - A
     *
     * ===========================
     *
     * Final aggregation object contains aggregated proofs for 2 instances of A and 1 instance of B
     */
    std::vector<Builder> layer_1_circuits;
    layer_1_circuits.push_back(create_inner_circuit());
    info("created first inner circuit");

    std::vector<Builder> layer_2_circuits;
    layer_2_circuits.push_back(create_inner_circuit());
    info("created second inner circuit");

    layer_2_circuits.push_back(create_outer_circuit(layer_1_circuits));
    info("created first outer circuit");

    auto layer_3_circuit = create_outer_circuit(layer_2_circuits);
    info("created second outer circuit");
    info("number of gates in layer 3 = ", layer_3_circuit.get_num_gates());

    auto instance = std::make_shared<ProverInstance>(layer_3_circuit);
    Prover prover(instance);
    info("prover gates = ", instance->proving_key.circuit_size);
    auto proof = prover.construct_proof();
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}

TEST_F(AcirHonkRecursionConstraint, TestFullRecursiveComposition)
{
    std::vector<Builder> layer_b_1_circuits;
    layer_b_1_circuits.push_back(create_inner_circuit());
    info("created first inner circuit");

    std::vector<Builder> layer_b_2_circuits;
    layer_b_2_circuits.push_back(create_inner_circuit());
    info("created second inner circuit");

    std::vector<Builder> layer_2_circuits;
    layer_2_circuits.push_back(create_outer_circuit(layer_b_1_circuits));
    info("created first outer circuit");

    layer_2_circuits.push_back(create_outer_circuit(layer_b_2_circuits));
    info("created second outer circuit");

    auto layer_3_circuit = create_outer_circuit(layer_2_circuits);
    info("created third outer circuit");
    info("number of gates in layer 3 circuit = ", layer_3_circuit.get_num_gates());

    auto instance = std::make_shared<ProverInstance>(layer_3_circuit);
    Prover prover(instance);
    info("prover gates = ", instance->proving_key.circuit_size);
    auto proof = prover.construct_proof();
    auto verification_key = std::make_shared<VerificationKey>(instance->proving_key);
    Verifier verifier(verification_key);
    EXPECT_EQ(verifier.verify_proof(proof), true);
}
