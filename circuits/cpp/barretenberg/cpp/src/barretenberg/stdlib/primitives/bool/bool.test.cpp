#include "bool.hpp"
#include "barretenberg/plonk/proof_system/constants.hpp"
#include <gtest/gtest.h>
#include "barretenberg/plonk/composer/standard_composer.hpp"
#include "barretenberg/honk/composer/standard_honk_composer.hpp"

namespace test_stdlib_bool {
using namespace barretenberg;
using namespace proof_system::plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

typedef stdlib::bool_t<honk::StandardHonkComposer> bool_t;
typedef stdlib::witness_t<honk::StandardHonkComposer> witness_t;

TEST(stdlib_bool, test_basic_operations)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    bool_t a(&composer);
    bool_t b(&composer);
    a = stdlib::witness_t(&composer, barretenberg::fr::one());
    b = stdlib::witness_t(&composer, barretenberg::fr::zero());
    a = a ^ b;           // a = 1
    b = !b;              // b = 1 (witness 0)
    bool_t d = (a == b); //
    d = false;           // d = 0
    bool_t e = a | d;    // e = 1 = a
    bool_t f = e ^ b;    // f = 0
    d = (!f) & a;        // d = 1
    auto prover = composer.create_prover();
    // if constexpr (Composer::type == ComposerType::STANDARD_HONK) {
    EXPECT_EQ(prover.wire_polynomials[0][3], fr(1));
    EXPECT_EQ(prover.wire_polynomials[1][3], fr(1));
    EXPECT_EQ(prover.wire_polynomials[2][3], fr(1));
    EXPECT_EQ(prover.wire_polynomials[0][4], fr(0));
    EXPECT_EQ(prover.wire_polynomials[1][4], fr(0));
    EXPECT_EQ(prover.wire_polynomials[2][4], fr(0));
    EXPECT_EQ(prover.wire_polynomials[0][5], fr(1));
    EXPECT_EQ(prover.wire_polynomials[1][5], fr(0));
    EXPECT_EQ(prover.wire_polynomials[2][5], fr(1));
    EXPECT_EQ(prover.wire_polynomials[0][6], fr(1));
    EXPECT_EQ(prover.wire_polynomials[1][6], fr(0));
    EXPECT_EQ(prover.wire_polynomials[2][6], fr(1));
    EXPECT_EQ(prover.wire_polynomials[0][7], fr(1));
    EXPECT_EQ(prover.wire_polynomials[1][7], fr(0));
    EXPECT_EQ(prover.wire_polynomials[2][7], fr(0));
    EXPECT_EQ(prover.wire_polynomials[0][8], fr(0));
    EXPECT_EQ(prover.wire_polynomials[1][8], fr(1));
    EXPECT_EQ(prover.wire_polynomials[2][8], fr(1));
    // } else {
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_1_lagrange")[1], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_2_lagrange")[1], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_3_lagrange")[1], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_1_lagrange")[2], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_2_lagrange")[2], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_3_lagrange")[2], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_1_lagrange")[3], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_2_lagrange")[3], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_3_lagrange")[3], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_1_lagrange")[4], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_2_lagrange")[4], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_3_lagrange")[4], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_1_lagrange")[5], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_2_lagrange")[5], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_3_lagrange")[5], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_1_lagrange")[6], fr(0));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_2_lagrange")[6], fr(1));
    //     EXPECT_EQ(prover.key->polynomial_store.get("w_3_lagrange")[6], fr(1));
    // }
    EXPECT_EQ(prover.key->circuit_size, 16UL);
}

