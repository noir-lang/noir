#include "plookup_composer.hpp"
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <numeric/bitop/get_msb.hpp>
#include <plonk/proof_system/widgets/permutation_widget.hpp>
#include <plonk/proof_system/widgets/turbo_arithmetic_widget.hpp>
#include <plonk/proof_system/widgets/turbo_fixed_base_widget.hpp>
#include <plonk/proof_system/widgets/turbo_logic_widget.hpp>
#include <plonk/proof_system/widgets/turbo_range_widget.hpp>
#include <plonk/proof_system/widgets/plookup_widget.hpp>
#include <plonk/reference_string/file_reference_string.hpp>

using namespace barretenberg;

namespace waffle {

PLookupComposer::PLookupComposer()
    : PLookupComposer("../srs_db", 0)
{}

PLookupComposer::PLookupComposer(std::string const& crs_path, const size_t size_hint)
    : PLookupComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)), size_hint){};

PLookupComposer::PLookupComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint)
    : ComposerBase(std::move(crs_factory))
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);
    q_m.reserve(size_hint);
    q_1.reserve(size_hint);
    q_2.reserve(size_hint);
    q_3.reserve(size_hint);
    q_4.reserve(size_hint);
    q_arith.reserve(size_hint);
    q_c.reserve(size_hint);
    q_5.reserve(size_hint);
    q_ecc_1.reserve(size_hint);
    q_range.reserve(size_hint);
    q_logic.reserve(size_hint);
    q_lookup_index.reserve(size_hint);
    q_lookup_type.reserve(size_hint);

    zero_idx = put_constant_variable(fr::zero());
    // zero_idx = add_variable(barretenberg::fr::zero());
}

PLookupComposer::PLookupComposer(std::shared_ptr<proving_key> const& p_key,
                                 std::shared_ptr<verification_key> const& v_key,
                                 size_t size_hint)
    : ComposerBase(p_key, v_key)
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);
    q_m.reserve(size_hint);
    q_1.reserve(size_hint);
    q_2.reserve(size_hint);
    q_3.reserve(size_hint);
    q_4.reserve(size_hint);
    q_arith.reserve(size_hint);
    q_c.reserve(size_hint);
    q_5.reserve(size_hint);
    q_ecc_1.reserve(size_hint);
    q_range.reserve(size_hint);
    q_logic.reserve(size_hint);
    q_lookup_index.reserve(size_hint);
    q_lookup_type.reserve(size_hint);

    zero_idx = put_constant_variable(fr::zero());
};

void PLookupComposer::create_dummy_gate()
{
    gate_flags.push_back(0);
    uint32_t idx = add_variable(fr{ 1, 1, 1, 1 }.to_montgomery_form());
    w_l.emplace_back(idx);
    w_r.emplace_back(idx);
    w_o.emplace_back(idx);
    w_4.emplace_back(idx);
    q_arith.emplace_back(fr::zero());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

    wire_epicycles[static_cast<size_t>(idx)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(idx)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(idx)].emplace_back(out);
    wire_epicycles[static_cast<size_t>(idx)].emplace_back(fourth);

    ++n;
}

void PLookupComposer::create_add_gate(const add_triple& in)
{
    gate_flags.push_back(0);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);

    ++n;
}

void PLookupComposer::create_big_add_gate(const add_quad& in)
{
    gate_flags.push_back(0);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > in.d);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
    wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

    ++n;
}

void PLookupComposer::create_big_add_gate_with_bit_extraction(const add_quad& in)
{
    gate_flags.push_back(0);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(fr::one() + fr::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > in.d);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
    wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

    ++n;
}

void PLookupComposer::create_big_mul_gate(const mul_quad& in)
{
    gate_flags.push_back(0);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > in.d);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
    wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

    ++n;
}

// Creates a width-4 addition gate, where the fourth witness must be a boolean.
// Can be used to normalize a 32-bit addition
void PLookupComposer::create_balanced_add_gate(const add_quad& in)
{
    gate_flags.push_back(0);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(fr::one());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > in.d);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
    wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

    ++n;
}

void PLookupComposer::create_mul_gate(const mul_triple& in)
{
    gate_flags.push_back(0);
    add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_LEFT_WIRE);
    add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_RIGHT_WIRE);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > zero_idx);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);

    ++n;
}

void PLookupComposer::create_bool_gate(const uint32_t variable_index)
{
    gate_flags.push_back(0);
    add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_LEFT_WIRE);
    add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_RIGHT_WIRE);
    w_l.emplace_back(variable_index);
    w_r.emplace_back(variable_index);
    w_o.emplace_back(variable_index);
    w_4.emplace_back(zero_idx);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());

    q_m.emplace_back(fr::one());
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::neg_one());
    q_c.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

    ASSERT(wire_epicycles.size() > variable_index);
    wire_epicycles[static_cast<size_t>(variable_index)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(variable_index)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(variable_index)].emplace_back(out);

    ++n;
}

void PLookupComposer::create_poly_gate(const poly_triple& in)
{
    gate_flags.push_back(0);
    add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_LEFT_WIRE);
    add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_RIGHT_WIRE);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(in.q_m);
    q_1.emplace_back(in.q_l);
    q_2.emplace_back(in.q_r);
    q_3.emplace_back(in.q_o);
    q_c.emplace_back(in.q_c);
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());

    q_arith.emplace_back(fr::one());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > zero_idx);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);

    ++n;
}

void PLookupComposer::create_fixed_group_add_gate(const fixed_group_add_quad& in)
{
    gate_flags.push_back(0);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    q_arith.emplace_back(fr::zero());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_m.emplace_back(fr::zero());
    q_c.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_ecc_1.emplace_back(in.q_y_2);
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > in.d);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
    wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

    ++n;
}

