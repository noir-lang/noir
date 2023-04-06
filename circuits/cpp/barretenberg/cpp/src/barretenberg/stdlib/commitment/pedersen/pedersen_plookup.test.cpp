#include "pedersen.hpp"
#include "pedersen_plookup.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen.hpp"
#include "barretenberg/crypto/pedersen_commitment/pedersen_lookup.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen_lookup.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include "barretenberg/common/test.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"

namespace test_stdlib_pedersen {
using namespace barretenberg;
using namespace proof_system::plonk;
namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace plookup_pedersen_tests {
typedef stdlib::field_t<UltraComposer> field_ct;
typedef stdlib::witness_t<UltraComposer> witness_ct;
TEST(stdlib_pedersen, test_pedersen_plookup)
{
    UltraComposer composer = UltraComposer();

    fr left_in = fr::random_element();
    fr right_in = fr::random_element();

    field_ct left = witness_ct(&composer, left_in);
    field_ct right = witness_ct(&composer, right_in);

    field_ct result = stdlib::pedersen_plookup_commitment<UltraComposer>::compress(left, right);

    fr expected = crypto::pedersen_hash::lookup::hash_pair(left_in, right_in);

    EXPECT_EQ(result.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_compress_many_plookup)
{
    UltraComposer composer = UltraComposer();

    std::vector<fr> input_values{
        fr::random_element(), fr::random_element(), fr::random_element(),
        fr::random_element(), fr::random_element(), fr::random_element(),
    };
    std::vector<field_ct> inputs;
    for (const auto& input : input_values) {
        inputs.emplace_back(witness_ct(&composer, input));
    }

    const size_t hash_idx = 20;

    field_ct result = stdlib::pedersen_plookup_commitment<UltraComposer>::compress(inputs, hash_idx);

    auto expected = crypto::pedersen_commitment::lookup::compress_native(input_values, hash_idx);

    EXPECT_EQ(result.get_value(), expected);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_merkle_damgard_compress_plookup)
{
    UltraComposer composer = UltraComposer();

    std::vector<fr> input_values{
        fr::random_element(), fr::random_element(), fr::random_element(),
        fr::random_element(), fr::random_element(), fr::random_element(),
    };
    std::vector<field_ct> inputs;
    for (const auto& input : input_values) {
        inputs.emplace_back(witness_ct(&composer, input));
    }
    field_ct iv = witness_ct(&composer, fr(10));

    field_ct result = stdlib::pedersen_plookup_commitment<UltraComposer>::merkle_damgard_compress(inputs, iv).x;

    auto expected = crypto::pedersen_commitment::lookup::merkle_damgard_compress(input_values, 10);

    EXPECT_EQ(result.get_value(), expected.normalize().x);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_merkle_damgard_compress_multiple_iv_plookup)
{
    UltraComposer composer = UltraComposer();

    const size_t m = 10;
    std::vector<fr> input_values;
    std::vector<size_t> iv_values;
    for (size_t i = 0; i < m; i++) {
        input_values.push_back(fr::random_element());
        iv_values.push_back(engine.get_random_uint8());
    }

    std::vector<field_ct> inputs;
    std::vector<field_ct> ivs;
    for (size_t i = 0; i < m; i++) {
        inputs.emplace_back(witness_ct(&composer, input_values[i]));
        ivs.emplace_back(witness_ct(&composer, fr(iv_values[i])));
    }

    field_ct result = stdlib::pedersen_plookup_commitment<UltraComposer>::merkle_damgard_compress(inputs, ivs).x;

    auto expected = crypto::pedersen_commitment::lookup::merkle_damgard_compress(input_values, iv_values);

    EXPECT_EQ(result.get_value(), expected.normalize().x);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_pedersen, test_merkle_damgard_tree_compress_plookup)
{
    UltraComposer composer = UltraComposer();

    const size_t m = 16;
    std::vector<fr> input_values;
    std::vector<size_t> iv_values;
    for (size_t i = 0; i < m; i++) {
        input_values.push_back(fr::random_element());
        iv_values.push_back(engine.get_random_uint8());
    }

    std::vector<field_ct> inputs;
    std::vector<field_ct> ivs;
    for (size_t i = 0; i < m; i++) {
        inputs.emplace_back(witness_ct(&composer, input_values[i]));
        ivs.emplace_back(witness_ct(&composer, fr(iv_values[i])));
    }

    field_ct result = stdlib::pedersen_plookup_commitment<UltraComposer>::merkle_damgard_tree_compress(inputs, ivs).x;

    auto expected = crypto::pedersen_commitment::lookup::merkle_damgard_tree_compress(input_values, iv_values);

    EXPECT_EQ(result.get_value(), expected.normalize().x);

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

} // namespace plookup_pedersen_tests
} // namespace test_stdlib_pedersen
