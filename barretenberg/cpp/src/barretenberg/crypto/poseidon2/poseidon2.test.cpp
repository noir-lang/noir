#include "poseidon2.hpp"
#include "barretenberg/crypto/poseidon2/poseidon2_params.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include <gtest/gtest.h>

using namespace barretenberg;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

namespace poseidon2_tests {
TEST(Poseidon2, BasicTests)
{

    barretenberg::fr a = barretenberg::fr::random_element(&engine);
    barretenberg::fr b = barretenberg::fr::random_element(&engine);
    barretenberg::fr c = barretenberg::fr::random_element(&engine);
    barretenberg::fr d = barretenberg::fr::random_element(&engine);

    std::vector<barretenberg::fr> input1{ a, b, c, d };
    std::vector<barretenberg::fr> input2{ d, c, b, a };

    auto r0 = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input1);
    auto r1 = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input1);
    auto r2 = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input2);

    EXPECT_EQ(r0, r1);
    EXPECT_NE(r0, r2);
}

// N.B. these hardcoded values were extracted from the algorithm being tested. These are NOT independent test vectors!
// TODO(@zac-williamson #3132): find independent test vectors we can compare against! (very hard to find given
// flexibility of Poseidon's parametrisation)
TEST(Poseidon2, ConsistencyCheck)
{
    barretenberg::fr a(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));
    barretenberg::fr b(std::string("9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));
    barretenberg::fr c(std::string("0x9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));
    barretenberg::fr d(std::string("0x9a807b615c4d3e2fa0b1c2d3e4f56789fedcba9876543210abcdef0123456789"));

    std::array<barretenberg::fr, 4> input{ a, b, c, d };
    auto result = crypto::Poseidon2<crypto::Poseidon2Bn254ScalarFieldParams>::hash(input);

    barretenberg::fr expected(std::string("0x150c19ae11b3290c137c7a4d760d9482a6581d731535f560c3601d6a766b0937"));

    EXPECT_EQ(result, expected);
}
} // namespace poseidon2_tests