void PLookupComposer::create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in,
                                                            const fixed_group_init_quad& init)
{
    gate_flags.push_back(0);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    q_arith.emplace_back(fr::zero());
    q_4.emplace_back(init.q_x_1);
    q_5.emplace_back(init.q_x_2);
    q_m.emplace_back(init.q_y_1);
    q_c.emplace_back(init.q_y_2);
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_ecc_1.emplace_back(in.q_y_2);
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
    epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

    ASSERT(wire_epicycles.size() > in.a);
    ASSERT(wire_epicycles.size() > in.b);
    ASSERT(wire_epicycles.size() > in.c);
    ASSERT(wire_epicycles.size() > in.d);

    wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
    wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

    ++n;
}

void PLookupComposer::fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
{
    gate_flags.push_back(0);

    w_l.emplace_back(witness_index);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::one());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(-witness_value);
    q_arith.emplace_back(fr::one());
    q_4.emplace_back(fr::zero());
    q_5.emplace_back(fr::zero());
    q_ecc_1.emplace_back(fr::zero());
    q_range.emplace_back(fr::zero());
    q_logic.emplace_back(fr::zero());
    q_lookup_index.emplace_back(fr::zero());
    q_lookup_type.emplace_back(fr::zero());

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };

    ASSERT(wire_epicycles.size() > witness_index);
    ASSERT(wire_epicycles.size() > zero_idx);
    ASSERT(wire_epicycles.size() > zero_idx);
    wire_epicycles[static_cast<size_t>(witness_index)].emplace_back(left);

    ++n;
}

std::vector<uint32_t> PLookupComposer::create_range_constraint(const uint32_t witness_index, const size_t num_bits)
{
    ASSERT(static_cast<uint32_t>(variables.size()) > witness_index);
    ASSERT(((num_bits >> 1U) << 1U) == num_bits);

    /*
     * The range constraint accumulates base 4 values into a sum.
     * We do this by evaluating a kind of 'raster scan', where we compare adjacent elements
     * and validate that their differences map to a base for value  *
     * Let's say that we want to perform a 32-bit range constraint in 'x'.
     * We can represent x via 16 constituent base-4 'quads' {q_0, ..., q_15}:
     *
     *      15
     *      ===
     *      \          i
     * x =  /    q  . 4
     *      ===   i
     *     i = 0
     *
     * In program memory, we place an accumulating base-4 sum of x {a_0, ..., a_15}, where
     *
     *         i
     *        ===
     *        \                  j
     * a   =  /    q         .  4
     *  i     ===   (15 - j)
     *       j = 0
     *
     *
     * From this, we can use our range transition constraint to validate that
     *
     *
     *  a      - 4 . a  ϵ [0, 1, 2, 3]
     *   i + 1        i
     *
     *
     * We place our accumulating sums in program memory in the following sequence:
     *
     * +-----+-----+-----+-----+
     * |  A  |  B  |  C  |  D  |
     * +-----+-----+-----+-----+
     * | a3  | a2  | a1  | 0   |
     * | a7  | a6  | a5  | a4  |
     * | a11 | a10 | a9  | a8  |
     * | a15 | a14 | a13 | a12 |
     * | --- | --- | --- | a16 |
     * +-----+-----+-----+-----+
     *
     * Our range transition constraint on row 'i'
     * performs our base-4 range check on the follwing pairs:
     *
     * (D_{i}, C_{i}), (C_{i}, B_{i}), (B_{i}, A_{i}), (A_{i}, D_{i+1})
     *
     * We need to start our raster scan at zero, so we simplify matters and just force the first value
     * to be zero.
     *
     * The output will be in the 4th column of an otherwise unused row. Assuming this row can
     * be used for a width-3 standard gate, the total number of gates for an n-bit range constraint
     * is (n / 8) gates
     *
     **/

    const fr witness_value = variables[witness_index].from_montgomery_form();

    // one gate accmulates 4 quads, or 8 bits.
    // # gates = (bits / 8)
    size_t num_quad_gates = (num_bits >> 3);

    num_quad_gates = (num_quad_gates << 3 == num_bits) ? num_quad_gates : num_quad_gates + 1;

    // hmm
    std::vector<uint32_t>* wires[4]{ &w_4, &w_o, &w_r, &w_l };

    // hmmm
    WireType wire_types[4]{ WireType::FOURTH, WireType::OUTPUT, WireType::RIGHT, WireType::LEFT };

    const size_t num_quads = (num_quad_gates << 2);
    const size_t forced_zero_threshold = 1 + (((num_quads << 1) - num_bits) >> 1);
    std::vector<uint32_t> accumulators;
    fr accumulator = fr::zero();

    for (size_t i = 0; i < num_quads + 1; ++i) {
        const size_t gate_index = n + (i / 4);
        uint32_t accumulator_index;
        if (i < forced_zero_threshold) {
            accumulator_index = zero_idx;
        } else {
            const size_t bit_index = (num_quads - i) << 1;
            const uint64_t quad = static_cast<uint64_t>(witness_value.get_bit(bit_index)) +
                                  2ULL * static_cast<uint64_t>(witness_value.get_bit(bit_index + 1));
            const fr quad_element = fr{ quad, 0, 0, 0 }.to_montgomery_form();
            accumulator += accumulator;
            accumulator += accumulator;
            accumulator += quad_element;

            accumulator_index = add_variable(accumulator);
            accumulators.emplace_back(accumulator_index);
        }

        // hmmmm
        (*(wires + (i & 3)))->emplace_back(accumulator_index);
        const size_t wire_index = i & 3;

        wire_epicycles[accumulator_index].emplace_back(
            epicycle(static_cast<uint32_t>(gate_index), wire_types[wire_index]));
    }
    size_t used_gates = (num_quads + 1) / 4;

    // TODO: handle partially used gates. For now just set them to be zero
    if (used_gates * 4 != (num_quads + 1)) {
        ++used_gates;
    }

    for (size_t i = 0; i < used_gates; ++i) {
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_c.emplace_back(fr::zero());
        q_arith.emplace_back(fr::zero());
        q_4.emplace_back(fr::zero());
        q_5.emplace_back(fr::zero());
        q_ecc_1.emplace_back(fr::zero());
        q_logic.emplace_back(fr::zero());
        q_range.emplace_back(fr::one());
        q_lookup_index.emplace_back(fr::zero());
        q_lookup_type.emplace_back(fr::zero());
    }

    q_range[q_range.size() - 1] = fr::zero();

    w_l.emplace_back(zero_idx);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);

    assert_equal(accumulators[accumulators.size() - 1], witness_index);
    accumulators[accumulators.size() - 1] = witness_index;

    n += used_gates;
    return accumulators;
}

