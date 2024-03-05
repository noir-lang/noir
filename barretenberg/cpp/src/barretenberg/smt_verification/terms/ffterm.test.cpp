#include <unordered_map>

#include "ffterm.hpp"

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
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", { true, 0 }, 16);

    FFTerm x = FFTerm::Var("x", &s);
    FFTerm y = FFTerm::Var("y", &s);
    FFTerm bval = FFTerm(b, &s);
    FFTerm z = x + y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.s.getValue(y.term).getFiniteFieldValue();
    std::string bvals = s.s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFTerm, subtraction)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a - b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", { true, 0 }, 16);

    FFTerm x = FFTerm::Var("x", &s);
    FFTerm y = FFTerm::Var("y", &s);
    FFTerm bval = FFTerm(b, &s);
    FFTerm z = x - y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.s.getValue(y.term).getFiniteFieldValue();
    std::string bvals = s.s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFTerm, multiplication)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a * b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", { true, 0 }, 16);

    FFTerm x = FFTerm::Var("x", &s);
    FFTerm y = FFTerm::Var("y", &s);
    FFTerm bval = FFTerm(b, &s);
    FFTerm z = x * y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.s.getValue(y.term).getFiniteFieldValue();
    std::string bvals = s.s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFTerm, division)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a / b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001", { true, 0 }, 16);

    FFTerm x = FFTerm::Var("x", &s);
    FFTerm y = FFTerm::Var("y", &s);
    FFTerm bval = FFTerm(b, &s);
    FFTerm z = x / y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.s.getValue(y.term).getFiniteFieldValue();
    std::string bvals = s.s.getValue(bval.term).getFiniteFieldValue();
    ASSERT_EQ(bvals, yvals);
}