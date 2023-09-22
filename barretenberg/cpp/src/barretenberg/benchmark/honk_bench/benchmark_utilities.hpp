#include <benchmark/benchmark.h>

#include "barretenberg/honk/composer/ultra_composer.hpp"
#include "barretenberg/proof_system/types/circuit_type.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/hash/keccak/keccak.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_store.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/merkle_tree.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/curves/secp256k1.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"

using namespace benchmark;

namespace bench_utils {

struct BenchParams {
    // Num iterations of the operation of interest in a test circuit, e.g. num sha256 hashes
    static constexpr size_t MIN_NUM_ITERATIONS = 10;
    static constexpr size_t MAX_NUM_ITERATIONS = 10;

    // Log num gates; for simple circuits only, e.g. standard arithmetic circuit
    static constexpr size_t MIN_LOG_NUM_GATES = 16;
    static constexpr size_t MAX_LOG_NUM_GATES = 16;

    static constexpr size_t NUM_REPETITIONS = 1;
};

/**
 * @brief Generate test circuit with basic arithmetic operations
 *
 * @param composer
 * @param num_iterations
 */
template <typename Builder> void generate_basic_arithmetic_circuit(Builder& builder, size_t num_gates)
{
    proof_system::plonk::stdlib::field_t a(
        proof_system::plonk::stdlib::witness_t(&builder, barretenberg::fr::random_element()));
    proof_system::plonk::stdlib::field_t b(
        proof_system::plonk::stdlib::witness_t(&builder, barretenberg::fr::random_element()));
    proof_system::plonk::stdlib::field_t c(&builder);
    for (size_t i = 0; i < (num_gates / 4) - 4; ++i) {
        c = a + b;
        c = a * c;
        a = b * b;
        b = c * c;
    }
}

/**
 * @brief Generate test circuit with specified number of sha256 hashes
 *
 * @param builder
 * @param num_iterations
 */
template <typename Builder> void generate_sha256_test_circuit(Builder& builder, size_t num_iterations)
{
    std::string in;
    in.resize(32);
    for (size_t i = 0; i < 32; ++i) {
        in[i] = 0;
    }
    proof_system::plonk::stdlib::packed_byte_array<Builder> input(&builder, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = proof_system::plonk::stdlib::sha256<Builder>(input);
    }
}

/**
 * @brief Generate test circuit with specified number of keccak hashes
 *
 * @param builder
 * @param num_iterations
 */
template <typename Builder> void generate_keccak_test_circuit(Builder& builder, size_t num_iterations)
{
    std::string in = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";

    proof_system::plonk::stdlib::byte_array<Builder> input(&builder, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = proof_system::plonk::stdlib::keccak<Builder>::hash(input);
    }
}

/**
 * @brief Generate test circuit with specified number of ecdsa verifications
 *
 * @param builder
 * @param num_iterations
 */
template <typename Builder> void generate_ecdsa_verification_test_circuit(Builder& builder, size_t num_iterations)
{
    using curve = proof_system::plonk::stdlib::secp256k1<Builder>;
    using fr = typename curve::fr;
    using fq = typename curve::fq;
    using g1 = typename curve::g1;

    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa::key_pair<fr, g1> account;
    for (size_t i = 0; i < num_iterations; i++) {
        // Generate unique signature for each iteration
        account.private_key = curve::fr::random_element();
        account.public_key = curve::g1::one * account.private_key;

        crypto::ecdsa::signature signature =
            crypto::ecdsa::construct_signature<Sha256Hasher, fq, fr, g1>(message_string, account);

        bool first_result =
            crypto::ecdsa::verify_signature<Sha256Hasher, fq, fr, g1>(message_string, account.public_key, signature);
        static_cast<void>(first_result); // TODO(Cody): This is not used anywhere.

        std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
        std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
        uint8_t vv = signature.v;

        typename curve::g1_bigfr_ct public_key = curve::g1_bigfr_ct::from_witness(&builder, account.public_key);

        proof_system::plonk::stdlib::ecdsa::signature<Builder> sig{ typename curve::byte_array_ct(&builder, rr),
                                                                    typename curve::byte_array_ct(&builder, ss),
                                                                    proof_system::plonk::stdlib::uint8<Builder>(
                                                                        &builder, vv) };

        typename curve::byte_array_ct message(&builder, message_string);

        // Verify ecdsa signature
        proof_system::plonk::stdlib::ecdsa::verify_signature<Builder,
                                                             curve,
                                                             typename curve::fq_ct,
                                                             typename curve::bigfr_ct,
                                                             typename curve::g1_bigfr_ct>(message, public_key, sig);
    }
}

/**
 * @brief Generate test circuit with specified number of merkle membership checks
 *
 * @param builder
 * @param num_iterations
 */
template <typename Builder> void generate_merkle_membership_test_circuit(Builder& builder, size_t num_iterations)
{
    using namespace proof_system::plonk::stdlib;
    using field_ct = field_t<Builder>;
    using witness_ct = witness_t<Builder>;
    using witness_ct = witness_t<Builder>;
    using MemStore = merkle_tree::MemoryStore;
    using MerkleTree_ct = merkle_tree::MerkleTree<MemStore>;

    MemStore store;
    const size_t tree_depth = 7;
    auto merkle_tree = MerkleTree_ct(store, tree_depth);

    for (size_t i = 0; i < num_iterations; i++) {
        // For each iteration update and check the membership of a different value
        size_t idx = i;
        size_t value = i * 2;
        merkle_tree.update_element(idx, value);

        field_ct root_ct = witness_ct(&builder, merkle_tree.root());
        auto idx_ct = field_ct(witness_ct(&builder, fr(idx))).decompose_into_bits();
        auto value_ct = field_ct(value);

        merkle_tree::check_membership(
            root_ct, merkle_tree::create_witness_hash_path(builder, merkle_tree.get_hash_path(idx)), value_ct, idx_ct);
    }
}

/**
 * @brief Performs proof constuction for benchmarks based on a provided circuit function
 *
 * @details This function assumes state.range refers to num_gates which is the size of the underlying circuit
 *
 * @tparam Builder
 * @param state
 * @param test_circuit_function
 */
template <typename Composer>
void construct_proof_with_specified_num_gates(State& state,
                                              void (*test_circuit_function)(typename Composer::CircuitBuilder&,
                                                                            size_t)) noexcept
{
    barretenberg::srs::init_crs_factory("../srs_db/ignition");
    auto num_gates = static_cast<size_t>(1 << (size_t)state.range(0));
    for (auto _ : state) {
        // Constuct circuit and prover; don't include this part in measurement
        state.PauseTiming();
        auto builder = typename Composer::CircuitBuilder();
        test_circuit_function(builder, num_gates);

        auto composer = Composer();
        auto ext_prover = composer.create_prover(builder);
        state.ResumeTiming();

        // Construct proof
        auto proof = ext_prover.construct_proof();
    }
}

/**
 * @brief Performs proof constuction for benchmarks based on a provided circuit function
 *
 * @details This function assumes state.range refers to num_iterations which is the number of times to perform a given
 * basic operation in the circuit, e.g. number of hashes
 *
 * @tparam Builder
 * @param state
 * @param test_circuit_function
 */
template <typename Composer>
void construct_proof_with_specified_num_iterations(State& state,
                                                   void (*test_circuit_function)(typename Composer::CircuitBuilder&,
                                                                                 size_t)) noexcept
{
    barretenberg::srs::init_crs_factory("../srs_db/ignition");
    auto num_iterations = static_cast<size_t>(state.range(0));
    for (auto _ : state) {
        // Constuct circuit and prover; don't include this part in measurement
        state.PauseTiming();
        auto builder = typename Composer::CircuitBuilder();
        test_circuit_function(builder, num_iterations);

        auto composer = Composer();
        if constexpr (proof_system::IsAnyOf<Composer, proof_system::honk::UltraComposer>) {
            auto instance = composer.create_instance(builder);
            auto ext_prover = composer.create_prover(instance);
            state.ResumeTiming();

            // Construct proof
            auto proof = ext_prover.construct_proof();

        } else {
            auto ext_prover = composer.create_prover(builder);
            state.ResumeTiming();

            // Construct proof
            auto proof = ext_prover.construct_proof();
        }
    }
}

} // namespace bench_utils