waffle::accumulator_triple PLookupComposer::create_logic_constraint(const uint32_t a,
                                                                    const uint32_t b,
                                                                    const size_t num_bits,
                                                                    const bool is_xor_gate)
{
    ASSERT(static_cast<uint32_t>(variables.size()) > a);
    ASSERT(static_cast<uint32_t>(variables.size()) > b);
    ASSERT(((num_bits >> 1U) << 1U) == num_bits); // no odd number of bits! bad! only quads!

    /*
     * The LOGIC constraint accumulates 3 base-4 values (a, b, c) into a sum, where c = a & b OR c = a ^ b
     *
     * In program memory, we place an accumulating base-4 sum of a, b, c {a_0, ..., a_15}, where
     *
     *         i
     *        ===
     *        \                  j
     * a   =  /    q         .  4
     *  i     ===   (15 - j)
     *       j = 0
     *
     *
     * From this, we can use our logic transition constraint to validate that
     *
     *
     *  a      - 4 . a  ϵ [0, 1, 2, 3]
     *   i + 1        i
     *
     *
     *
     *
     *  b      - 4 . b  ϵ [0, 1, 2, 3]
     *   i + 1        i
     *
     *
     *
     *
     *                    /                 \          /                 \
     *  c      - 4 . c  = | a      - 4 . a  | (& OR ^) | b      - b . a  |
     *   i + 1        i   \  i + 1        i /          \  i + 1        i /
     *
     *
     * We also need the following temporary, w, stored in program memory:
     *
     *      /                 \   /                 \
     * w  = | a      - 4 . a  | * | b      - b . a  |
     *  i   \  i + 1        i /   \  i + 1        i /
     *
     *
     * w is needed to prevent the degree of our quotient polynomial from blowing up
     *
     * We place our accumulating sums in program memory in the following sequence:
     *
     * +-----+-----+-----+-----+
     * |  A  |  B  |  C  |  D  |
     * +-----+-----+-----+-----+
     * | 0   | 0   | w1  | 0   |
     * | a1  | b1  | w2  | c1  |
     * | a2  | b2  | w3  | c2  |
     * |  :  |  :  |  :  |  :  |
     * | an  | bn  | --- | cn  |
     * +-----+-----+-----+-----+
     *
     * Our transition constraint extracts quads by taking the difference between two accumulating sums,
     * so we need to start the chain with a row of zeroes
     *
     * The total number of gates required to evaluate an AND operation is (n / 2) + 1,
     * where n = max(num_bits(a), num_bits(b))
     *
     * One additional benefit of this constraint, is that both our inputs and output are in 'native' uint32 form.
     * This means we *never* have to decompose a uint32 into bits and back in order to chain together
     * addition and logic operations.
     *
     **/

    const fr left_witness_value = variables[a].from_montgomery_form();
    const fr right_witness_value = variables[b].from_montgomery_form();

    // one gate accmulates 1 quads, or 2 bits.
    // # gates = (bits / 2)
    const size_t num_quads = (num_bits >> 1);

    waffle::accumulator_triple accumulators;
    fr left_accumulator = fr::zero();
    fr right_accumulator = fr::zero();
    fr out_accumulator = fr::zero();

    // Step 1: populare 1st row accumulators with zero
    w_l.emplace_back(zero_idx);
    w_r.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);

    wire_epicycles[zero_idx].emplace_back(epicycle(static_cast<uint32_t>(n), WireType::LEFT));
    wire_epicycles[zero_idx].emplace_back(epicycle(static_cast<uint32_t>(n), WireType::RIGHT));
    wire_epicycles[zero_idx].emplace_back(epicycle(static_cast<uint32_t>(n), WireType::FOURTH));

    // w_l, w_r, w_4 should now point to 1 gate ahead of w_o
    for (size_t i = 0; i < num_quads; ++i) {
        const size_t gate_index = n + i + 1;
        uint32_t left_accumulator_index;
        uint32_t right_accumulator_index;
        uint32_t out_accumulator_index;
        uint32_t product_index;

        const size_t bit_index = (num_quads - 1 - i) << 1;
        const uint64_t left_quad = static_cast<uint64_t>(left_witness_value.get_bit(bit_index)) +
                                   2ULL * static_cast<uint64_t>(left_witness_value.get_bit(bit_index + 1));

        const uint64_t right_quad = static_cast<uint64_t>(right_witness_value.get_bit(bit_index)) +
                                    2ULL * static_cast<uint64_t>(right_witness_value.get_bit(bit_index + 1));
        const fr left_quad_element = fr{ left_quad, 0, 0, 0 }.to_montgomery_form();
        const fr right_quad_element = fr{ right_quad, 0, 0, 0 }.to_montgomery_form();
        fr out_quad_element;
        if (is_xor_gate) {
            out_quad_element = fr{ left_quad ^ right_quad, 0, 0, 0 }.to_montgomery_form();
        } else {
            out_quad_element = fr{ left_quad & right_quad, 0, 0, 0 }.to_montgomery_form();
        }

        const fr product_quad_element = fr{ left_quad * right_quad, 0, 0, 0 }.to_montgomery_form();

        left_accumulator += left_accumulator;
        left_accumulator += left_accumulator;
        left_accumulator += left_quad_element;

        right_accumulator += right_accumulator;
        right_accumulator += right_accumulator;
        right_accumulator += right_quad_element;

        out_accumulator += out_accumulator;
        out_accumulator += out_accumulator;
        out_accumulator += out_quad_element;

        left_accumulator_index = add_variable(left_accumulator);
        accumulators.left.emplace_back(left_accumulator_index);

        right_accumulator_index = add_variable(right_accumulator);
        accumulators.right.emplace_back(right_accumulator_index);

        out_accumulator_index = add_variable(out_accumulator);
        accumulators.out.emplace_back(out_accumulator_index);

        product_index = add_variable(product_quad_element);

        w_l.emplace_back(left_accumulator_index);
        w_r.emplace_back(right_accumulator_index);
        w_4.emplace_back(out_accumulator_index);
        w_o.emplace_back(product_index);

        wire_epicycles[left_accumulator_index].emplace_back(
            epicycle(static_cast<uint32_t>(gate_index), WireType::LEFT));
        wire_epicycles[right_accumulator_index].emplace_back(
            epicycle(static_cast<uint32_t>(gate_index), WireType::RIGHT));
        wire_epicycles[out_accumulator_index].emplace_back(
            epicycle(static_cast<uint32_t>(gate_index), WireType::FOURTH));
        wire_epicycles[product_index].emplace_back(epicycle(static_cast<uint32_t>(gate_index - 1), WireType::OUTPUT));
    }

    w_o.emplace_back(zero_idx);

    for (size_t i = 0; i < num_quads + 1; ++i) {
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_arith.emplace_back(fr::zero());
        q_4.emplace_back(fr::zero());
        q_5.emplace_back(fr::zero());
        q_ecc_1.emplace_back(fr::zero());
        q_range.emplace_back(fr::zero());
        if (is_xor_gate) {
            q_c.emplace_back(fr::neg_one());
            q_logic.emplace_back(fr::neg_one());
        } else {
            q_c.emplace_back(fr::one());
            q_logic.emplace_back(fr::one());
        }
        q_lookup_index.emplace_back(fr::zero());
        q_lookup_type.emplace_back(fr::zero());
    }
    q_c[q_c.size() - 1] = fr::zero();         // last gate is a noop
    q_logic[q_logic.size() - 1] = fr::zero(); // last gate is a noop

    assert_equal(accumulators.left[accumulators.left.size() - 1], a);
    accumulators.left[accumulators.left.size() - 1] = a;

    assert_equal(accumulators.right[accumulators.right.size() - 1], b);
    accumulators.right[accumulators.right.size() - 1] = b;

    n += (num_quads + 1);
    return accumulators;
}