TEST(stdlib_bool, xor)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t i = 0; i < 4; ++i) {
            bool a_val = (bool)(i % 2);
            bool b_val = (bool)(i > 1 ? true : false);
            bool_t a = lhs_constant ? bool_t(a_val) : (witness_t(&composer, a_val));
            bool_t b = rhs_constant ? bool_t(b_val) : (witness_t(&composer, b_val));
            bool_t c = a ^ b;
            EXPECT_EQ(c.get_value(), a.get_value() ^ b.get_value());
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, xor_constants)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t i = 0; i < 32; ++i) {
        bool_t a = witness_t(&composer, (bool)(i % 2));
        bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
        a ^ b;
    }
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_t a = witness_t(&composer, (bool)(i % 2));
            bool_t b(&composer, (bool)(i % 3 == 1));
            a ^ b;
        } else {
            bool_t a(&composer, (bool)(i % 2));
            bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
            a ^ b;
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, xor_twin_constants)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    bool_t c;
    for (size_t i = 0; i < 32; ++i) {
        bool_t a(&composer, (i % 1) == 0);
        bool_t b(&composer, (i % 1) == 1);
        c = c ^ a ^ b;
    }
    c = c ^ bool_t(witness_t(&composer, true));
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_t a = witness_t(&composer, (bool)(i % 2));
            bool_t b(&composer, (bool)(i % 3 == 1));
            c = c ^ a ^ b;
        } else {
            bool_t a(&composer, (bool)(i % 2));
            bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
            c = c ^ a ^ b;
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

// TEST(stdlib_bool, logical_and)
// {
//     honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
//     bool_t a = witness_t(&composer, 1);
//     bool_t b = witness_t(&composer, 1);
//     (!a) && (!b);

//     auto prover = composer.create_prover();
//     auto verifier = composer.create_verifier();

//     plonk::proof proof = prover.construct_proof();

//     bool result = verifier.verify_proof(proof);
//     EXPECT_EQ(result, true);
// }

TEST(stdlib_bool, and)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t i = 0; i < 32; ++i) {
        bool_t a = witness_t(&composer, (bool)(i % 1));
        bool_t b = witness_t(&composer, (bool)(i % 2 == 1));
        a& b;
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, and_constants)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t i = 0; i < 32; ++i) {
        bool_t a = witness_t(&composer, (bool)(i % 2));
        bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
        a& b;
    }
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_t a = witness_t(&composer, (bool)(i % 2));
            bool_t b(&composer, (bool)(i % 3 == 1));
            a& b;
        } else {
            bool_t a(&composer, (bool)(i % 2));
            bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
            a& b;
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, or)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t i = 0; i < 32; ++i) {
        bool_t a = witness_t(&composer, (bool)(i % 2));
        bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
        a | b;
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, or_constants)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t i = 0; i < 32; ++i) {
        bool_t a = witness_t(&composer, (bool)(i % 2));
        bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
        a | b;
    }
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_t a = witness_t(&composer, (bool)(i % 2));
            bool_t b(&composer, (bool)(i % 3 == 1));
            a | b;
        } else {
            bool_t a(&composer, (bool)(i % 2));
            bool_t b = witness_t(&composer, (bool)(i % 3 == 1));
            a | b;
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, eq)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    bool a_alt[32];
    bool b_alt[32];
    bool c_alt[32];
    bool d_alt[32];
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            a_alt[i] = bool(i % 2);
            b_alt[i] = false;
            c_alt[i] = a_alt[i] ^ b_alt[i];
            d_alt[i] = a_alt[i] == c_alt[i];
        } else {
            a_alt[i] = true;
            b_alt[i] = false;
            c_alt[i] = false;
            d_alt[i] = false;
        }
    }
    bool_t a[32];
    bool_t b[32];
    bool_t c[32];
    bool_t d[32];
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            a[i] = witness_t(&composer, (bool)(i % 2));
            b[i] = witness_t(&composer, (bool)(0));
            c[i] = a[i] ^ b[i];
            d[i] = a[i] == c[i];
        } else {
            a[i] = witness_t(&composer, (bool)(1));
            b[i] = witness_t(&composer, (bool)(0));
            c[i] = a[i] & b[i];
            d[i] = a[i] == c[i];
        }
    }
    for (size_t i = 0; i < 32; ++i) {
        EXPECT_EQ(a[i].get_value(), a_alt[i]);
        EXPECT_EQ(b[i].get_value(), b_alt[i]);
        EXPECT_EQ(c[i].get_value(), c_alt[i]);
        EXPECT_EQ(d[i].get_value(), d_alt[i]);
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, implies)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t i = 0; i < 4; ++i) {
            bool a_val = (bool)(i % 2);
            bool b_val = (bool)(i > 1 ? true : false);
            bool_t a = lhs_constant ? bool_t(a_val) : (witness_t(&composer, a_val));
            bool_t b = rhs_constant ? bool_t(b_val) : (witness_t(&composer, b_val));
            bool_t c = a.implies(b);
            EXPECT_EQ(c.get_value(), !a.get_value() || b.get_value());
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, implies_both_ways)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t i = 0; i < 4; ++i) {
            bool a_val = (bool)(i % 2);
            bool b_val = (bool)(i > 1 ? true : false);
            bool_t a = lhs_constant ? bool_t(a_val) : (witness_t(&composer, a_val));
            bool_t b = rhs_constant ? bool_t(b_val) : (witness_t(&composer, b_val));
            bool_t c = a.implies_both_ways(b);
            EXPECT_EQ(c.get_value(), !(a.get_value() ^ b.get_value()));
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, must_imply)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();
    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t i = 4; i < 14; i += 2) {
            // If a number is divisible by 2 and 3, it is divisible by 6
            bool two = (bool)(i % 2);
            bool three = (bool)(i % 3);
            bool six = (bool)(i % 6);
            bool a_val = (two && three);
            bool b_val = six;
            bool_t a = lhs_constant ? bool_t(a_val) : (witness_t(&composer, a_val));
            bool_t b = rhs_constant ? bool_t(b_val) : (witness_t(&composer, b_val));
            a.must_imply(b);
        }
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, must_imply_fails)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();
    for (size_t j = 0; j < 3; ++j) { // ignore the case when both lhs and rhs are constants
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        // If a number is divisible by 2 and 3, it is divisible by 6
        // => 8 is not divisible by 3, so it must not be divisible by 6
        const size_t i = 8;
        bool a_val = (bool)(i % 2 == 0);
        bool b_val = (bool)(i % 6 == 0);
        bool_t a = lhs_constant ? bool_t(a_val) : (witness_t(&composer, a_val));
        bool_t b = rhs_constant ? bool_t(b_val) : (witness_t(&composer, b_val));
        a.must_imply(b, "div by 2 does not imply div by 8");

        EXPECT_EQ(composer.failed(), true);
        EXPECT_EQ(composer.err(), "div by 2 does not imply div by 8");
    }
}

