#include <fstream>
#include <iostream>
#include <string>

#include "barretenberg/stdlib/primitives/field/field.hpp"
#include "barretenberg/stdlib/primitives/uint/uint.hpp"
#include "barretenberg/stdlib_circuit_builders/standard_circuit_builder.hpp"

#include "barretenberg/smt_verification/circuit/standard_circuit.hpp"
#include "barretenberg/smt_verification/util/smt_util.hpp"

#include <gtest/gtest.h>

using namespace bb;
using namespace smt_circuit;

namespace {
auto& engine = numeric::get_debug_randomness();
}

using field_t = stdlib::field_t<StandardCircuitBuilder>;
using witness_t = stdlib::witness_t<StandardCircuitBuilder>;
using pub_witness_t = stdlib::public_witness_t<StandardCircuitBuilder>;
using uint_ct = stdlib::uint32<StandardCircuitBuilder>;

TEST(standard_circuit, assert_equal)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();

    field_t a(witness_t(&builder, fr::random_element()));
    field_t b(witness_t(&builder, fr::random_element()));
    builder.set_variable_name(a.witness_index, "a");
    builder.set_variable_name(b.witness_index, "b");
    field_t c = (a + a) / (b + b + b);
    builder.set_variable_name(c.witness_index, "c");

    field_t d(witness_t(&builder, a.get_value()));
    field_t e(witness_t(&builder, b.get_value()));
    field_t f(witness_t(&builder, c.get_value()));
    builder.assert_equal(d.get_witness_index(), a.get_witness_index());
    builder.assert_equal(e.get_witness_index(), b.get_witness_index());

    field_t g = d + d;
    field_t h = e + e + e;
    field_t i = g / h;
    builder.assert_equal(i.get_witness_index(), c.get_witness_index());
    field_t j(witness_t(&builder, i.get_value()));
    field_t k(witness_t(&builder, j.get_value()));
    builder.assert_equal(i.get_witness_index(), j.get_witness_index());
    builder.assert_equal(i.get_witness_index(), k.get_witness_index());

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus);
    StandardCircuit circuit(circuit_info, &s, TermType::FFTerm);

    ASSERT_EQ(circuit[k.get_witness_index()].term, circuit["c"].term);
    ASSERT_EQ(circuit[d.get_witness_index()].term, circuit["a"].term);
    ASSERT_EQ(circuit[e.get_witness_index()].term, circuit["b"].term);

    ASSERT_EQ(circuit[i.get_witness_index()].term, circuit[k.get_witness_index()].term);
    ASSERT_EQ(circuit[i.get_witness_index()].term, circuit[j.get_witness_index()].term);
}

TEST(standard_circuit, cached_subcircuits)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    field_t a(witness_t(&builder, fr::zero()));
    builder.set_variable_name(a.get_witness_index(), "a");
    a.create_range_constraint(5);
    field_t b(witness_t(&builder, fr::zero()));
    b.create_range_constraint(5);
    builder.set_variable_name(b.get_witness_index(), "b");

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus);
    StandardCircuit circuit(circuit_info, &s, TermType::FFITerm);
    s.print_assertions();
}

TEST(standard_circuit, range_relaxation_assertions)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    field_t a(witness_t(&builder, fr(120)));
    a.create_range_constraint(10);

    field_t b(witness_t(&builder, fr(65567)));
    field_t c = a * b;
    c.create_range_constraint(27);
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");
    builder.set_variable_name(c.get_witness_index(), "c");

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus, default_solver_config, 16, 32);
    StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);

    s.print_assertions();
}

TEST(standard_circuit, range_relaxation)
{
    for (size_t i = 2; i < 256; i++) {
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        field_t a(witness_t(&builder, fr::zero()));
        a.create_range_constraint(i);

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus);
        StandardCircuit circuit(circuit_info, &s, TermType::FFITerm);
    }
}

