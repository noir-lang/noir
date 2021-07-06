#include "ratio_check.hpp"
#include <common/test.hpp>
#include <numeric/random/engine.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs::claim;

namespace {
auto& engine = numeric::random::get_debug_engine();
} // namespace

TEST(ratio_check, product_check)
{
    uint256_t a0 = engine.get_random_uint256();
    a0.data[3] = a0.data[3] & 0x0fffffffffffffffULL;

    uint256_t b0 = engine.get_random_uint256();
    b0.data[3] = b0.data[3] & 0x0fffffffffffffffULL;
    b0.data[0] = b0.data[0] & 0xfffffffffffffffeULL;

    uint256_t a1 = a0 << 1;
    uint256_t b1 = b0 >> 1;

    uint512_t test_left = uint512_t(a0) * uint512_t(b0);
    uint512_t test_right = uint512_t(a1) * uint512_t(b1);
    EXPECT_EQ(test_left, test_right);

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct left1(witness_ct(&composer, a0));
    field_ct right1(witness_ct(&composer, b0));
    field_ct left2(witness_ct(&composer, a1));
    field_ct right2(witness_ct(&composer, b1));

    auto result = product_check(composer, left1, right1, left2, right2, witness_ct(&composer, 0));
    result.assert_equal(true);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(ratio_check, product_check_with_zeros)
{
    uint256_t a = 10;
    uint256_t b = 0;
    uint256_t c = 5;
    uint256_t d = 0;

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct a1(witness_ct(&composer, a));
    field_ct b1(witness_ct(&composer, b));
    field_ct a2(witness_ct(&composer, c));
    field_ct b2(witness_ct(&composer, d));

    auto result = product_check(composer, a1, b1, a2, b2, witness_ct(&composer, 0));
    result.assert_equal(true);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(ratio_check, ratio_check)
{
    uint256_t a = engine.get_random_uint256();
    a.data[3] = a.data[3] & 0x0fffffffffffffffULL;
    uint256_t b = engine.get_random_uint256();
    b.data[3] = b.data[3] & 0x0fffffffffffffffULL;
    uint256_t c = engine.get_random_uint256();
    c.data[3] = c.data[3] & 0x0fffffffffffffffULL;

    // TODO: check total_in >= user_in! Does not work otherwise because we get integer overflow
    if (b > c) {
        std::swap(b, c);
    }

    const uint256_t d = ((uint512_t(a) * uint512_t(b)) / uint512_t(c)).lo;

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct a1(witness_ct(&composer, a));
    field_ct a2(witness_ct(&composer, c));
    field_ct b1(witness_ct(&composer, d));
    field_ct b2(witness_ct(&composer, b));

    ratios ratios{ a1, a2, b1, b2 };
    auto result = ratio_check(composer, ratios);
    result.assert_equal(true);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(ratio_check, bad_ratio_check)
{
    uint256_t a = 100;
    uint256_t b = 10;
    uint256_t c = 200;
    uint256_t d = 21;

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct a1(witness_ct(&composer, a));
    field_ct a2(witness_ct(&composer, b));
    field_ct b1(witness_ct(&composer, c));
    field_ct b2(witness_ct(&composer, d));

    ratios ratios{ a1, a2, b1, b2 };
    auto result = ratio_check(composer, ratios);
    result.assert_equal(false);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(ratio_check, zero_denominator_returns_false)
{
    uint256_t a = 10;
    uint256_t b = 0;
    uint256_t c = 5;
    uint256_t d = 0;

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct a1(witness_ct(&composer, a));
    field_ct a2(witness_ct(&composer, d));
    field_ct b1(witness_ct(&composer, c));
    field_ct b2(witness_ct(&composer, b));

    ratios ratios{ a1, a2, b1, b2 };
    auto result = ratio_check(composer, ratios);
    result.assert_equal(false);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}