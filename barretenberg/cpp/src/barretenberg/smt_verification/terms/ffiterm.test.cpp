#include <unordered_map>

#include "ffiterm.hpp"

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

    FFITerm x = FFITerm::Var("x", &s);
    FFITerm y = FFITerm::Var("y", &s);
    FFITerm bval = FFITerm(b, &s);
    FFITerm z = x + y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFITerm, subtraction)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a - b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    FFITerm x = FFITerm::Var("x", &s);
    FFITerm y = FFITerm::Var("y", &s);
    FFITerm bval = FFITerm(b, &s);
    FFITerm z = x - y;

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

    FFITerm x = FFITerm::Var("x", &s);
    FFITerm y = FFITerm::Var("y", &s);
    FFITerm bval = FFITerm(b, &s);
    FFITerm z = x * y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(FFITerm, division)
{
    bb::fr a = bb::fr::random_element();
    bb::fr b = bb::fr::random_element();
    bb::fr c = a / b;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

    FFITerm x = FFITerm::Var("x", &s);
    FFITerm y = FFITerm::Var("y", &s);
    FFITerm bval = FFITerm(b, &s);
    FFITerm z = x / y;

    z == c;
    x == a;
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getIntegerValue();
    std::string bvals = s.getValue(bval.term).getIntegerValue();
    ASSERT_EQ(bvals, yvals);
}