waffle::accumulator_triple PLookupComposer::create_and_constraint(const uint32_t a,
                                                                  const uint32_t b,
                                                                  const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, false);
}

waffle::accumulator_triple PLookupComposer::create_xor_constraint(const uint32_t a,
                                                                  const uint32_t b,
                                                                  const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, true);
}

uint32_t PLookupComposer::put_constant_variable(const barretenberg::fr& variable)
{
    if (constant_variables.count(variable) == 1) {
        return constant_variables.at(variable);
    } else {
        uint32_t variable_index = add_variable(variable);
        fix_witness(variable_index, variable);
        constant_variables.insert({ variable, variable_index });
        return variable_index;
    }
}

void PLookupComposer::add_lookup_selector(polynomial& small, const std::string& tag)
{
    polynomial lagrange_base(small, circuit_proving_key->small_domain.size + 1);
    small.ifft(circuit_proving_key->small_domain);
    polynomial large(small, circuit_proving_key->n * 4 + 4);
    large.coset_fft(circuit_proving_key->large_domain);

    large.add_lagrange_base_coefficient(large[0]);
    large.add_lagrange_base_coefficient(large[1]);
    large.add_lagrange_base_coefficient(large[2]);
    large.add_lagrange_base_coefficient(large[3]);

    circuit_proving_key->permutation_selectors.insert({ tag, std::move(small) });
    circuit_proving_key->permutation_selectors_lagrange_base.insert({ tag, std::move(lagrange_base) });
    circuit_proving_key->permutation_selector_ffts.insert({ tag + "_fft", std::move(large) });
}

