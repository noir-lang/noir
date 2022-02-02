#include "ratio_check.hpp"
#include <common/test.hpp>
#include <numeric/random/engine.hpp>

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;
using namespace rollup::proofs::claim;

namespace {
auto& engine = numeric::random::get_debug_engine();
} // namespace

// Testing a1 * b1 == a2 * b2 passes for valid ratios.
TEST(ratio_check, product_check)
{
    uint256_t a1 = engine.get_random_uint256();
    a1.data[3] = a1.data[3] & 0x0fffffffffffffffULL; // 60-bits

    uint256_t b1 = engine.get_random_uint256();
    b1.data[3] = b1.data[3] & 0x0fffffffffffffffULL; // 60-bits
    b1.data[0] = b1.data[0] & 0xfffffffffffffffeULL; // 64-bits (lsb zero)

    // Halve & double to retain same ratio on RHS.
    uint256_t a2 = a1 << 1;
    uint256_t b2 = b1 >> 1;

    uint512_t test_left = uint512_t(a1) * uint512_t(b1);
    uint512_t test_right = uint512_t(a2) * uint512_t(b2);
    EXPECT_EQ(test_left, test_right);

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct left1(witness_ct(&composer, a1));
    field_ct right1(witness_ct(&composer, b1));
    field_ct left2(witness_ct(&composer, a2));
    field_ct right2(witness_ct(&composer, b2));

    auto result = product_check(composer, left1, right1, left2, right2, witness_ct(&composer, 0));
    result.assert_equal(true);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

// Testing a1 * b1 == a2 * b2 passes with a zero term on each side.
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
    a.data[3] = a.data[3] & 0x0fffffffffffffffULL; // 60-bits
    uint256_t b = engine.get_random_uint256();
    b.data[3] = b.data[3] & 0x0fffffffffffffffULL; // 60-bits
    uint256_t c;
    while (c == 0) {
        c = engine.get_random_uint256(); // it'll 'never' happen, but just in case it's 0, try again.
    }
    c.data[3] = c.data[3] & 0x0fffffffffffffffULL; // 60-bits

    // Notice: if b > c, then (b/c) > 1, so in the equation below, a * (b / c) can overflow 256-bits if `a` is
    // sufficiently big.
    // This check is done within the circuit by checking total_in >= user_in.
    if (b > c) {
        std::swap(b, c);
    }

    const uint256_t d = ((uint512_t(a) * uint512_t(b)) / uint512_t(c)).lo;

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct a1(witness_ct(&composer, a));
    field_ct a2(witness_ct(&composer, c));
    field_ct b1(witness_ct(&composer, d));
    field_ct b2(witness_ct(&composer, b));

    // Above, we calculated d = (a * b) / c.
    // Accounting for the renamings, that is, b1 = (a1 * b2) / a2.
    // We want to check that a1 * b2 == b1 * a2.
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

TEST(ratio_check, zero_denominator_a2_returns_false)
{
    uint256_t a = 10;
    uint256_t b = 1;
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

TEST(ratio_check, zero_denominator_b2_returns_false)
{
    uint256_t a = 10;
    uint256_t b = 0;
    uint256_t c = 5;
    uint256_t d = 1;

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

TEST(ratio_check, zero_denominator_both_returns_false)
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

TEST(ratio_check, field_modulus_overflow_fails)
{
    uint256_t a = 1;
    uint256_t b = 1;
    uint256_t c = 2;
    // uint256_t d = 10944121435919637611123202872628637544274182200208017171849102093287904247809; // = 2^(-1)
    uint256_t d(0xA1F0FAC9F8000001ULL, 0x9419F4243CDCB848ULL, 0xDC2822DB40C0AC2EULL, 0x183227397098D014ULL); // = 2^(-1)

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct a1(witness_ct(&composer, a));
    field_ct a2(witness_ct(&composer, d));
    field_ct b1(witness_ct(&composer, c));
    field_ct b2(witness_ct(&composer, b));

    // We want to check that a  * b  == c  * d.
    // Or, renamed:          a1 * b2 == b1 * a2.
    ratios ratios{ a1, a2, b1, b2 };
    auto result = ratio_check(composer, ratios);
    result.assert_equal(false);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}

TEST(ratio_check, field_modulus_overflow_with_biggest_numbers_possible_fails)
{
    // field modulus
    uint256_t r(0x43E1F593F0000001ULL, 0x2833E84879B97091ULL, 0xB85045B68181585DULL, 0x30644E72E131A029ULL);

    uint256_t a = 1;
    uint256_t b = 1;
    uint256_t c = r - 1;
    uint256_t d = r - 1;

    waffle::TurboComposer composer = waffle::TurboComposer();

    field_ct a1(witness_ct(&composer, a));
    field_ct a2(witness_ct(&composer, d));
    field_ct b1(witness_ct(&composer, c));
    field_ct b2(witness_ct(&composer, b));

    // We want to check that a  * b  == c  * d.
    // Or, renamed:          a1 * b2 == b1 * a2.
    ratios ratios{ a1, a2, b1, b2 };
    auto result = ratio_check(composer, ratios);
    result.assert_equal(false);

    waffle::TurboProver prover = composer.create_prover();
    waffle::TurboVerifier verifier = composer.create_verifier();
    waffle::plonk_proof proof = prover.construct_proof();
    bool proof_result = verifier.verify_proof(proof);
    EXPECT_EQ(proof_result, true);
}