#include <unordered_map>

#include "term.hpp"

#include <gtest/gtest.h>

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

using namespace smt_terms;

TEST(FFTerm, addition)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a + b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFVar("x", &s);
    STerm y = FFVar("y", &s);
    STerm z = x + y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getFiniteFieldValue();

    STerm bval = STerm(b, &s, TermType::FFTerm);
    std::string bvals = s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFTerm, subtraction)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a - b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFVar("x", &s);
    STerm y = FFVar("y", &s);
    STerm z = x - y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getFiniteFieldValue();

    STerm bval = STerm(b, &s, TermType::FFTerm);
    std::string bvals = s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFTerm, multiplication)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a * b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFVar("x", &s);
    STerm y = FFVar("y", &s);
    STerm z = x * y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getFiniteFieldValue();

    STerm bval = STerm(b, &s, TermType::FFTerm);
    std::string bvals = s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFTerm, division)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a / b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFVar("x", &s);
    STerm y = FFVar("y", &s);
    STerm z = x / y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getFiniteFieldValue();

    STerm bval = STerm(b, &s, TermType::FFTerm);
    std::string bvals = s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}

// This test aims to check for the absence of unintended
// behavior. If an unsupported operator is called, an info message appears in stderr
// and the value is supposed to remain unchanged.
TEST(FFTerm, unsupported_operations)
{
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    STerm x = FFVar("x", &s);
    STerm y = FFVar("y", &s);

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

    size_t n = s.solver.getAssertions().size();
    z <= bb::fr(10);
    ASSERT_EQ(n, s.solver.getAssertions().size());
    z < bb::fr(10);
    ASSERT_EQ(n, s.solver.getAssertions().size());
    z > bb::fr(10);
    ASSERT_EQ(n, s.solver.getAssertions().size());
    z >= bb::fr(10);
    ASSERT_EQ(n, s.solver.getAssertions().size());
}