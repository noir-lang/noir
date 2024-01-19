#include "pedersen.hpp"
#include "barretenberg/common/timer.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include <gtest/gtest.h>

namespace bb::crypto {

using bb::fr;

TEST(Pedersen, Commitment)
{
    auto x = pedersen_commitment::Fq::one();
    auto r = pedersen_commitment::commit_native({ x, x });
    auto expected =
        grumpkin::g1::affine_element(fr(uint256_t("2f7a8f9a6c96926682205fb73ee43215bf13523c19d7afe36f12760266cdfe15")),
                                     fr(uint256_t("01916b316adbbf0e10e39b18c1d24b33ec84b46daddf72f43878bcc92b6057e6")));
    EXPECT_EQ(r, expected);
}

TEST(Pedersen, CommitmentWithZero)
{
    auto x = pedersen_commitment::Fq::zero();
    auto y = pedersen_commitment::Fq::one();
    auto r = pedersen_commitment::commit_native({ x, y });
    auto expected =
        grumpkin::g1::affine_element(fr(uint256_t("054aa86a73cb8a34525e5bbed6e43ba1198e860f5f3950268f71df4591bde402")),
                                     fr(uint256_t("209dcfbf2cfb57f9f6046f44d71ac6faf87254afc7407c04eb621a6287cac126")));
    EXPECT_EQ(r, expected);
}

TEST(Pedersen, CommitmentProf)
{
    GTEST_SKIP() << "Skipping mini profiler.";
    auto x = fr::random_element();
    auto y = fr::random_element();
    Timer t;
    for (int i = 0; i < 10000; ++i) {
        pedersen_commitment::commit_native({ x, y });
    }
    info(t.nanoseconds() / 1000 / 10000);
}

// Useful for pasting into ts version of pedersen.
TEST(Pedersen, GeneratorPrinter)
{
    GTEST_SKIP() << "Skipping generator-for-ts printer.";
    pedersen_commitment::GeneratorContext ctx;
    auto generators = ctx.generators->get_default_generators()->get(128);
    for (auto g : generators) {
        info("[", g.x, "n, ", g.y, "n],");
    }
}

}; // namespace bb::crypto