std::shared_ptr<proving_key> PLookupComposer::compute_proving_key()
{
    if (computed_proving_key) {
        return circuit_proving_key;
    }
    create_dummy_gate();
    ASSERT(wire_epicycles.size() == variables.size());
    ASSERT(n == q_m.size());
    ASSERT(n == q_1.size());
    ASSERT(n == q_2.size());
    ASSERT(n == q_3.size());
    ASSERT(n == q_3.size());
    ASSERT(n == q_4.size());
    ASSERT(n == q_5.size());
    ASSERT(n == q_arith.size());
    ASSERT(n == q_ecc_1.size());
    ASSERT(n == q_range.size());
    ASSERT(n == q_logic.size());
    ASSERT(n == q_lookup_index.size());
    ASSERT(n == q_lookup_type.size());

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }
    const size_t total_num_gates = std::max(n + public_inputs.size(), tables_size + lookups_size);

    size_t log2_n = static_cast<size_t>(numeric::get_msb(total_num_gates + 1));
    if ((1UL << log2_n) != (total_num_gates + 1)) {
        ++log2_n;
    }
    size_t new_n = 1UL << log2_n;

    for (size_t i = total_num_gates; i < new_n; ++i) {
        q_m.emplace_back(fr::zero());
        q_1.emplace_back(fr::zero());
        q_2.emplace_back(fr::zero());
        q_3.emplace_back(fr::zero());
        q_c.emplace_back(fr::zero());
        q_4.emplace_back(fr::zero());
        q_5.emplace_back(fr::zero());
        q_arith.emplace_back(fr::zero());
        q_ecc_1.emplace_back(fr::zero());
        q_range.emplace_back(fr::zero());
        q_logic.emplace_back(fr::zero());
        q_lookup_index.emplace_back(fr::zero());
        q_lookup_type.emplace_back(fr::zero());
    }

    for (size_t i = 0; i < public_inputs.size(); ++i) {
        epicycle left{ static_cast<uint32_t>(i - public_inputs.size()), WireType::LEFT };
        epicycle right{ static_cast<uint32_t>(i - public_inputs.size()), WireType::RIGHT };

        std::vector<epicycle>& old_epicycles = wire_epicycles[static_cast<size_t>(public_inputs[i])];

        std::vector<epicycle> new_epicycles;

        new_epicycles.emplace_back(left);
        new_epicycles.emplace_back(right);
        for (size_t i = 0; i < old_epicycles.size(); ++i) {
            new_epicycles.emplace_back(old_epicycles[i]);
        }
        old_epicycles = new_epicycles;
    }
    auto crs = crs_factory_->get_prover_crs(new_n);
    circuit_proving_key = std::make_shared<proving_key>(new_n, public_inputs.size(), crs);

    polynomial poly_q_m(new_n);
    polynomial poly_q_c(new_n);
    polynomial poly_q_1(new_n);
    polynomial poly_q_2(new_n);
    polynomial poly_q_3(new_n);
    polynomial poly_q_4(new_n);
    polynomial poly_q_5(new_n);
    polynomial poly_q_arith(new_n);
    polynomial poly_q_ecc_1(new_n);
    polynomial poly_q_range(new_n);
    polynomial poly_q_logic(new_n);
    polynomial poly_q_lookup_index(new_n + 1);
    polynomial poly_q_lookup_type(new_n + 1);

    for (size_t i = 0; i < public_inputs.size(); ++i) {
        poly_q_m[i] = fr::zero();
        poly_q_1[i] = fr::one();
        poly_q_2[i] = fr::zero();
        poly_q_3[i] = fr::zero();
        poly_q_4[i] = fr::zero();
        poly_q_5[i] = fr::zero();
        poly_q_arith[i] = fr::zero();
        poly_q_ecc_1[i] = fr::zero();
        poly_q_c[i] = fr::zero();
        poly_q_range[i] = fr::zero();
        poly_q_logic[i] = fr::zero();
        poly_q_lookup_index[i] = fr::zero();
        poly_q_lookup_type[i] = fr::zero();
    }

    for (size_t i = public_inputs.size(); i < new_n; ++i) {
        poly_q_m[i] = q_m[i - public_inputs.size()];
        poly_q_1[i] = q_1[i - public_inputs.size()];
        poly_q_2[i] = q_2[i - public_inputs.size()];
        poly_q_3[i] = q_3[i - public_inputs.size()];
        poly_q_c[i] = q_c[i - public_inputs.size()];
        poly_q_4[i] = q_4[i - public_inputs.size()];
        poly_q_5[i] = q_5[i - public_inputs.size()];
        poly_q_arith[i] = q_arith[i - public_inputs.size()];
        poly_q_ecc_1[i] = q_ecc_1[i - public_inputs.size()];
        poly_q_range[i] = q_range[i - public_inputs.size()];
        poly_q_logic[i] = q_logic[i - public_inputs.size()];
        poly_q_lookup_index[i] = q_lookup_index[i - public_inputs.size()];
        poly_q_lookup_type[i] = q_lookup_type[i - public_inputs.size()];
    }

    add_selector(poly_q_1, "q_1");
    add_selector(poly_q_2, "q_2");
    add_selector(poly_q_3, "q_3");
    add_selector(poly_q_4, "q_4");
    add_selector(poly_q_5, "q_5");
    add_selector(poly_q_m, "q_m");
    add_selector(poly_q_c, "q_c");
    add_selector(poly_q_ecc_1, "q_ecc_1");
    add_selector(poly_q_range, "q_range");
    add_selector(poly_q_logic, "q_logic");

    polynomial poly_q_table_1(new_n + 1);
    polynomial poly_q_table_2(new_n + 1);
    polynomial poly_q_table_3(new_n + 1);
    polynomial poly_q_table_4(new_n + 1);
    size_t offset = new_n - tables_size;

    for (size_t i = 0; i < offset; ++i) {
        poly_q_table_1[i] = fr::zero();
        poly_q_table_2[i] = fr::zero();
        poly_q_table_3[i] = fr::zero();
        poly_q_table_4[i] = fr::zero();
    }

    for (const auto& table : lookup_tables) {
        const fr table_index(table.table_index);
        for (size_t i = 0; i < table.size; ++i) {
            poly_q_table_1[offset] = table.column_1[i];
            poly_q_table_2[offset] = table.column_2[i];
            poly_q_table_3[offset] = table.column_3[i];
            poly_q_table_4[offset] = table_index;
            ++offset;
        }
    }

    add_lookup_selector(poly_q_table_1, "table_value_1");
    add_lookup_selector(poly_q_table_2, "table_value_2");
    add_lookup_selector(poly_q_table_3, "table_value_3");
    add_lookup_selector(poly_q_table_4, "table_value_4");
    add_lookup_selector(poly_q_lookup_index, "table_index");
    add_lookup_selector(poly_q_lookup_type, "table_type");

    polynomial z_lookup_fft(new_n * 4 + 4, new_n * 4 + 4);
    polynomial s_fft(new_n * 4 + 4, new_n * 4 + 4);
    circuit_proving_key->wire_ffts.insert({ "z_lookup_fft", std::move(z_lookup_fft) });
    circuit_proving_key->wire_ffts.insert({ "s_fft", std::move(s_fft) });

    auto& lookup_mapping = circuit_proving_key->lookup_mapping;
    auto& table_indices = circuit_proving_key->table_indices;

    lookup_mapping.resize(new_n);
    table_indices.resize(new_n);
    for (size_t i = 0; i < new_n; ++i) {
        lookup_mapping[i] = LookupType::NONE;
    }

    for (const auto& table : lookup_tables) {
        for (const auto& lookup_entry : table.lookup_gates) {
            lookup_mapping[lookup_entry.first] = lookup_entry.second;
            table_indices[lookup_entry.first] = table.table_index;
        }
    }

    circuit_proving_key->num_lookup_tables = lookup_tables.size();
    circuit_proving_key->lookup_table_step_size = plookup_step_size;

    compute_sigma_permutations<4>(circuit_proving_key.get());
    computed_proving_key = true;
    return circuit_proving_key;
}

