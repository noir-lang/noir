#include "barretenberg/stdlib/primitives/byte_array/byte_array.cpp"
#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders.hpp"
#include "bool.hpp"
#include <gtest/gtest.h>

#define STDLIB_TYPE_ALIASES                                                                                            \
    using Composer = TypeParam;                                                                                        \
    using witness_ct = stdlib::witness_t<Composer>;                                                                    \
    using bool_ct = stdlib::bool_t<Composer>;

namespace test_stdlib_bool {
using namespace barretenberg;
using namespace proof_system::plonk;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

template <class Composer> class BoolTest : public ::testing::Test {};

using CircuitTypes = ::testing::
    Types<proof_system::StandardCircuitBuilder, proof_system::TurboCircuitBuilder, proof_system::UltraCircuitBuilder>;

TYPED_TEST_SUITE(BoolTest, CircuitTypes);
TYPED_TEST(BoolTest, TestBasicOperations)
{

    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    auto gates_before = composer.get_num_gates();

    bool_ct a = witness_ct(&composer, barretenberg::fr::one());
    bool_ct b = witness_ct(&composer, barretenberg::fr::zero());
    a = a ^ b; // a = 1
    EXPECT_EQ(a.get_value(), 1);
    b = !b; // b = 1 (witness 0)
    EXPECT_EQ(b.get_value(), 1);
    bool_ct d = (a == b); //
    EXPECT_EQ(d.get_value(), 1);
    d = false; // d = 0
    EXPECT_EQ(d.get_value(), 0);
    bool_ct e = a | d; // e = 1 = a
    EXPECT_EQ(e.get_value(), 1);
    bool_ct f = e ^ b; // f = 0
    EXPECT_EQ(f.get_value(), 0);
    d = (!f) & a; // d = 1
    EXPECT_EQ(d.get_value(), 1);

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);

    auto gates_after = composer.get_num_gates();
    EXPECT_EQ(gates_after - gates_before, 6UL);
}

