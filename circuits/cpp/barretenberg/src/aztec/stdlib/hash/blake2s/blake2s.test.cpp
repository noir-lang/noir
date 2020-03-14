#include "blake2s.hpp"
#include <crypto/blake2s/blake2s.hpp>
#include <gtest/gtest.h>
#include <plonk/composer/turbo_composer.hpp>

using namespace barretenberg;
using namespace plonk;

typedef waffle::TurboComposer Composer;
typedef stdlib::byte_array<Composer> byte_array;
typedef stdlib::public_witness_t<Composer> public_witness_t;

namespace std {
inline std::ostream& operator<<(std::ostream& os, std::vector<uint8_t> const& t)
{
    os << "[ ";
    for (auto e : t) {
        os << std::setfill('0') << std::hex << std::setw(2) << (int)e << " ";
    }
    os << "]";
    return os;
}
} // namespace std

TEST(stdlib_blake2s, test_single_block)
{
    Composer composer = Composer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz01";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&composer, input_v);
    byte_array output = stdlib::blake2s(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), std::string(expected.begin(), expected.end()));

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(stdlib_blake2s, test_double_block)
{
    Composer composer = Composer();
    std::string input = "abcdefghijklmnopqrstuvwxyz0123456789abcdefghijklmnopqrstuvwxyz0123456789";
    std::vector<uint8_t> input_v(input.begin(), input.end());

    byte_array input_arr(&composer, input_v);
    byte_array output = stdlib::blake2s(input_arr);

    std::vector<uint8_t> expected = blake2::blake2s(input_v);

    EXPECT_EQ(output.get_value(), std::string(expected.begin(), expected.end()));

    auto prover = composer.create_prover();

    printf("composer gates = %zu\n", composer.get_num_gates());
    auto verifier = composer.create_verifier();

    auto proof = prover.construct_proof();

    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}