TEST(stdlib_bool, must_imply_multiple)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();

    /**
     * Define g(x) = 2x + 12
     * if x is divisible by both 4 and 6:
     *     => g(x) > 0
     *     => g(x) is even
     *     => g(x) >= 12
     *     => g(x) is a multiple of 6
     */
    auto g = [](size_t x) { return 2 * x + 12; };

    for (size_t j = 0; j < 3; ++j) { // ignore when both lhs and rhs are constants
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t x = 10; x < 18; x += 2) {
            std::vector<std::pair<bool_t, std::string>> conditions;
            bool four = (bool)(x % 4 == 0);
            bool six = (bool)(x % 6 == 0);

            bool_t a = lhs_constant ? bool_t(four) : (witness_t(&composer, four));
            bool_t b = rhs_constant ? bool_t(six) : (witness_t(&composer, six));

            auto g_x = g(x);
            conditions.push_back(std::make_pair(g_x > 0, "g(x) > 0"));
            conditions.push_back(std::make_pair(g_x % 2 == 0, "g(x) is even"));
            conditions.push_back(std::make_pair(g_x >= 12, "g(x) >= 12"));
            conditions.push_back(std::make_pair(g_x % 6 == 0, "g(x) is a multiple of 6"));

            (a && b).must_imply(conditions);

            if (composer.failed()) {
                EXPECT_EQ(composer.err(), "multi implication fail: g(x) is a multiple of 6");
            } else {
                auto prover = composer.create_prover();
                auto verifier = composer.create_verifier();

                plonk::proof proof = prover.construct_proof();
                bool result = verifier.verify_proof(proof);
                EXPECT_EQ(result, true);
            }
        }
    }
}

