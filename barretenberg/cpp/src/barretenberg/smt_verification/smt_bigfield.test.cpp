#include "barretenberg/numeric/random/engine.hpp"

#include "barretenberg/ecc/curves/bn254/fq.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"

#include "barretenberg/stdlib/primitives/bigfield/bigfield.hpp"
#include "barretenberg/stdlib/primitives/bool/bool.hpp"
#include "barretenberg/stdlib/primitives/byte_array/byte_array.hpp"
#include "barretenberg/stdlib/primitives/field/field.hpp"

#include "barretenberg/plonk/proof_system/constants.hpp"
#include "barretenberg/plonk/proof_system/prover/prover.hpp"
#include "barretenberg/plonk/proof_system/verifier/verifier.hpp"

#include "barretenberg/stdlib/primitives/circuit_builders/circuit_builders.hpp"
#include "barretenberg/stdlib/primitives/curves/bn254.hpp"

#include "barretenberg/polynomials/polynomial_arithmetic.hpp"
#include <fstream>
#include <memory>
#include <utility>

#include <chrono>
#include <gtest/gtest.h>
#include <iostream>

#include "barretenberg/smt_verification/circuit/circuit.hpp"

using namespace smt_circuit;
using namespace bb;
using namespace bb::plonk;

using field_ct = bb::plonk::stdlib::field_t<StandardCircuitBuilder>;
using witness_t = bb::plonk::stdlib::witness_t<StandardCircuitBuilder>;
using pub_witness_t = bb::plonk::stdlib::public_witness_t<StandardCircuitBuilder>;

using bn254 = stdlib::bn254<StandardCircuitBuilder>;

using fr_ct = bn254::ScalarField;
using fq_ct = bn254::BaseField;
using public_witness_ct = bn254::public_witness_ct;
using witness_ct = bn254::witness_ct;

SolverConfiguration config = { true, 0 };