std::shared_ptr<verification_key> PLookupComposer::compute_verification_key()
{
    if (computed_verification_key) {
        return circuit_verification_key;
    }
    if (!computed_proving_key) {
        compute_proving_key();
    }

    std::array<fr*, 21> poly_coefficients;
    poly_coefficients[0] = circuit_proving_key->constraint_selectors.at("q_1").get_coefficients();
    poly_coefficients[1] = circuit_proving_key->constraint_selectors.at("q_2").get_coefficients();
    poly_coefficients[2] = circuit_proving_key->constraint_selectors.at("q_3").get_coefficients();
    poly_coefficients[3] = circuit_proving_key->constraint_selectors.at("q_4").get_coefficients();
    poly_coefficients[4] = circuit_proving_key->constraint_selectors.at("q_5").get_coefficients();
    poly_coefficients[5] = circuit_proving_key->constraint_selectors.at("q_m").get_coefficients();
    poly_coefficients[6] = circuit_proving_key->constraint_selectors.at("q_c").get_coefficients();
    poly_coefficients[7] = circuit_proving_key->constraint_selectors.at("q_arith").get_coefficients();
    poly_coefficients[8] = circuit_proving_key->constraint_selectors.at("q_ecc_1").get_coefficients();
    poly_coefficients[9] = circuit_proving_key->constraint_selectors.at("q_range").get_coefficients();
    poly_coefficients[10] = circuit_proving_key->constraint_selectors.at("q_logic").get_coefficients();

    poly_coefficients[11] = circuit_proving_key->permutation_selectors.at("sigma_1").get_coefficients();
    poly_coefficients[12] = circuit_proving_key->permutation_selectors.at("sigma_2").get_coefficients();
    poly_coefficients[13] = circuit_proving_key->permutation_selectors.at("sigma_3").get_coefficients();
    poly_coefficients[14] = circuit_proving_key->permutation_selectors.at("sigma_4").get_coefficients();

    poly_coefficients[15] = circuit_proving_key->permutation_selectors.at("table_value_1").get_coefficients();
    poly_coefficients[16] = circuit_proving_key->permutation_selectors.at("table_value_2").get_coefficients();
    poly_coefficients[17] = circuit_proving_key->permutation_selectors.at("table_value_3").get_coefficients();
    poly_coefficients[18] = circuit_proving_key->permutation_selectors.at("table_value_4").get_coefficients();
    poly_coefficients[19] = circuit_proving_key->permutation_selectors.at("table_value_index").get_coefficients();
    poly_coefficients[20] = circuit_proving_key->permutation_selectors.at("table_value_type").get_coefficients();

    std::vector<barretenberg::g1::affine_element> commitments;
    commitments.resize(21);

    for (size_t i = 0; i < 21; ++i) {
        commitments[i] =
            g1::affine_element(scalar_multiplication::pippenger(poly_coefficients[i],
                                                                circuit_proving_key->reference_string->get_monomials(),
                                                                circuit_proving_key->n,
                                                                circuit_proving_key->pippenger_runtime_state));
    }

    auto crs = crs_factory_->get_verifier_crs();
    circuit_verification_key =
        std::make_shared<verification_key>(circuit_proving_key->n, circuit_proving_key->num_public_inputs, crs);

    circuit_verification_key->constraint_selectors.insert({ "Q_1", commitments[0] });
    circuit_verification_key->constraint_selectors.insert({ "Q_2", commitments[1] });
    circuit_verification_key->constraint_selectors.insert({ "Q_3", commitments[2] });
    circuit_verification_key->constraint_selectors.insert({ "Q_4", commitments[3] });
    circuit_verification_key->constraint_selectors.insert({ "Q_5", commitments[4] });
    circuit_verification_key->constraint_selectors.insert({ "Q_M", commitments[5] });
    circuit_verification_key->constraint_selectors.insert({ "Q_C", commitments[6] });
    circuit_verification_key->constraint_selectors.insert({ "Q_ARITHMETIC_SELECTOR", commitments[7] });
    circuit_verification_key->constraint_selectors.insert({ "Q_FIXED_BASE_SELECTOR", commitments[8] });
    circuit_verification_key->constraint_selectors.insert({ "Q_RANGE_SELECTOR", commitments[9] });
    circuit_verification_key->constraint_selectors.insert({ "Q_LOGIC_SELECTOR", commitments[10] });

    circuit_verification_key->permutation_selectors.insert({ "SIGMA_1", commitments[11] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_2", commitments[12] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_3", commitments[13] });
    circuit_verification_key->permutation_selectors.insert({ "SIGMA_4", commitments[14] });

    circuit_verification_key->permutation_selectors.insert({ "TABLE_0", commitments[15] });
    circuit_verification_key->permutation_selectors.insert({ "TABLE_1", commitments[16] });
    circuit_verification_key->permutation_selectors.insert({ "TABLE_2", commitments[17] });
    circuit_verification_key->permutation_selectors.insert({ "TABLE_3", commitments[18] });

    circuit_verification_key->permutation_selectors.insert({ "TABLE_INDEX", commitments[19] });
    circuit_verification_key->permutation_selectors.insert({ "TABLE_TYPE", commitments[20] });

    computed_verification_key = true;
    return circuit_verification_key;
}