TEST(stdlib_bool, must_imply_multiple_fails)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();

    /**
     * Given x = 15:
     * (x > 10)
     *  => (x > 8)
     *  => (x > 5)
     *  ≠> (x > 18)
     */
    for (size_t j = 0; j < 2; ++j) { // ignore when both lhs and rhs are constants
        bool is_constant = (bool)(j % 2);

        size_t x = 15;
        bool main = (bool)(x > 10);
        bool_t main_ct = is_constant ? bool_t(main) : (witness_t(&composer, main));

        std::vector<std::pair<bool_t, std::string>> conditions;
        conditions.push_back(std::make_pair(witness_t(&composer, x > 8), "x > 8"));
        conditions.push_back(std::make_pair(witness_t(&composer, x > 5), "x > 5"));
        conditions.push_back(std::make_pair(witness_t(&composer, x > 18), "x > 18"));

        main_ct.must_imply(conditions);

        EXPECT_EQ(composer.failed(), true);
        EXPECT_EQ(composer.err(), "multi implication fail: x > 18");
    }
}

TEST(stdlib_bool, conditional_assign)
{
    honk::StandardHonkComposer composer = honk::StandardHonkComposer();
    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        const uint256_t x = (uint256_t(1) << 128) - 1;
        const uint256_t val = engine.get_random_uint256();

        bool condition = (val % 2 == 0);
        bool right = x < val;
        bool left = x > val;
        bool_t l_ct = lhs_constant ? bool_t(left) : (witness_t(&composer, left));
        bool_t r_ct = rhs_constant ? bool_t(right) : (witness_t(&composer, right));
        bool_t cond = (witness_t(&composer, condition));

        auto result = bool_t::conditional_assign(cond, l_ct, r_ct);

        EXPECT_EQ(result.get_value(), condition ? left : right);
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();
    info("composer gates = ", composer.get_num_gates());
    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, test_simple_proof)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();
    bool_t a(&composer);
    bool_t b(&composer);
    a = stdlib::witness_t(&composer, barretenberg::fr::one());
    b = stdlib::witness_t(&composer, barretenberg::fr::zero());
    // bool_t c(&composer);
    a = a ^ b;           // a = 1
    b = !b;              // b = 1 (witness 0)
    bool_t c = (a == b); // c = 1
    bool_t d(&composer); // d = ?
    d = false;           // d = 0
    bool_t e = a | d;    // e = 1 = a
    bool_t f = e ^ b;    // f = 0
    d = (!f) & a;        // d = 1
    for (size_t i = 0; i < 64; ++i) {
        a = witness_t(&composer, (bool)(i % 2));
        b = witness_t(&composer, (bool)(i % 3 == 1));
        c = a ^ b;
        a = b ^ c;
        c = a;
        a = b;
        f = b;
    }
    auto prover = composer.create_prover();
    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}

TEST(stdlib_bool, normalize)
{
    honk::StandardHonkComposer composer = proof_system::honk::StandardHonkComposer();

    auto generate_constraints = [&composer](bool value, bool is_constant, bool is_inverted) {
        bool_t a = is_constant ? bool_t(&composer, value) : witness_t(&composer, value);
        bool_t b = is_inverted ? !a : a;
        bool_t c = b.normalize();
        EXPECT_EQ(c.get_value(), value ^ is_inverted);
    };

    generate_constraints(false, false, false);
    generate_constraints(false, false, true);
    generate_constraints(false, true, false);
    generate_constraints(false, true, true);
    generate_constraints(true, false, false);
    generate_constraints(true, false, true);
    generate_constraints(true, true, false);
    generate_constraints(true, true, true);

    auto prover = composer.create_prover();

    auto verifier = composer.create_verifier();

    plonk::proof proof = prover.construct_proof();

    bool result = verifier.verify_proof(proof);
    EXPECT_EQ(result, true);
}
} // namespace test_stdlib_bool
