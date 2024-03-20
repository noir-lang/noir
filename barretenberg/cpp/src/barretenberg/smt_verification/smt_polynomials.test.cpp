#include <chrono>
#include <fstream>
#include <gtest/gtest.h>
#include <iostream>
#include <string>

#include "barretenberg/proof_system/circuit_builder/standard_circuit_builder.hpp"
#include "barretenberg/serialize/cbind.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

#include "barretenberg/smt_verification/circuit/circuit.hpp"
#include "barretenberg/smt_verification/util/smt_util.hpp"

using namespace bb;
using namespace smt_circuit;

using field_t = stdlib::field_t<StandardCircuitBuilder>;
using witness_t = stdlib::witness_t<StandardCircuitBuilder>;
using pub_witness_t = stdlib::public_witness_t<StandardCircuitBuilder>;

namespace {
auto& engine = numeric::get_debug_randomness();
}

msgpack::sbuffer create_polynomial_evaluation_circuit(size_t n, bool pub_coeffs)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    std::vector<field_t> coeffs;
    for (size_t i = 0; i < n; i++) {
        if (pub_coeffs) {
            coeffs.emplace_back(pub_witness_t(&builder, fr::random_element()));
        } else {
            coeffs.emplace_back(witness_t(&builder, fr::random_element()));
        }
        builder.set_variable_name(coeffs.back().get_witness_index(), "coeff_" + std::to_string(i));
    }

    field_t z(witness_t(&builder, 10));
    builder.set_variable_name(z.get_witness_index(), "point");

    field_t res = field_t::from_witness_index(&builder, 0);

    for (size_t i = 0; i < n; i++) {
        res = res * z + coeffs[i];
    }
    builder.set_variable_name(res.get_witness_index(), "result");

    info("evaluation at point ", z, ": ", res);
    info("gates: ", builder.num_gates);
    info("variables: ", builder.get_num_variables());
    info("public inputs: ", builder.get_num_public_inputs());

    return builder.export_circuit();
}

STerm direct_polynomial_evaluation(Circuit& c, size_t n)
{
    STerm point = c["point"];
    STerm result = c["result"];
    STerm ev = c["zero"];
    for (size_t i = 0; i < n; i++) {
        ev = ev * point + c["coeff_" + std::to_string(i)];
    }
    return ev;
}

void model_variables(Circuit& c, Solver* s, STerm& evaluation)
{
    std::unordered_map<std::string, cvc5::Term> terms;
    terms.insert({ "point", c["point"] });
    terms.insert({ "result", c["result"] });
    terms.insert({ "evaluation", evaluation });

    auto values = s->model(terms);

    info("point = ", values["point"]);
    info("circuit_result = ", values["result"]);
    info("function_evaluation = ", values["evaluation"]);
}

TEST(polynomial_evaluation, public)
{
    size_t n = 40;
    auto buf = create_polynomial_evaluation_circuit(n, true);

    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus);
    Circuit circuit(circuit_info, &s, TermType::FFTerm);
    STerm ev = direct_polynomial_evaluation(circuit, n);
    ev != circuit["result"];

    bool res = smt_timer(&s, false);
    ASSERT_FALSE(res);
}

TEST(polynomial_evaluation, private)
{
    size_t n = 40;
    auto buf = create_polynomial_evaluation_circuit(n, false);

    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus);
    Circuit circuit(circuit_info, &s, TermType::FFTerm);
    STerm ev = direct_polynomial_evaluation(circuit, n);
    ev != circuit["result"];

    bool res = smt_timer(&s, false);
    ASSERT_FALSE(res);
    info("Gates: ", circuit.get_num_gates());
    info("Result: ", s.getResult());
}