#include "barretenberg/crypto/ecdsa/ecdsa.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include <benchmark/benchmark.h>
#include <cstddef>
#include "barretenberg/stdlib/primitives/composers/composers_fwd.hpp"
#include "barretenberg/stdlib/primitives/composers/composers.hpp"
#include "barretenberg/stdlib/encryption/ecdsa/ecdsa.hpp"
#include "barretenberg/stdlib/hash/keccak/keccak.hpp"
#include "barretenberg/stdlib/primitives/curves/secp256k1.hpp"
#include "barretenberg/stdlib/primitives/packed_byte_array/packed_byte_array.hpp"
#include "barretenberg/stdlib/hash/sha256/sha256.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/witness/witness.hpp"
#include "barretenberg/stdlib/merkle_tree/merkle_tree.hpp"
#include "barretenberg/stdlib/merkle_tree/membership.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_store.hpp"
#include "barretenberg/stdlib/merkle_tree/memory_tree.hpp"

using namespace benchmark;

namespace ultra_honk_bench {

using Composer = proof_system::honk::UltraHonkComposer;

// Number of times to perform operation of interest in the benchmark circuits, e.g. # of hashes to perform
constexpr size_t MIN_NUM_ITERATIONS = 10;
constexpr size_t MAX_NUM_ITERATIONS = 10;
// Number of times to repeat each benchmark
constexpr size_t NUM_REPETITIONS = 1;

/**
 * @brief Generate test circuit with specified number of sha256 hashes
 *
 * @param composer
 * @param num_iterations
 */
void generate_sha256_test_circuit(Composer& composer, size_t num_iterations)
{
    std::string in;
    in.resize(32);
    for (size_t i = 0; i < 32; ++i) {
        in[i] = 0;
    }
    proof_system::plonk::stdlib::packed_byte_array<Composer> input(&composer, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = proof_system::plonk::stdlib::sha256<Composer>(input);
    }
}

/**
 * @brief Generate test circuit with specified number of keccak hashes
 *
 * @param composer
 * @param num_iterations
 */
void generate_keccak_test_circuit(Composer& composer, size_t num_iterations)
{
    std::string in = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";

    proof_system::plonk::stdlib::byte_array<Composer> input(&composer, in);
    for (size_t i = 0; i < num_iterations; i++) {
        input = proof_system::plonk::stdlib::keccak<Composer>::hash(input);
    }
}

/**
 * @brief Generate test circuit with specified number of ecdsa verifications
 *
 * @param composer
 * @param num_iterations
 */
void generate_ecdsa_verification_test_circuit(Composer& composer, size_t num_iterations)
{
    using curve = proof_system::plonk::stdlib::secp256k1<Composer>;

    std::string message_string = "Instructions unclear, ask again later.";

    crypto::ecdsa::key_pair<curve::fr, curve::g1> account;
    account.private_key = curve::fr::random_element();
    account.public_key = curve::g1::one * account.private_key;

    crypto::ecdsa::signature signature =
        crypto::ecdsa::construct_signature<Sha256Hasher, curve::fq, curve::fr, curve::g1>(message_string, account);

    bool first_result = crypto::ecdsa::verify_signature<Sha256Hasher, curve::fq, curve::fr, curve::g1>(
        message_string, account.public_key, signature);

    std::vector<uint8_t> rr(signature.r.begin(), signature.r.end());
    std::vector<uint8_t> ss(signature.s.begin(), signature.s.end());
    uint8_t vv = signature.v;

    curve::g1_bigfr_ct public_key = curve::g1_bigfr_ct::from_witness(&composer, account.public_key);

    proof_system::plonk::stdlib::ecdsa::signature<Composer> sig{ curve::byte_array_ct(&composer, rr),
                                                                 curve::byte_array_ct(&composer, ss),
                                                                 proof_system::plonk::stdlib::uint8<Composer>(&composer,
                                                                                                              vv) };

    curve::byte_array_ct message(&composer, message_string);

    for (size_t i = 0; i < num_iterations; i++) {
        proof_system::plonk::stdlib::ecdsa::
            verify_signature<Composer, curve, curve::fq_ct, curve::bigfr_ct, curve::g1_bigfr_ct>(
                message, public_key, sig);
    }
}

/**
 * @brief Generate test circuit with specified number of merkle membership checks
 *
 * @param composer
 * @param num_iterations
 * @todo (luke): should we consider deeper tree? non-zero leaf values? variable index?
 */
void generate_merkle_membership_test_circuit(Composer& composer, size_t num_iterations)
{
    using namespace proof_system::plonk::stdlib;
    using field_ct = field_t<Composer>;
    using witness_ct = witness_t<Composer>;
    using witness_ct = witness_t<Composer>;
    using MemStore = merkle_tree::MemoryStore;
    using MerkleTree_ct = merkle_tree::MerkleTree<MemStore>;

    MemStore store;
    auto db = MerkleTree_ct(store, 3);

    // Check that the leaf at index 0 has value 0.
    auto zero = field_ct(witness_ct(&composer, fr::zero())).decompose_into_bits();
    field_ct root = witness_ct(&composer, db.root());

    for (size_t i = 0; i < num_iterations; i++) {
        merkle_tree::check_membership(
            root, merkle_tree::create_witness_hash_path(composer, db.get_hash_path(0)), field_ct(0), zero);
    }
}

/**
 * @brief Benchmark: Construction of a Ultra Honk proof for a circuit determined by the provided text circuit function
 */
void construct_proof_ultra(State& state, void (*test_circuit_function)(Composer&, size_t)) noexcept
{
    auto num_iterations = static_cast<size_t>(state.range(0));
    for (auto _ : state) {
        state.PauseTiming();
        auto composer = Composer();
        test_circuit_function(composer, num_iterations);
        auto ext_prover = composer.create_prover();
        state.ResumeTiming();

        auto proof = ext_prover.construct_proof();
    }
}

BENCHMARK_CAPTURE(construct_proof_ultra, sha256, &generate_sha256_test_circuit)
    ->DenseRange(MIN_NUM_ITERATIONS, MAX_NUM_ITERATIONS)
    ->Repetitions(NUM_REPETITIONS);
BENCHMARK_CAPTURE(construct_proof_ultra, keccak, &generate_keccak_test_circuit)
    ->DenseRange(MIN_NUM_ITERATIONS, MAX_NUM_ITERATIONS)
    ->Repetitions(NUM_REPETITIONS);
BENCHMARK_CAPTURE(construct_proof_ultra, ecdsa_verification, &generate_ecdsa_verification_test_circuit)
    ->DenseRange(MIN_NUM_ITERATIONS, MAX_NUM_ITERATIONS)
    ->Repetitions(NUM_REPETITIONS);
BENCHMARK_CAPTURE(construct_proof_ultra, merkle_membership, &generate_merkle_membership_test_circuit)
    ->DenseRange(MIN_NUM_ITERATIONS, MAX_NUM_ITERATIONS)
    ->Repetitions(NUM_REPETITIONS);

} // namespace ultra_honk_bench