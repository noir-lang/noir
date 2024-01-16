#include "poseidon2.hpp"
#include "barretenberg/crypto/poseidon2/poseidon2_params.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include <gtest/gtest.h>

using namespace bb;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace poseidon2_tests {
TEST(Poseidon2, HashBasicTests)
{

    bb::fr a = bb::fr::random_element(&engine);
    bb::fr b = bb::fr::random_element(&engine);
    bb::fr c = bb::fr::random_element(&engine);
    bb::fr d = bb::fr::random_element(&engine);

    std::vector<bb::fr> input1{ a, b, c, d };
    std::vector<bb::fr> input2{ d, c, b, a };

    auto r0 = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input1);
    auto r1 = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input1);
    auto r2 = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input2);

    EXPECT_EQ(r0, r1);
    EXPECT_NE(r0, r2);
}

// N.B. these hardcoded values were extracted from the algorithm being tested. These are NOT independent test vectors!
// TODO(@zac-williamson #3132): find independent test vectors we can compare against! (very hard to find given
// flexibility of Poseidon's parametrisation)
TEST(Poseidon2, HashConsistencyCheck)
{
    bb::fr a(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));
    bb::fr b(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));
    bb::fr c(std::string("0x9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));
    bb::fr d(std::string("0x9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));

    std::vector<bb::fr> input{ a, b, c, d };
    auto result = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input);

    bb::fr expected(std::string("0x2f43a0f83b51a6f5fc839dea0ecec74947637802a579fa9841930a25a0bcec11"));

    EXPECT_EQ(result, expected);
}

TEST(Poseidon2, HashBufferConsistencyCheck)
{
    // 31 byte inputs because hash_buffer slicing is only injective with 31 bytes, as it slices 31 bytes for each field
    // element
    bb::fr a(std::string("00000b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));

    auto input_vec = to_buffer(a);      // takes field element and converts it to 32 bytes
    input_vec.erase(input_vec.begin()); // erase first byte since we want 31 bytes
    std::vector<bb::fr> input{ a };
    auto expected = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input);

    bb::fr result = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash_buffer(input_vec);

    EXPECT_EQ(result, expected);
}
} // namespace poseidon2_tests