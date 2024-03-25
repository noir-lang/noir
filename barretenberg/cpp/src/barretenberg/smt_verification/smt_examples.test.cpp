#include "barretenberg/circuit_checker/circuit_checker.hpp"
#include "barretenberg/stdlib_circuit_builders/standard_circuit_builder.hpp"
#include <fstream>
#include <gtest/gtest.h>
#include <iostream>
#include <string>

#include "barretenberg/stdlib/primitives/field/field.hpp"

#include "barretenberg/smt_verification/circuit/circuit.hpp"

using namespace bb;

namespace {
auto& engine = numeric::get_debug_randomness();
}

using field_t = stdlib::field_t<StandardCircuitBuilder>;
using witness_t = stdlib::witness_t<StandardCircuitBuilder>;
using pub_witness_t = stdlib::public_witness_t<StandardCircuitBuilder>;

TEST(SMT_Example, multiplication_true)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a + a) / (b + b + b);

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(CircuitChecker::check(builder));

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus);
    smt_circuit::Circuit circuit(circuit_info, &s, smt_terms::TermType::FFTerm);
    smt_terms::STerm a1 = circuit["a"];
    smt_terms::STerm b1 = circuit["b"];
    smt_terms::STerm c1 = circuit["c"];
    smt_terms::STerm two = smt_terms::FFConst("2", &s, 10);
    smt_terms::STerm thr = smt_terms::FFConst("3", &s, 10);
    smt_terms::STerm cr = smt_terms::FFVar("cr", &s);
    cr = (two * a1) / (thr * b1);
    c1 != cr;

    bool res = s.check();
    ASSERT_FALSE(res);
}

TEST(SMT_Example, multiplication_true_kind)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a + a) / (b + b + b);

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(CircuitChecker::check(builder));

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus);
    smt_circuit::Circuit circuit(circuit_info, &s, smt_terms::TermType::FFTerm);
    smt_terms::STerm a1 = circuit["a"];
    smt_terms::STerm b1 = circuit["b"];
    smt_terms::STerm c1 = circuit["c"];
    smt_terms::STerm two = smt_terms::FFConst("2", &s, 10);
    smt_terms::STerm thr = smt_terms::FFConst("3", &s, 10);
    smt_terms::STerm cr = smt_terms::FFVar("cr", &s);
    cr* thr* b1 == two* a1;
    c1 != cr;

    bool res = s.check();
    ASSERT_FALSE(res);
}

TEST(SMT_Example, multiplication_false)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    field_t c = (a) / (b + b + b); // mistake was here

    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    builder.set_variable_name(c.witness_index, "c");
    ASSERT_TRUE(CircuitChecker::check(builder));

    auto buf = builder.export_circuit();

    smt_circuit::CircuitSchema circuit_info = smt_circuit::unpack_from_buffer(buf);
    smt_solver::Solver s(circuit_info.modulus);
    smt_circuit::Circuit circuit(circuit_info, &s, smt_terms::TermType::FFTerm);

    smt_terms::STerm a1 = circuit["a"];
    smt_terms::STerm b1 = circuit["b"];
    smt_terms::STerm c1 = circuit["c"];

    smt_terms::STerm two = smt_terms::FFConst("2", &s, 10);
    smt_terms::STerm thr = smt_terms::FFConst("3", &s, 10);
    smt_terms::STerm cr = smt_terms::FFVar("cr", &s);
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

// Make sure that quadratic polynomial evaluation doesn't have unique
// witness using unique_witness_ext function
// Find both roots of a quadratic equation x^2 + a * x + b = s
TEST(SMT_Example, unique_witness_ext)
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
    smt_solver::Solver s(circuit_info.modulus);

    std::pair<smt_circuit::Circuit, smt_circuit::Circuit> cirs =
        smt_circuit::unique_witness_ext(circuit_info, &s, smt_terms::TermType::FFTerm, { "ev" }, { "z" });

    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> terms = { { "z_c1", cirs.first["z"] }, { "z_c2", cirs.second["z"] } };
    std::unordered_map<std::string, std::string> vals = s.model(terms);
    ASSERT_NE(vals["z_c1"], vals["z_c2"]);
}

// Make sure that quadratic polynomial evaluation doesn't have unique
// witness using unique_witness function
// Finds both roots of a quadratic eq x^2 + a * x + b = s
TEST(SMT_Example, unique_witness)
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
    smt_solver::Solver s(circuit_info.modulus);

    std::pair<smt_circuit::Circuit, smt_circuit::Circuit> cirs =
        smt_circuit::unique_witness(circuit_info, &s, smt_terms::TermType::FFTerm, { "ev" });

    bool res = s.check();
    ASSERT_TRUE(res);

    std::unordered_map<std::string, cvc5::Term> terms = { { "z_c1", cirs.first["z"] }, { "z_c2", cirs.second["z"] } };
    std::unordered_map<std::string, std::string> vals = s.model(terms);
    ASSERT_NE(vals["z_c1"], vals["z_c2"]);
}