std::shared_ptr<program_witness> PLookupComposer::compute_witness()
{
    if (computed_witness) {
        return witness;
    }

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }
    const size_t total_num_gates = std::max(n + public_inputs.size(), tables_size + lookups_size);

    size_t log2_n = static_cast<size_t>(numeric::get_msb(total_num_gates + 1));
    if ((1UL << log2_n) != (total_num_gates + 1)) {
        ++log2_n;
    }
    size_t new_n = 1UL << log2_n;

    for (size_t i = total_num_gates; i < new_n; ++i) {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        w_o.emplace_back(zero_idx);
        w_4.emplace_back(zero_idx);
    }

    polynomial poly_w_1(new_n);
    polynomial poly_w_2(new_n);
    polynomial poly_w_3(new_n);
    polynomial poly_w_4(new_n);
    polynomial s(new_n + 1);
    polynomial z_lookup(new_n + 1);
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        fr::__copy(fr::zero(), poly_w_1[i]);
        fr::__copy(variables[public_inputs[i]], poly_w_2[i]);
        fr::__copy(fr::zero(), poly_w_3[i]);
        fr::__copy(fr::zero(), poly_w_4[i]);
    }
    for (size_t i = public_inputs.size(); i < new_n; ++i) {
        fr::__copy(variables[w_l[i - public_inputs.size()]], poly_w_1.at(i));
        fr::__copy(variables[w_r[i - public_inputs.size()]], poly_w_2.at(i));
        fr::__copy(variables[w_o[i - public_inputs.size()]], poly_w_3.at(i));
        fr::__copy(variables[w_4[i - public_inputs.size()]], poly_w_4.at(i));
    }

    witness = std::make_shared<program_witness>();
    witness->wires.insert({ "w_1", std::move(poly_w_1) });
    witness->wires.insert({ "w_2", std::move(poly_w_2) });
    witness->wires.insert({ "w_3", std::move(poly_w_3) });
    witness->wires.insert({ "w_4", std::move(poly_w_4) });
    witness->wires.insert({ "s", std::move(s) });
    witness->wires.insert({ "z_lookup", std::move(z_lookup) });

    computed_witness = true;
    return witness;
}

