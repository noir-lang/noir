#include "pedersen.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include <gtest/gtest.h>

namespace crypto {

using barretenberg::fr;

TEST(Pedersen, Commitment)
{
    auto x = pedersen_commitment::Fq::one();
    auto r = pedersen_commitment::commit_native({ x, x });
    auto expected =
        grumpkin::g1::affine_element(fr(uint256_t("2f7a8f9a6c96926682205fb73ee43215bf13523c19d7afe36f12760266cdfe15")),
                                     fr(uint256_t("01916b316adbbf0e10e39b18c1d24b33ec84b46daddf72f43878bcc92b6057e6")));
    EXPECT_EQ(r, expected);
}

} // namespace crypto