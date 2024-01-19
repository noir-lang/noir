#pragma once
#include <benchmark/benchmark.h>
#include <cstddef>

#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/plonk/composer/ultra_composer.hpp"
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
#include "barretenberg/ultra_honk/ultra_composer.hpp"
#include "barretenberg/ultra_honk/ultra_prover.hpp"

using namespace benchmark;

namespace bench_utils {

/**
 * @brief Generate test circuit with basic arithmetic operations
 *
 * @param composer
 * @param num_iterations
 */
template <typename Builder> void generate_basic_arithmetic_circuit(Builder& builder, size_t log2_num_gates)
{
    bb::stdlib::field_t a(bb::stdlib::witness_t(&builder, bb::fr::random_element()));
    bb::stdlib::field_t b(bb::stdlib::witness_t(&builder, bb::fr::random_element()));
    bb::stdlib::field_t c(&builder);
    size_t passes = (1UL << log2_num_gates) / 4 - 4;
    if (static_cast<int>(passes) <= 0) {
        throw std::runtime_error("too few gates");
    }

    for (size_t i = 0; i < passes; ++i) {
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
    bb::stdlib::packed_byte_array<Builder> input(&builder, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = bb::stdlib::sha256<Builder>(input);
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

    bb::stdlib::byte_array<Builder> input(&builder, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = bb::stdlib::keccak<Builder>::hash(input);
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
    using curve = bb::stdlib::secp256k1<Builder>;
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

        bb::stdlib::ecdsa::signature<Builder> sig{ typename curve::byte_array_ct(&builder, rr),
                                                   typename curve::byte_array_ct(&builder, ss),
                                                   bb::stdlib::uint8<Builder>(&builder, vv) };

        typename curve::byte_array_ct message(&builder, message_string);

        // Verify ecdsa signature
        bb::stdlib::ecdsa::verify_signature<Builder,
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
    using namespace bb::stdlib;
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

// ultrahonk
inline bb::honk::UltraProver get_prover(bb::honk::UltraComposer& composer,
                                        void (*test_circuit_function)(bb::honk::UltraComposer::CircuitBuilder&, size_t),
                                        size_t num_iterations)
{
    bb::honk::UltraComposer::CircuitBuilder builder;
    test_circuit_function(builder, num_iterations);
    std::shared_ptr<bb::honk::UltraComposer::Instance> instance = composer.create_instance(builder);
    return composer.create_prover(instance);
}

// standard plonk
inline bb::plonk::Prover get_prover(bb::plonk::StandardComposer& composer,
                                    void (*test_circuit_function)(bb::StandardCircuitBuilder&, size_t),
                                    size_t num_iterations)
{
    bb::StandardCircuitBuilder builder;
    test_circuit_function(builder, num_iterations);
    return composer.create_prover(builder);
}

// ultraplonk
inline bb::plonk::UltraProver get_prover(bb::plonk::UltraComposer& composer,
                                         void (*test_circuit_function)(bb::honk::UltraComposer::CircuitBuilder&,
                                                                       size_t),
                                         size_t num_iterations)
{
    bb::plonk::UltraComposer::CircuitBuilder builder;
    test_circuit_function(builder, num_iterations);
    return composer.create_prover(builder);
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
void construct_proof_with_specified_num_iterations(
    State& state, void (*test_circuit_function)(typename Composer::CircuitBuilder&, size_t), size_t num_iterations)
{
    bb::srs::init_crs_factory("../srs_db/ignition");

    Composer composer;

    for (auto _ : state) {
        // Construct circuit and prover; don't include this part in measurement
        state.PauseTiming();
        auto prover = get_prover(composer, test_circuit_function, num_iterations);
        state.ResumeTiming();

        // Construct proof
        auto proof = prover.construct_proof();
    }
}

} // namespace bench_utils