TYPED_TEST(BoolTest, Xor)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t i = 0; i < 4; ++i) {
            bool a_val = (bool)(i % 2);
            bool b_val = (bool)(i > 1 ? true : false);
            bool_ct a = lhs_constant ? bool_ct(a_val) : (witness_ct(&composer, a_val));
            bool_ct b = rhs_constant ? bool_ct(b_val) : (witness_ct(&composer, b_val));
            bool_ct c = a ^ b;
            EXPECT_EQ(c.get_value(), a.get_value() ^ b.get_value());
        }
    }
    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, XorConstants)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t i = 0; i < 32; ++i) {
        bool_ct a = witness_ct(&composer, (bool)(i % 2));
        bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
        a ^ b;
    }
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_ct a = witness_ct(&composer, (bool)(i % 2));
            bool_ct b(&composer, (bool)(i % 3 == 1));
            a ^ b;
        } else {
            bool_ct a(&composer, (bool)(i % 2));
            bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
            a ^ b;
        }
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, XorTwinConstants)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    bool_ct c;
    for (size_t i = 0; i < 32; ++i) {
        bool_ct a(&composer, (i % 1) == 0);
        bool_ct b(&composer, (i % 1) == 1);
        c = c ^ a ^ b;
    }
    c = c ^ bool_ct(witness_ct(&composer, true));
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_ct a = witness_ct(&composer, (bool)(i % 2));
            bool_ct b(&composer, (bool)(i % 3 == 1));
            c = c ^ a ^ b;
        } else {
            bool_ct a(&composer, (bool)(i % 2));
            bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
            c = c ^ a ^ b;
        }
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, LogicalAnd)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    bool_ct a = witness_ct(&composer, 1);
    bool_ct b = witness_ct(&composer, 1);
    (!a) && (!b);

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, And)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t i = 0; i < 32; ++i) {
        bool_ct a = witness_ct(&composer, (bool)(i % 1));
        bool_ct b = witness_ct(&composer, (bool)(i % 2 == 1));
        a& b;
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, AndConstants)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t i = 0; i < 32; ++i) {
        bool_ct a = witness_ct(&composer, (bool)(i % 2));
        bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
        a& b;
    }
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_ct a = witness_ct(&composer, (bool)(i % 2));
            bool_ct b(&composer, (bool)(i % 3 == 1));
            a& b;
        } else {
            bool_ct a(&composer, (bool)(i % 2));
            bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
            a& b;
        }
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, or)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t i = 0; i < 32; ++i) {
        bool_ct a = witness_ct(&composer, (bool)(i % 2));
        bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
        a | b;
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, OrConstants)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t i = 0; i < 32; ++i) {
        bool_ct a = witness_ct(&composer, (bool)(i % 2));
        bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
        a | b;
    }
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            bool_ct a = witness_ct(&composer, (bool)(i % 2));
            bool_ct b(&composer, (bool)(i % 3 == 1));
            a | b;
        } else {
            bool_ct a(&composer, (bool)(i % 2));
            bool_ct b = witness_ct(&composer, (bool)(i % 3 == 1));
            a | b;
        }
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, Eq)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

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
    bool_ct a[32];
    bool_ct b[32];
    bool_ct c[32];
    bool_ct d[32];
    for (size_t i = 0; i < 32; ++i) {
        if (i % 2 == 0) {
            a[i] = witness_ct(&composer, (bool)(i % 2));
            b[i] = witness_ct(&composer, (bool)(0));
            c[i] = a[i] ^ b[i];
            d[i] = a[i] == c[i];
        } else {
            a[i] = witness_ct(&composer, (bool)(1));
            b[i] = witness_ct(&composer, (bool)(0));
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

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, Implies)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t i = 0; i < 4; ++i) {
            bool a_val = (bool)(i % 2);
            bool b_val = (bool)(i > 1 ? true : false);
            bool_ct a = lhs_constant ? bool_ct(a_val) : (witness_ct(&composer, a_val));
            bool_ct b = rhs_constant ? bool_ct(b_val) : (witness_ct(&composer, b_val));
            bool_ct c = a.implies(b);
            EXPECT_EQ(c.get_value(), !a.get_value() || b.get_value());
        }
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, ImpliesBothWays)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        for (size_t i = 0; i < 4; ++i) {
            bool a_val = (bool)(i % 2);
            bool b_val = (bool)(i > 1 ? true : false);
            bool_ct a = lhs_constant ? bool_ct(a_val) : (witness_ct(&composer, a_val));
            bool_ct b = rhs_constant ? bool_ct(b_val) : (witness_ct(&composer, b_val));
            bool_ct c = a.implies_both_ways(b);
            EXPECT_EQ(c.get_value(), !(a.get_value() ^ b.get_value()));
        }
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, MustImply)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

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
            bool_ct a = lhs_constant ? bool_ct(a_val) : (witness_ct(&composer, a_val));
            bool_ct b = rhs_constant ? bool_ct(b_val) : (witness_ct(&composer, b_val));
            a.must_imply(b);
        }
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, MustImplyFails)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t j = 0; j < 3; ++j) { // ignore the case when both lhs and rhs are constants
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        // If a number is divisible by 2 and 3, it is divisible by 6
        // => 8 is not divisible by 3, so it must not be divisible by 6
        const size_t i = 8;
        bool a_val = (bool)(i % 2 == 0);
        bool b_val = (bool)(i % 6 == 0);
        bool_ct a = lhs_constant ? bool_ct(a_val) : (witness_ct(&composer, a_val));
        bool_ct b = rhs_constant ? bool_ct(b_val) : (witness_ct(&composer, b_val));
        a.must_imply(b, "div by 2 does not imply div by 8");

        EXPECT_EQ(composer.failed(), true);
        EXPECT_EQ(composer.err(), "div by 2 does not imply div by 8");
    }
}

