#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include <fstream>
#include <gtest/gtest.h>
#include <iostream>
#include <string>

#include "barretenberg/stdlib/primitives/field/field.hpp"

#include "barretenberg/smt_verification/circuit/circuit.hpp"

using namespace bb;
using namespace bb;

namespace {
auto& engine = numeric::random::get_debug_engine();
}

using field_t = bb::plonk::stdlib::field_t<StandardCircuitBuilder>;
using witness_t = bb::plonk::stdlib::witness_t<StandardCircuitBuilder>;
using pub_witness_t = bb::plonk::stdlib::public_witness_t<StandardCircuitBuilder>;

TEST(circuit_verification, multiplication_true)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a + a) / (b + b + b);

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(builder.check_circuit());

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus, { true, 0 });
    smt_circuit::Circuit<smt_terms::FFTerm> circuit(circuit_info, &s);
    smt_terms::FFTerm a1 = circuit["a"];
    smt_terms::FFTerm b1 = circuit["b"];
    smt_terms::FFTerm c1 = circuit["c"];
    smt_terms::FFTerm two = smt_terms::FFTerm::Const("2", &s, 10);
    smt_terms::FFTerm thr = smt_terms::FFTerm::Const("3", &s, 10);
    smt_terms::FFTerm cr = smt_terms::FFTerm::Var("cr", &s);
    cr = (two * a1) / (thr * b1);
    c1 != cr;

    bool res = s.check();
    ASSERT_FALSE(res);
}

TEST(circuit_verification, multiplication_true_kind)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a + a) / (b + b + b);

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(builder.check_circuit());

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus, { true, 0 });
    smt_circuit::Circuit<smt_terms::FFTerm> circuit(circuit_info, &s);
    smt_terms::FFTerm a1 = circuit["a"];
    smt_terms::FFTerm b1 = circuit["b"];
    smt_terms::FFTerm c1 = circuit["c"];
    smt_terms::FFTerm two = smt_terms::FFTerm::Const("2", &s, 10);
    smt_terms::FFTerm thr = smt_terms::FFTerm::Const("3", &s, 10);
    smt_terms::FFTerm cr = smt_terms::FFTerm::Var("cr", &s);
    cr* thr* b1 == two* a1;
    c1 != cr;

    bool res = s.check();
    ASSERT_FALSE(res);
}

TEST(circuit_verification, multiplication_false)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a) / (b + b + b); // mistake was here

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(builder.check_circuit());

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus, { true, 0 });
    smt_circuit::Circuit<smt_terms::FFTerm> circuit(circuit_info, &s);

    smt_terms::FFTerm a1 = circuit["a"];
    smt_terms::FFTerm b1 = circuit["b"];
    smt_terms::FFTerm c1 = circuit["c"];

    smt_terms::FFTerm two = smt_terms::FFTerm::Const("2", &s, 10);
    smt_terms::FFTerm thr = smt_terms::FFTerm::Const("3", &s, 10);
    smt_terms::FFTerm cr = smt_terms::FFTerm::Var("cr", &s);
    cr = (two * a1) / (thr * b1);
    c1 != cr;

    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> terms({ { "a", a1 }, { "b", b1 }, { "c", c1 }, { "cr", cr } });

    std::unordered_map<std::string, std::string> vals = s.model(terms);

    info("a = ", vals["a"]);
    info("b = ", vals["b"]);
    info("c = ", vals["c"]);
    info("c_res = ", vals["cr"]);
}

TEST(circuit_verifiaction, unique_witness)
// two roots of a quadratic eq x^2 + a * x + b = s
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(pub_witness_t(&builder, fr::random_element()));
    field_t b(pub_witness_t(&builder, fr::random_element()));
    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    field_t z(witness_t(&builder, fr::random_element()));
    field_t ev = z * z + a * z + b;
    builder.set_variable_name(z.witness_index, "z");
    builder.set_variable_name(ev.witness_index, "ev");

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus, { true, 0 });

    std::pair<smt_circuit::Circuit<smt_terms::FFTerm>, smt_circuit::Circuit<smt_terms::FFTerm>> cirs =
        smt_circuit::unique_witness<smt_terms::FFTerm>(circuit_info, &s, { "ev" }, { "z" });

    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> terms = { { "z_c1", cirs.first["z"] }, { "z_c2", cirs.second["z"] } };
    std::unordered_map<std::string, std::string> vals = s.model(terms);
    ASSERT_NE(vals["z_c1"], vals["z_c2"]);
}

using namespace smt_terms;

TEST(solver_use_case, solver)
{
    Solver s("11", { true, 0 }, 10);
    FFTerm x = FFTerm::Var("x", &s);
    FFTerm y = FFTerm::Var("y", &s);

    FFTerm z = x * y + x * x;
    z == FFTerm::Const("15", &s, 10);
    x != y;
    x != FFTerm::Const("0", &s);
    y != FFTerm::Const("0", &s);

    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> vars = { { "x", x }, { "y", y } };
    std::unordered_map<std::string, std::string> mvars = s.model(vars);

    info("x = ", mvars["x"]);
    info("y = ", mvars["y"]);
}

// TODO(alex): Try setting the whole witness to be not equal at the same time, while setting inputs and outputs to be
// equal