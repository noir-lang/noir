#pragma once

#include "barretenberg/commitment_schemes/commitment_key.hpp"
#include "barretenberg/common/op_count.hpp"
#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/crypto/merkle_tree/membership.hpp"
#include "barretenberg/crypto/merkle_tree/memory_store.hpp"
#include "barretenberg/crypto/merkle_tree/merkle_tree.hpp"
#include "barretenberg/srs/global_crs.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/protogalaxy_recursive_verifier.hpp"
#include "barretenberg/stdlib/honk_recursion/verifier/ultra_recursive_verifier.hpp"
#include "barretenberg/stdlib/primitives/curves/secp256k1.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"
#include "barretenberg/stdlib_circuit_builders/mega_flavor.hpp"
#include "barretenberg/stdlib_circuit_builders/mock_circuits.hpp"

namespace bb {

class GoblinMockCircuits {
  public:
    using Curve = curve::BN254;
    using FF = Curve::ScalarField;
    using Fbase = Curve::BaseField;
    using Point = Curve::AffineElement;
    using CommitmentKey = bb::CommitmentKey<Curve>;
    using OpQueue = bb::ECCOpQueue;
    using MegaBuilder = bb::MegaCircuitBuilder;
    using Flavor = bb::MegaFlavor;
    using RecursiveFlavor = bb::MegaRecursiveFlavor_<MegaBuilder>;
    using RecursiveVerifier = bb::stdlib::recursion::honk::UltraRecursiveVerifier_<RecursiveFlavor>;
    using VerifierInstance = bb::VerifierInstance_<Flavor>;
    using RecursiveVerifierInstance = ::bb::stdlib::recursion::honk::RecursiveVerifierInstance_<RecursiveFlavor>;
    using RecursiveVerifierAccumulator = std::shared_ptr<RecursiveVerifierInstance>;
    using VerificationKey = Flavor::VerificationKey;
    static constexpr size_t NUM_OP_QUEUE_COLUMNS = Flavor::NUM_WIRES;

    struct KernelInput {
        HonkProof proof;
        std::shared_ptr<Flavor::VerificationKey> verification_key;
    };

    /**
     * @brief Populate a builder with some arbitrary but nontrivial constraints
     * @details Although the details of the circuit constructed here are arbitrary, the intent is to mock something a
     * bit more realistic than a circuit comprised entirely of arithmetic gates. E.g. the circuit should respond
     * realistically to efforts to parallelize circuit construction.
     *
     * @param builder
     * @param large If true, construct a "large" circuit (2^19), else a medium circuit (2^17)
     */
    static void construct_mock_function_circuit(MegaBuilder& builder, bool large = false)
    {
        // Determine number of times to execute the below operations that constitute the mock circuit logic. Note that
        // the circuit size does not scale linearly with number of iterations due to e.g. amortization of lookup costs
        const size_t NUM_ITERATIONS_LARGE = 12; // results in circuit size 2^19 (502238 gates)

        if (large) {
            stdlib::generate_sha256_test_circuit(builder, NUM_ITERATIONS_LARGE);
            stdlib::generate_ecdsa_verification_test_circuit(builder, NUM_ITERATIONS_LARGE / 2);
            stdlib::generate_merkle_membership_test_circuit(builder, NUM_ITERATIONS_LARGE);
        } else { // Results in circuit size 2^17 when accumulated via ClientIvc
            stdlib::generate_sha256_test_circuit(builder, 5);
            stdlib::generate_ecdsa_verification_test_circuit(builder, 1);
            stdlib::generate_merkle_membership_test_circuit(builder, 10);
        }

        // TODO(https://github.com/AztecProtocol/barretenberg/issues/911): We require goblin ops to be added to the
        // function circuit because we cannot support zero commtiments. While the builder handles this at
        // ProverInstance creation stage via the add_gates_to_ensure_all_polys_are_non_zero function for other MegaHonk
        // circuits (where we don't explicitly need to add goblin ops), in ClientIVC merge proving happens prior to
        // folding where the absense of goblin ecc ops will result in zero commitments.
        MockCircuits::construct_goblin_ecc_op_circuit(builder);
    }

    /**
     * @brief Mock the interactions of a simple curcuit with the op_queue
     * @todo The transcript aggregation protocol in the Goblin proof system can not yet support an empty "previous
     * transcript" (see issue #723) because the corresponding commitments are zero / the point at infinity. This
     * function mocks the interactions with the op queue of a fictional "first" circuit. This way, when we go to
     * generate a proof over our first "real" circuit, the transcript aggregation protocol can proceed nominally.
     * The mock data is valid in the sense that it can be processed by all stages of Goblin as if it came from a
     * genuine circuit.
     *
     *
     * @param op_queue
     */
    static void perform_op_queue_interactions_for_mock_first_circuit(std::shared_ptr<bb::ECCOpQueue>& op_queue)
    {
        bb::MegaCircuitBuilder builder{ op_queue };

        // Add some goblinized ecc ops
        MockCircuits::construct_goblin_ecc_op_circuit(builder);

        op_queue->set_size_data();

        // Manually compute the op queue transcript commitments (which would normally be done by the merge prover)
        bb::srs::init_crs_factory("../srs_db/ignition");
        auto commitment_key = CommitmentKey(op_queue->get_current_size());
        std::array<Point, Flavor::NUM_WIRES> op_queue_commitments;
        size_t idx = 0;
        for (auto& entry : op_queue->get_aggregate_transcript()) {
            op_queue_commitments[idx++] = commitment_key.commit(entry);
        }
        // Store the commitment data for use by the prover of the next circuit
        op_queue->set_commitment_data(op_queue_commitments);
    }