msgpack::sbuffer create_circuit(bool pub_ab, bool ab)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    fq inputs[2]{ fq::random_element(), fq::random_element() };
    fq_ct a, b;
    if (pub_ab) {
        a = fq_ct(public_witness_ct(&builder, fr(uint256_t(inputs[0]).slice(0, fq_ct::NUM_LIMB_BITS * 2))),
                  public_witness_ct(
                      &builder, fr(uint256_t(inputs[0]).slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));
        b = fq_ct(public_witness_ct(&builder, fr(uint256_t(inputs[1]).slice(0, fq_ct::NUM_LIMB_BITS * 2))),
                  public_witness_ct(
                      &builder, fr(uint256_t(inputs[1]).slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));
    } else {
        a = fq_ct(
            witness_ct(&builder, fr(uint256_t(inputs[0]).slice(0, fq_ct::NUM_LIMB_BITS * 2))),
            witness_ct(&builder, fr(uint256_t(inputs[0]).slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));
        b = fq_ct(
            witness_ct(&builder, fr(uint256_t(inputs[1]).slice(0, fq_ct::NUM_LIMB_BITS * 2))),
            witness_ct(&builder, fr(uint256_t(inputs[1]).slice(fq_ct::NUM_LIMB_BITS * 2, fq_ct::NUM_LIMB_BITS * 4))));
    }
    builder.set_variable_name(a.binary_basis_limbs[0].element.witness_index, "a_limb_0");
    builder.set_variable_name(a.binary_basis_limbs[1].element.witness_index, "a_limb_1");
    builder.set_variable_name(a.binary_basis_limbs[2].element.witness_index, "a_limb_2");
    builder.set_variable_name(a.binary_basis_limbs[3].element.witness_index, "a_limb_3");

    if (ab) {
        builder.set_variable_name(b.binary_basis_limbs[0].element.witness_index, "b_limb_0");
        builder.set_variable_name(b.binary_basis_limbs[1].element.witness_index, "b_limb_1");
        builder.set_variable_name(b.binary_basis_limbs[2].element.witness_index, "b_limb_2");
        builder.set_variable_name(b.binary_basis_limbs[3].element.witness_index, "b_limb_3");
    }

    fq_ct c;
    if (ab) {
        c = a * b;
    } else {
        c = a * a;
    }
    builder.set_variable_name(c.binary_basis_limbs[0].element.witness_index, "c_limb_0");
    builder.set_variable_name(c.binary_basis_limbs[1].element.witness_index, "c_limb_1");
    builder.set_variable_name(c.binary_basis_limbs[2].element.witness_index, "c_limb_2");
    builder.set_variable_name(c.binary_basis_limbs[3].element.witness_index, "c_limb_3");
    return builder.export_circuit();
}

const std::string q = "21888242871839275222246405745257275088696311157297823662689037894645226208583";

std::vector<FFTerm> correct_result(Circuit<FFTerm>& c, Solver* s)
{
    FFTerm a_limb0 = c["a_limb_0"];
    FFTerm a_limb1 = c["a_limb_1"];
    FFTerm a_limb2 = c["a_limb_2"];
    FFTerm a_limb3 = c["a_limb_3"];

    FFTerm b_limb0 = c["b_limb_0"];
    FFTerm b_limb1 = c["b_limb_1"];
    FFTerm b_limb2 = c["b_limb_2"];
    FFTerm b_limb3 = c["b_limb_3"];

    FFTerm c_limb0 = c["c_limb_0"];
    FFTerm c_limb1 = c["c_limb_1"];
    FFTerm c_limb2 = c["c_limb_2"];
    FFTerm c_limb3 = c["c_limb_3"];

    FFTerm two68 = FFTerm::Const("100000000000000000", s);
    FFTerm two136 = two68 * two68;
    FFTerm two204 = two136 * two68;

    FFTerm a = a_limb0 + two68 * a_limb1 + two136 * a_limb2 + two204 * a_limb3;
    FFTerm b = b_limb0 + two68 * b_limb1 + two136 * b_limb2 + two204 * b_limb3;
    FFTerm cr = c_limb0 + two68 * c_limb1 + two136 * c_limb2 + two204 * c_limb3;
    FFTerm n = FFTerm::Var("n", s);
    FFTerm q_ = FFTerm::Const(q, s, 10); // Const(q_hex, s)
    a* b != cr + n* q_;
    return { cr, n };
}

void model_variables(Circuit<FFTerm>& c, Solver* s, std::vector<FFTerm>& evaluation)
{
    std::unordered_map<std::string, cvc5::Term> terms;
    for (size_t i = 0; i < 4; i++) {
        terms.insert({ "a_limb_" + std::to_string(i), c["a_limb_" + std::to_string(i)] });
        terms.insert({ "b_limb_" + std::to_string(i), c["b_limb_" + std::to_string(i)] });
        terms.insert({ "c_limb_" + std::to_string(i), c["c_limb_" + std::to_string(i)] });
    }
    terms.insert({ "cr", evaluation[0] });
    terms.insert({ "n", evaluation[1] });

    auto values = s->model(terms);

    for (size_t i = 0; i < 4; i++) {
        std::string tmp = "a_limb_" + std::to_string(i);
        info(tmp, " = ", values[tmp]);
    }
    for (size_t i = 0; i < 4; i++) {
        std::string tmp = "b_limb_" + std::to_string(i);
        info(tmp, " = ", values[tmp]);
    }
    for (size_t i = 0; i < 4; i++) {
        std::string tmp = "c_limb_" + std::to_string(i);
        info(tmp, " = ", values[tmp]);
    }
    info("cr = ", values["cr"]);
    info("n = ", values["n"]);
}

void model_variables1(Circuit<FFTerm>& c1, Circuit<FFTerm>& c2, Solver* s)
{
    std::unordered_map<std::string, cvc5::Term> terms;
    for (size_t i = 0; i < 4; i++) {
        terms.insert({ "a_limb_" + std::to_string(i), c1["a_limb_" + std::to_string(i)] });
        terms.insert({ "c1_limb_" + std::to_string(i), c1["c_limb_" + std::to_string(i)] });
        terms.insert({ "c2_limb_" + std::to_string(i), c2["c_limb_" + std::to_string(i)] });
    }
    auto values = s->model(terms);

    for (size_t i = 0; i < 4; i++) {
        std::string tmp = "a_limb_" + std::to_string(i);
        info(tmp, " = ", values[tmp]);
    }

    for (size_t i = 0; i < 4; i++) {
        std::string tmp = "c1_limb_" + std::to_string(i);
        info(tmp, " = ", values[tmp]);
    }

    for (size_t i = 0; i < 4; i++) {
        std::string tmp = "c2_limb_" + std::to_string(i);
        info(tmp, " = ", values[tmp]);
    }
}

TEST(bigfield, multiplication_equal)
{
    bool public_a_b = true;
    bool a_neq_b = true;
    auto buf = create_circuit(public_a_b, a_neq_b);

    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus, config);
    Circuit<FFTerm> circuit(circuit_info, &s);
    std::vector<FFTerm> ev = correct_result(circuit, &s);

    auto start = std::chrono::high_resolution_clock::now();
    bool res = s.check();
    auto stop = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::microseconds>(stop - start);

    info();
    info("Gates: ", circuit.get_num_gates());
    info("Result: ", s.getResult());
    info("Time elapsed: ", static_cast<double>(duration.count()) / 1e6, " sec");

    if (res) {
        model_variables(circuit, &s, ev);
    }
}

TEST(bigfield, unique_square)
{
    auto buf = create_circuit(true, false);

    CircuitSchema circuit_info = unpack_from_buffer(buf);

    Solver s(circuit_info.modulus, config);

    std::pair<Circuit<FFTerm>, Circuit<FFTerm>> cs =
        unique_witness<FFTerm>(circuit_info,
                               &s,
                               { "a_limb_0", "a_limb_1", "a_limb_2", "a_limb_3" },
                               { "c_limb_0", "c_limb_1", "c_limb_2", "c_limb_3" });

    auto start = std::chrono::high_resolution_clock::now();
    bool res = s.check();
    auto stop = std::chrono::high_resolution_clock::now();
    auto duration = std::chrono::duration_cast<std::chrono::microseconds>(stop - start);

    ASSERT_FALSE(res);

    info();
    info("Gates: ", cs.first.get_num_gates());
    info("Result: ", s.getResult());
    info("Time elapsed: ", static_cast<double>(duration.count()) / 1e6, " sec");

    if (res) {
        model_variables1(cs.first, cs.second, &s);
    }
}