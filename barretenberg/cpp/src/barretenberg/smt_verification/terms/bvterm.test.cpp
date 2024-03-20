#include <unordered_map>

#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "term.hpp"

#include <gtest/gtest.h>

namespace {
auto& engine = bb::numeric::get_debug_randomness();
}

using namespace bb;
using witness_ct = stdlib::witness_t<StandardCircuitBuilder>;
using uint_ct = stdlib::uint32<StandardCircuitBuilder>;

using namespace smt_terms;

TEST(BVTerm, addition)
{
    StandardCircuitBuilder builder;
    uint_ct a = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct b = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct c = a + b;

    uint32_t modulus_base = 16;
    uint32_t bitvector_size = 32;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
             default_solver_config,
             modulus_base,
             bitvector_size);

    STerm x = BVVar("x", &s);
    STerm y = BVVar("y", &s);
    STerm z = x + y;

    z == c.get_value();
    x == a.get_value();
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getBitVectorValue();

    STerm bval = STerm(b.get_value(), &s, TermType::BVTerm);
    std::string bvals = s.getValue(bval.term).getBitVectorValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(BVTerm, subtraction)
{
    StandardCircuitBuilder builder;
    uint_ct a = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct b = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct c = a - b;

    uint32_t modulus_base = 16;
    uint32_t bitvector_size = 32;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
             default_solver_config,
             modulus_base,
             bitvector_size);

    STerm x = BVVar("x", &s);
    STerm y = BVVar("y", &s);
    STerm z = x - y;

    z == c.get_value();
    x == a.get_value();
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y.term).getBitVectorValue();

    STerm bval = STerm(b.get_value(), &s, TermType::BVTerm);
    std::string bvals = s.getValue(bval.term).getBitVectorValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(BVTerm, xor)
{
    StandardCircuitBuilder builder;
    uint_ct a = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct b = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct c = a ^ b;

    uint32_t modulus_base = 16;
    uint32_t bitvector_size = 32;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
             default_solver_config,
             modulus_base,
             bitvector_size);

    STerm x = BVVar("x", &s);
    STerm y = BVVar("y", &s);
    STerm z = x ^ y;

    z == c.get_value();
    x == a.get_value();
    ASSERT_TRUE(s.check());

    std::string yvals = s.getValue(y).getBitVectorValue();

    STerm bval = STerm(b.get_value(), &s, TermType::BVTerm);
    std::string bvals = s.getValue(bval.term).getBitVectorValue();
    ASSERT_EQ(bvals, yvals);
}

TEST(BVTerm, rotr)
{
    StandardCircuitBuilder builder;
    uint_ct a = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct b = a.ror(10);

    uint32_t modulus_base = 16;
    uint32_t bitvector_size = 32;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
             default_solver_config,
             modulus_base,
             bitvector_size);

    STerm x = BVVar("x", &s);
    STerm y = x.rotr(10);

    y == b.get_value();
    ASSERT_TRUE(s.check());

    std::string xvals = s.getValue(x.term).getBitVectorValue();

    STerm bval = STerm(a.get_value(), &s, TermType::BVTerm);
    std::string bvals = s.getValue(bval.term).getBitVectorValue();
    ASSERT_EQ(bvals, xvals);
}

TEST(BVTerm, rotl)
{
    StandardCircuitBuilder builder;
    uint_ct a = witness_ct(&builder, static_cast<uint32_t>(fr::random_element()));
    uint_ct b = a.rol(10);

    uint32_t modulus_base = 16;
    uint32_t bitvector_size = 32;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
             default_solver_config,
             modulus_base,
             bitvector_size);

    STerm x = BVVar("x", &s);
    STerm y = x.rotl(10);

    y == b.get_value();
    ASSERT_TRUE(s.check());

    std::string xvals = s.getValue(x.term).getBitVectorValue();

    STerm bval = STerm(a.get_value(), &s, TermType::BVTerm);
    std::string bvals = s.getValue(bval.term).getBitVectorValue();
    ASSERT_EQ(bvals, xvals);
}

// MUL, LSH, RSH, AND and OR are not tested, since they are not bijective

// This test aims to check for the absence of unintended
// behavior. If an unsupported operator is called, an info message appears in stderr
// and the value is supposed to remain unchanged.
TEST(BVTerm, unsupported_operations)
{

    uint32_t modulus_base = 16;
    uint32_t bitvector_size = 32;
    Solver s("30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
             default_solver_config,
             modulus_base,
             bitvector_size);

    STerm x = BVVar("x", &s);
    STerm y = BVVar("y", &s);

    STerm z = x / y;
    ASSERT_EQ(z.term, x.term);
}