#include "pedersen.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/crypto/pedersen_commitment/c_bind.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include <gtest/gtest.h>

namespace bb::crypto {

using bb::fr;

TEST(Pedersen, DeriveLengthGenerator)
{
    auto generator = pedersen_hash::length_generator;
    std::cout << generator << std::endl;
    EXPECT_EQ(generator,
              grumpkin::g1::affine_element(
                  fr(uint256_t("0x2df8b940e5890e4e1377e05373fae69a1d754f6935e6a780b666947431f2cdcd")),
                  fr(uint256_t("0x2ecd88d15967bc53b885912e0d16866154acb6aac2d3f85e27ca7eefb2c19083"))));
}

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