    /**
     * @brief Generate a simple test circuit with some ECC op gates and conventional arithmetic gates
     *
     * @param builder
     */
    static void add_some_ecc_op_gates(MegaBuilder& builder)
    {
        // Add some arbitrary ecc op gates
        for (size_t i = 0; i < 3; ++i) {
            auto point = Point::random_element(&engine);
            auto scalar = FF::random_element(&engine);
            builder.queue_ecc_add_accum(point);
            builder.queue_ecc_mul_accum(point, scalar);
        }
        // queues the result of the preceding ECC
        builder.queue_ecc_eq(); // should be eq and reset
    }

    /**
     * @brief Generate a simple test circuit with some ECC op gates and conventional arithmetic gates
     *
     * @param builder
     */
    static void construct_simple_circuit(MegaBuilder& builder)
    {
        add_some_ecc_op_gates(builder);
        MockCircuits::construct_arithmetic_circuit(builder);
    }

    /**
     * @brief Construct a size 2^17 mock kernel circuit based on vanilla recursion for benchmarking
     * @details This circuit contains (1) some arbitrary operations representing general kernel logic, (2) recursive
     * verification of a function circuit proof, and optionally (3) recursive verification of a previous kernel circuit
     * proof. The arbitrary kernel logic is structured to bring the final dyadic circuit size of the kernel to 2^17.
     *
     * TODO(https://github.com/AztecProtocol/barretenberg/issues/801): Pairing point aggregation not implemented
     * @param builder
     * @param function_accum {proof, vkey} for function circuit to be recursively verified
     * @param prev_kernel_accum {proof, vkey} for previous kernel circuit to be recursively verified
     */
    static void construct_mock_recursion_kernel_circuit(MegaBuilder& builder,
                                                        const KernelInput& function_accum,
                                                        const KernelInput& prev_kernel_accum)
    {
        // Add operations representing general kernel logic e.g. state updates. Note: these are structured to make the
        // kernel "full" within the dyadic size 2^17 (130914 gates)
        const size_t NUM_MERKLE_CHECKS = 40;
        const size_t NUM_ECDSA_VERIFICATIONS = 1;
        const size_t NUM_SHA_HASHES = 1;
        stdlib::generate_merkle_membership_test_circuit(builder, NUM_MERKLE_CHECKS);
        stdlib::generate_ecdsa_verification_test_circuit(builder, NUM_ECDSA_VERIFICATIONS);
        stdlib::generate_sha256_test_circuit(builder, NUM_SHA_HASHES);

        // Execute recursive aggregation of function proof
        RecursiveVerifier verifier1{ &builder, function_accum.verification_key };
        verifier1.verify_proof(function_accum.proof);

        // Execute recursive aggregation of previous kernel proof if one exists
        if (!prev_kernel_accum.proof.empty()) {
            RecursiveVerifier verifier2{ &builder, prev_kernel_accum.verification_key };
            verifier2.verify_proof(prev_kernel_accum.proof);
        }
    }

    /**
     * @brief Construct a mock kernel circuit
     * @details Construct an arbitrary circuit meant to represent the aztec private function execution kernel. Recursive
     * folding verification is handled internally by ClientIvc, not in the kernel.
     *
     * @param builder
     * @param function_fold_proof
     * @param kernel_fold_proof
     */
    static void construct_mock_folding_kernel(MegaBuilder& builder)
    {
        // Add operations representing general kernel logic e.g. state updates. Note: these are structured to make
        // the kernel "full" within the dyadic size 2^17
        const size_t NUM_MERKLE_CHECKS = 20;
        const size_t NUM_ECDSA_VERIFICATIONS = 1;
        const size_t NUM_SHA_HASHES = 1;
        stdlib::generate_merkle_membership_test_circuit(builder, NUM_MERKLE_CHECKS);
        stdlib::generate_ecdsa_verification_test_circuit(builder, NUM_ECDSA_VERIFICATIONS);
        stdlib::generate_sha256_test_circuit(builder, NUM_SHA_HASHES);
    }

    /**
     * @brief A minimal version of the mock kernel (recursive verifiers only) for faster testing
     *
     */
    static void construct_mock_kernel_small(MegaBuilder& builder,
                                            const KernelInput& function_accum,
                                            const KernelInput& prev_kernel_accum)
    {
        // Execute recursive aggregation of function proof
        RecursiveVerifier verifier1{ &builder, function_accum.verification_key };
        verifier1.verify_proof(function_accum.proof);

        // Execute recursive aggregation of previous kernel proof if one exists
        if (!prev_kernel_accum.proof.empty()) {
            RecursiveVerifier verifier2{ &builder, prev_kernel_accum.verification_key };
            verifier2.verify_proof(prev_kernel_accum.proof);
        }
    }
};
} // namespace bb