TEST(standard_circuit, xor_relaxation_assertions)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    uint_ct a(witness_t(&builder, static_cast<uint32_t>(fr(120))));
    uint_ct b(witness_t(&builder, static_cast<uint32_t>(fr(120))));
    uint_ct c = a ^ b;
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");
    builder.set_variable_name(c.get_witness_index(), "c");

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus, default_solver_config, 16, 32);
    StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);

    s.print_assertions();
}

TEST(standard_circuit, xor_relaxation)
{
    for (size_t i = 2; i < 256; i += 2) {
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint32_t a_idx = builder.add_variable(0);
        uint32_t b_idx = builder.add_variable(0);
        builder.create_logic_constraint(a_idx, b_idx, i, true);

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 32);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
}

TEST(standard_circuit, and_relaxation_assertions)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    uint_ct a(witness_t(&builder, static_cast<uint32_t>(fr(120))));
    uint_ct b(witness_t(&builder, static_cast<uint32_t>(fr(120))));
    uint_ct c = a & b;
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");
    builder.set_variable_name(c.get_witness_index(), "c");

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus, default_solver_config, 16, 32);
    StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);

    s.print_assertions();
}

TEST(standard_circuit, ror_relaxation_assertions)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    uint_ct a(witness_t(&builder, static_cast<uint32_t>(fr(120))));
    uint_ct b = a.ror(17);
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus, default_solver_config, 16, 32);
    StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);

    s.print_assertions();
}

TEST(standard_circuit, ror_relaxation)
{
    for (size_t i = 1; i < 8; i++) {
        using uint_ct = stdlib::uint8<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a.ror(i);

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 16; i++) {
        using uint_ct = stdlib::uint16<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a.ror(i);

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 32; i++) {
        using uint_ct = stdlib::uint32<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a.ror(i);

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 64; i++) {
        using uint_ct = stdlib::uint64<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a.ror(i);

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
}

TEST(standard_circuit, shl_relaxation_assertions)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    uint_ct a(witness_t(&builder, static_cast<uint32_t>(fr(120))));
    uint_ct b = a << 17;
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus, default_solver_config, 16, 32);
    StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);

    s.print_assertions();
}

TEST(standard_circuit, shl_relaxation)
{
    for (size_t i = 1; i < 8; i++) {
        using uint_ct = stdlib::uint8<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a << i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 16; i++) {
        using uint_ct = stdlib::uint16<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a << i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 32; i++) {
        using uint_ct = stdlib::uint32<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a << i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 64; i++) {
        using uint_ct = stdlib::uint64<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a << i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
}

TEST(standard_circuit, shr_relaxation_assertions)
{
    StandardCircuitBuilder builder = StandardCircuitBuilder();
    uint_ct a(witness_t(&builder, static_cast<uint32_t>(fr(120))));
    uint_ct b = a >> 17;
    builder.set_variable_name(a.get_witness_index(), "a");
    builder.set_variable_name(b.get_witness_index(), "b");

    auto buf = builder.export_circuit();
    CircuitSchema circuit_info = unpack_from_buffer(buf);
    Solver s(circuit_info.modulus, default_solver_config, 16, 32);
    StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);

    s.print_assertions();
}

TEST(standard_circuit, shr_relaxation)
{
    for (size_t i = 1; i < 8; i += 2) {
        using uint_ct = stdlib::uint8<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a >> i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 16; i += 2) {
        using uint_ct = stdlib::uint16<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a >> i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 32; i += 2) {
        using uint_ct = stdlib::uint32<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a >> i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
    for (size_t i = 1; i < 64; i += 2) {
        using uint_ct = stdlib::uint64<StandardCircuitBuilder>;
        StandardCircuitBuilder builder = StandardCircuitBuilder();
        uint_ct a(witness_t(&builder, 0));
        uint_ct b = a >> i;

        auto buf = builder.export_circuit();
        CircuitSchema circuit_info = unpack_from_buffer(buf);
        Solver s(circuit_info.modulus, default_solver_config, 16, 64);
        StandardCircuit circuit(circuit_info, &s, TermType::BVTerm);
    }
}
