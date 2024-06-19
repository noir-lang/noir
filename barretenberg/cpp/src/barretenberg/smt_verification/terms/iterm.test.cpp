#include <unordered_map>

#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "term.hpp"

#include <gtest/gtest.h>

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

using namespace bb;
using witness_ct = stdlib::witness_t<StandardCircuitBuilder>;

using namespace smt_terms;

TEST(ITerm, addition)
{
    StandardCircuitBuilder builder;
    uint64_t a = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31);
    uint64_t b = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31);
    uint64_t c = a + b;

    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", default_solver_config);

    STerm x = IVar("x", &s);
    STerm y = IVar("y", &s);
    STerm z = x + y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();

    STerm bval = STerm(b, &s, TermType::ITerm);
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(ITerm, subtraction)
{
    StandardCircuitBuilder builder;
    uint64_t c = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31);
    uint64_t b = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31);
    uint64_t a = c + b;

    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", default_solver_config);

    STerm x = IVar("x", &s);
    STerm y = IVar("y", &s);
    STerm z = x - y;

    x == a;
    z == c;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();

    STerm bval = STerm(b, &s, TermType::ITerm);
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(ITerm, mul)
{
    StandardCircuitBuilder builder;
    uint64_t a = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31);
    uint64_t b = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31);
    uint64_t c = a * b;

    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", default_solver_config);

    STerm x = IVar("x", &s);
    STerm y = IVar("y", &s);
    STerm z = x * y;

    x == a;
    y == b;

    ASSERT_TRUE(s.check());

    std::string xvals = s.getValue(z.term).getIntegerValue();
    STerm bval = STerm(c, &s, TermType::ITerm);
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, xvals);
}

TEST(ITerm, div)
{
    StandardCircuitBuilder builder;
    uint64_t a = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31);
    uint64_t b = static_cast<uint32_t>(fr::random_element()) % (static_cast<uint32_t>(1) << 31) + 1;
    uint64_t c = a / b;

    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", default_solver_config);

    STerm x = IVar("x", &s);
    STerm y = IVar("y", &s);
    STerm z = x / y;

    x == a;
    y == b;

    ASSERT_TRUE(s.check());

    std::string xvals = s.getValue(z.term).getIntegerValue();
    STerm bval = STerm(c, &s, TermType::ITerm);
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, xvals);
}

// This test aims to check for the absence of unintended
// behavior. If an unsupported operator is called, an info message appears in stderr
// and the value is supposed to remain unchanged.
TEST(ITerm, unsupported_operations)
{
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = IVar("x", &s);
    STerm y = IVar("y", &s);

    STerm z = x ^ y;
    ASSERT_EQ(z.term, x.term);
    z = x & y;
    ASSERT_EQ(z.term, x.term);
    z = x | y;
    ASSERT_EQ(z.term, x.term);
    z = x >> 10;
    ASSERT_EQ(z.term, x.term);
    z = x << 10;
    ASSERT_EQ(z.term, x.term);
    z = x.rotr(10);
    ASSERT_EQ(z.term, x.term);
    z = x.rotl(10);
    ASSERT_EQ(z.term, x.term);

    cvc5::Term before_term = x.term;
    x ^= y;
    ASSERT_EQ(x.term, before_term);
    x &= y;
    ASSERT_EQ(x.term, before_term);
    x |= y;
    ASSERT_EQ(x.term, before_term);
    x >>= 10;
    ASSERT_EQ(x.term, before_term);
    x <<= 10;
    ASSERT_EQ(x.term, before_term);
}