#include "pedersen.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include <gtest/gtest.h>

namespace bb::crypto {

using bb::fr;

TEST(Pedersen, Hash)
{
    auto x = pedersen_hash::Fq::one();
    auto r = pedersen_hash::hash({ x, x });
    EXPECT_EQ(r, fr(uint256_t("07ebfbf4df29888c6cd6dca13d4bb9d1a923013ddbbcbdc3378ab8845463297b")));
}

TEST(Pedersen, HashWithIndex)
{
    auto x = pedersen_hash::Fq::one();
    auto r = pedersen_hash::hash({ x, x }, 5);
    EXPECT_EQ(r, fr(uint256_t("1c446df60816b897cda124524e6b03f36df0cec333fad87617aab70d7861daa6")));
}

} // namespace bb::crypto