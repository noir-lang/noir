#include "barretenberg/smt_verification/circuit/circuit.hpp"
#include <gtest/gtest.h>
#include <unordered_map>

// TODO(alex): more tests

TEST(integer_mod, basic_arithmetic)
{
    smt_solver::Solver s("101", { true, 0 }, 10);

    smt_terms::FFITerm x = smt_terms::FFITerm::Var("x", &s);
    smt_terms::FFITerm y = smt_terms::FFITerm::Var("y", &s);
    smt_terms::FFITerm z = smt_terms::FFITerm::Const("79", &s);

    (x * y) == z;
    info(s.check());

    std::string xval = s.s.getValue(x.term).getIntegerValue();
    std::string yval = s.s.getValue(y.term).getIntegerValue();
    info(xval, " ", yval);
}