TYPED_TEST(BoolTest, MustImplyMultiple)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

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
            std::vector<std::pair<bool_ct, std::string>> conditions;
            bool four = (bool)(x % 4 == 0);
            bool six = (bool)(x % 6 == 0);

            bool_ct a = lhs_constant ? bool_ct(four) : (witness_ct(&composer, four));
            bool_ct b = rhs_constant ? bool_ct(six) : (witness_ct(&composer, six));

            auto g_x = g(x);
            conditions.push_back(std::make_pair(g_x > 0, "g(x) > 0"));
            conditions.push_back(std::make_pair(g_x % 2 == 0, "g(x) is even"));
            conditions.push_back(std::make_pair(g_x >= 12, "g(x) >= 12"));
            conditions.push_back(std::make_pair(g_x % 6 == 0, "g(x) is a multiple of 6"));

            (a && b).must_imply(conditions);

            if (composer.failed()) {
                EXPECT_EQ(composer.err(), "multi implication fail: g(x) is a multiple of 6");
            } else {
                bool result = composer.check_circuit();
                EXPECT_EQ(result, true);
            }
        }
    }
}

TYPED_TEST(BoolTest, MustImplyMultipleFails)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

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
        bool_ct main_ct = is_constant ? bool_ct(main) : (witness_ct(&composer, main));

        std::vector<std::pair<bool_ct, std::string>> conditions;
        conditions.push_back(std::make_pair(witness_ct(&composer, x > 8), "x > 8"));
        conditions.push_back(std::make_pair(witness_ct(&composer, x > 5), "x > 5"));
        conditions.push_back(std::make_pair(witness_ct(&composer, x > 18), "x > 18"));

        main_ct.must_imply(conditions);

        EXPECT_EQ(composer.failed(), true);
        EXPECT_EQ(composer.err(), "multi implication fail: x > 18");
    }
}

TYPED_TEST(BoolTest, ConditionalAssign)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    for (size_t j = 0; j < 4; ++j) {
        bool lhs_constant = (bool)(j % 2);
        bool rhs_constant = (bool)(j > 1 ? true : false);

        const uint256_t x = (uint256_t(1) << 128) - 1;
        const uint256_t val = engine.get_random_uint256();

        bool condition = (val % 2 == 0);
        bool right = x < val;
        bool left = x > val;
        bool_ct l_ct = lhs_constant ? bool_ct(left) : (witness_ct(&composer, left));
        bool_ct r_ct = rhs_constant ? bool_ct(right) : (witness_ct(&composer, right));
        bool_ct cond = (witness_ct(&composer, condition));

        auto result = bool_ct::conditional_assign(cond, l_ct, r_ct);

        EXPECT_EQ(result.get_value(), condition ? left : right);
    }
    info("composer gates = ", composer.get_num_gates());
    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, TestSimpleProof)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    bool_ct a(&composer);
    bool_ct b(&composer);
    a = witness_ct(&composer, barretenberg::fr::one());
    b = witness_ct(&composer, barretenberg::fr::zero());
    // bool_ct c(&composer);
    a = a ^ b;            // a = 1
    b = !b;               // b = 1 (witness 0)
    bool_ct c = (a == b); // c = 1
    bool_ct d(&composer); // d = ?
    d = false;            // d = 0
    bool_ct e = a | d;    // e = 1 = a
    bool_ct f = e ^ b;    // f = 0
    d = (!f) & a;         // d = 1
    for (size_t i = 0; i < 64; ++i) {
        a = witness_ct(&composer, (bool)(i % 2));
        b = witness_ct(&composer, (bool)(i % 3 == 1));
        c = a ^ b;
        a = b ^ c;
        c = a;
        a = b;
        f = b;
    }

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}

TYPED_TEST(BoolTest, Normalize)
{
    STDLIB_TYPE_ALIASES
    auto composer = Composer();

    auto generate_constraints = [&composer](bool value, bool is_constant, bool is_inverted) {
        bool_ct a = is_constant ? bool_ct(&composer, value) : witness_ct(&composer, value);
        bool_ct b = is_inverted ? !a : a;
        bool_ct c = b.normalize();
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

    bool result = composer.check_circuit();
    EXPECT_EQ(result, true);
}
} // namespace test_stdlib_bool