PLookupProver PLookupComposer::create_prover()
{
    compute_proving_key();
    compute_witness();

    PLookupProver output_state(circuit_proving_key, witness, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverPLookupWidget> plookup_widget =
        std::make_unique<ProverPLookupWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboFixedBaseWidget> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboRangeWidget> range_widget =
        std::make_unique<ProverTurboRangeWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget> logic_widget =
        std::make_unique<ProverTurboLogicWidget>(circuit_proving_key.get(), witness.get());

    output_state.widgets.emplace_back(std::move(permutation_widget));
    output_state.widgets.emplace_back(std::move(plookup_widget));
    output_state.widgets.emplace_back(std::move(fixed_base_widget));
    output_state.widgets.emplace_back(std::move(range_widget));
    output_state.widgets.emplace_back(std::move(logic_widget));

    return output_state;
}

UnrolledPLookupProver PLookupComposer::create_unrolled_prover()
{
    compute_proving_key();
    compute_witness();

    UnrolledPLookupProver output_state(circuit_proving_key, witness, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverPLookupWidget> plookup_widget =
        std::make_unique<ProverPLookupWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboFixedBaseWidget> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboRangeWidget> range_widget =
        std::make_unique<ProverTurboRangeWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget> logic_widget =
        std::make_unique<ProverTurboLogicWidget>(circuit_proving_key.get(), witness.get());

    output_state.widgets.emplace_back(std::move(permutation_widget));
    output_state.widgets.emplace_back(std::move(plookup_widget));
    output_state.widgets.emplace_back(std::move(fixed_base_widget));
    output_state.widgets.emplace_back(std::move(range_widget));
    output_state.widgets.emplace_back(std::move(logic_widget));

    return output_state;
}

PLookupVerifier PLookupComposer::create_verifier()
{
    compute_verification_key();

    PLookupVerifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    return output_state;
}

UnrolledPLookupVerifier PLookupComposer::create_unrolled_verifier()
{
    compute_verification_key();

    UnrolledPLookupVerifier output_state(circuit_verification_key, create_unrolled_manifest(public_inputs.size()));

    return output_state;
}

void PLookupComposer::initialize_precomputed_table(const LookupTableId id,
                                                   const size_t size,
                                                   void (*generator)(size_t,
                                                                     std::vector<barretenberg::fr>&,
                                                                     std::vector<barretenberg::fr>&,
                                                                     std::vector<barretenberg::fr>&))
{
    for (auto table : lookup_tables) {
        ASSERT(table.id != id);
    }
    LookupTable new_table;
    new_table.id = id;
    new_table.table_index = lookup_tables.size() + 1;
    new_table.size = size;
    generator(size, new_table.column_1, new_table.column_2, new_table.column_3);

    lookup_tables.emplace_back(new_table);
}

PLookupComposer::LookupTable& PLookupComposer::get_table(const LookupTableId id)
{
    for (LookupTable& table : lookup_tables) {
        if (table.id == id) {
            return table;
        }
    }
    throw;
}

uint32_t PLookupComposer::read_from_table(const LookupTableId id, const std::pair<uint32_t, uint32_t> key)
{
    LookupTable& table = get_table(id);
    const uint64_t table_bits = numeric::get_msb(table.size) / 2;
    const uint64_t table_step = 1ULL << table_bits;
    const uint256_t left_val = variables[key.first];
    const uint256_t right_val = variables[key.second];
    const size_t index = static_cast<size_t>(left_val.data[0]) + static_cast<size_t>(right_val.data[0]) * table_step;

    ASSERT(table.column_1[index] == variables[key.first]);
    ASSERT(table.column_2[index] == variables[key.second]);
    const fr value = table.column_3[index];

    uint32_t value_index = add_variable(value);

    table.lookup_gates.push_back(std::make_pair(static_cast<uint32_t>(n), LookupType::ABSOLUTE_LOOKUP));

    q_lookup_type.emplace_back(fr::one());
    q_lookup_index.emplace_back(fr(table.table_index));
    w_l.emplace_back(key.first);
    w_r.emplace_back(key.second);
    w_o.emplace_back(value_index);
    w_4.emplace_back(zero_idx);
    q_1.emplace_back(fr(0));
    q_2.emplace_back(fr(0));
    q_3.emplace_back(fr(0));
    q_c.emplace_back(fr(0));
    q_arith.emplace_back(fr(0));
    q_4.emplace_back(fr(0));
    q_5.emplace_back(fr(0));
    q_ecc_1.emplace_back(fr(0));
    q_range.emplace_back(fr(0));
    q_logic.emplace_back(fr(0));

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

    ASSERT(wire_epicycles.size() > key.first);
    ASSERT(wire_epicycles.size() > key.second);
    ASSERT(wire_epicycles.size() > value_index);

    wire_epicycles[static_cast<size_t>(key.first)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(key.second)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(value_index)].emplace_back(out);

    ++n;

    return value_index;
}

std::pair<uint32_t, uint32_t> PLookupComposer::read_from_table(const LookupTableId id, const uint32_t key)
{
    LookupTable& table = get_table(id);

    const uint256_t left_val = variables[key];
    const size_t index = static_cast<size_t>(left_val.data[0]);

    ASSERT(table.column_1[index] == variables[key]);
    const fr value_1 = table.column_2[index];
    const fr value_2 = table.column_3[index];

    const auto value_indices = std::make_pair(add_variable(value_1), add_variable(value_2));

    table.lookup_gates.push_back(std::make_pair(static_cast<uint32_t>(n), LookupType::ABSOLUTE_LOOKUP));

    q_lookup_type.emplace_back(fr::one());
    q_lookup_index.emplace_back(fr(table.table_index));
    w_l.emplace_back(key);
    w_r.emplace_back(value_indices.first);
    w_o.emplace_back(value_indices.second);
    w_4.emplace_back(zero_idx);
    q_1.emplace_back(fr(0));
    q_2.emplace_back(fr(0));
    q_3.emplace_back(fr(0));
    q_c.emplace_back(fr(0));
    q_arith.emplace_back(fr(0));
    q_4.emplace_back(fr(0));
    q_5.emplace_back(fr(0));
    q_ecc_1.emplace_back(fr(0));
    q_range.emplace_back(fr(0));
    q_logic.emplace_back(fr(0));

    epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
    epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
    epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

    ASSERT(wire_epicycles.size() > key);
    ASSERT(wire_epicycles.size() > value_indices.first);
    ASSERT(wire_epicycles.size() > value_indices.second);

    wire_epicycles[static_cast<size_t>(key)].emplace_back(left);
    wire_epicycles[static_cast<size_t>(value_indices.first)].emplace_back(right);
    wire_epicycles[static_cast<size_t>(value_indices.second)].emplace_back(out);

    ++n;

    return value_indices;
}

std::vector<uint32_t> PLookupComposer::read_sequence_from_table(const LookupTableId id,
                                                                const std::vector<std::pair<uint32_t, uint32_t>>& keys)
{
    std::vector<uint32_t> value_indices;
    LookupTable& table = get_table(id);

    const size_t num_lookups = keys.size();

    std::vector<fr> lookup_values;
    for (size_t i = 0; i < num_lookups; ++i) {
        uint256_t left_val;
        uint256_t right_val;
        if (i < num_lookups - 1) {
            left_val = (variables[keys[i].first] - variables[keys[i + 1].first] * plookup_step_size);
            right_val = (variables[keys[i].second] - variables[keys[i + 1].second] * plookup_step_size);
        } else {
            left_val = variables[keys[i].first];
            right_val = variables[keys[i].second];
        }

        const uint64_t table_bits = numeric::get_msb(table.size) / 2;
        const uint64_t table_step = 1ULL << table_bits;

        const size_t index =
            static_cast<size_t>(left_val.data[0]) + static_cast<size_t>(right_val.data[0]) * table_step;

        ASSERT(table.column_1[index] == left_val);
        ASSERT(table.column_2[index] == right_val);

        const fr value = table.column_3[index];
        lookup_values.emplace_back(value);
    }
    for (size_t i = num_lookups - 2; i < num_lookups; --i) {
        lookup_values[i] += plookup_step_size * lookup_values[i + 1];
    }

    for (size_t i = 0; i < num_lookups; ++i) {
        uint32_t value_index = add_variable(lookup_values[i]);

        if (i < num_lookups - 1) {
            table.lookup_gates.push_back(std::make_pair(static_cast<uint32_t>(n), LookupType::RELATIVE_LOOKUP));
            q_lookup_type.emplace_back(fr(2));
        } else {
            table.lookup_gates.push_back(std::make_pair(static_cast<uint32_t>(n), LookupType::ABSOLUTE_LOOKUP));
            q_lookup_type.emplace_back(fr(1));
        }
        q_lookup_index.emplace_back(fr(table.table_index));
        w_l.emplace_back(keys[i].first);
        w_r.emplace_back(keys[i].second);
        w_o.emplace_back(value_index);
        w_4.emplace_back(zero_idx);
        q_1.emplace_back(fr(0));
        q_2.emplace_back(fr(0));
        q_3.emplace_back(fr(0));
        q_c.emplace_back(fr(0));
        q_arith.emplace_back(fr(0));
        q_4.emplace_back(fr(0));
        q_5.emplace_back(fr(0));
        q_ecc_1.emplace_back(fr(0));
        q_range.emplace_back(fr(0));
        q_logic.emplace_back(fr(0));

        epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
        epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
        epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

        ASSERT(wire_epicycles.size() > keys[i].first);
        ASSERT(wire_epicycles.size() > keys[i].second);
        ASSERT(wire_epicycles.size() > value_index);

        wire_epicycles[static_cast<size_t>(keys[i].first)].emplace_back(left);
        wire_epicycles[static_cast<size_t>(keys[i].second)].emplace_back(right);
        wire_epicycles[static_cast<size_t>(value_index)].emplace_back(out);

        ++n;
        value_indices.push_back(value_index);
    }

    return value_indices;
}

} // namespace waffle