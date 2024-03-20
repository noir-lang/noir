#include <unordered_map>

#include "term.hpp"

#include <gtest/gtest.h>

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

using namespace smt_terms;

TEST(FFITerm, addition)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a + b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFIVar("x", &s);
    STerm y = FFIVar("y", &s);
    STerm z = x + y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();

    STerm bval = STerm(b, &s, TermType::FFITerm);
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFITerm, subtraction)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a - b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFIVar("x", &s);
    STerm y = FFIVar("y", &s);
    STerm bval = STerm(b, &s, TermType::FFITerm);
    STerm z = x - y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFITerm, multiplication)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a * b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFIVar("x", &s);
    STerm y = FFIVar("y", &s);
    STerm z = x * y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();

    STerm bval = STerm(b, &s, TermType::FFITerm);
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFITerm, division)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a / b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFIVar("x", &s);
    STerm y = FFIVar("y", &s);
    STerm z = x / y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();

    STerm bval = STerm(b, &s, TermType::FFITerm);
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

// This test aims to check for the absence of unintended
// behavior. If an unsupported operator is called, an info message appears in stderr
// and the value is supposed to remain unchanged.
TEST(FFITerm, unsupported_operations)
{
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFIVar("x", &s);
    STerm y = FFIVar("y", &s);

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