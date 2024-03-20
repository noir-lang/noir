#include "solver.hpp"
#include "barretenberg/smt_verification/terms/term.hpp"

#include <gtest/gtest.h>

using namespace smt_terms;

// Find a point of order 3 on y^2 = x^3 + 2 mod 101 curve
// Terrible when using ffiterm
TEST(Solver, FFTerm_use_case)
{
    Solver s("101", default_solver_config, 10);
    STerm x = FFVar("x", &s);
    STerm y = FFVar("y", &s);

    y* y == x* x* x + bb::fr(2);
    STerm l = (3 * x * x) / (bb::fr(2) * y);
    STerm xr = l * l - x - x;
    STerm yr = l * (x - xr) - y;
    x == xr;
    y == -yr;
    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> vars = { { "Gx", x }, { "Gy", y } };
    std::unordered_map<std::string, std::string> mvars = s.model(vars);

    std::vector<cvc5::Term> terms = { x, y };
    std::unordered_map<std::string, std::string> vvars = s.model(terms);
    ASSERT_EQ(mvars["Gx"], vvars["x"]);
    ASSERT_EQ(mvars["Gy"], vvars["y"]);
}

TEST(Solver, FFITerm_use_case)
{
    Solver s("bce4e33b636e0cf38d13a55c3");
    STerm x = FFIVar("x", &s);
    STerm y = FFIVar("y", &s);

    bb::fr a = bb::fr::random_element();
    x <= bb::fr(2).pow(32);
    x >= bb::fr(2).pow(10);
    (x + y) == a;
    bool res = s.check();
    ASSERT_TRUE(res);

    std::vector<cvc5::Term> terms = { x, y };
    std::unordered_map<std::string, std::string> vvars = s.model(terms);
    info(vvars["x"]);
    info("+");
    info(vvars["y"]);
    info("=");
    info(s.getValue(STerm(a, &s, TermType::FFITerm).term));
}

TEST(Solver, human_readable_constraints_FFTerm)
{
    Solver s("101", default_solver_config, 10);
    STerm x = FFVar("x", &s);
    STerm y = FFVar("y", &s);
    y* y == x* x* x + bb::fr(2);
    STerm l = (3 * x * x) / (bb::fr(2) * y);
    STerm xr = l * l - x - x;
    STerm yr = l * (x - xr) - y;
    x == xr;
    y == -yr;
    s.print_assertions();
}

TEST(Solver, human_readable_constraints_FFITerm)
{
    Solver s("101", default_solver_config, 10);
    STerm x = FFIVar("x", &s);
    STerm y = FFIVar("y", &s);
    y* y == x* x* x + bb::fr(2);
    STerm l = (3 * x * x) / (bb::fr(2) * y);
    STerm xr = l * l - x - x;
    STerm yr = l * (x - xr) - y;
    x == xr;
    y == -yr;
    s.print_assertions();
}