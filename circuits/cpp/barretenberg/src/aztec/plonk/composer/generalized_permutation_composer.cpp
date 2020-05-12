#include "generalized_permutation_composer.hpp"
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <numeric/bitop/get_msb.hpp>
#include <plonk/proof_system/widgets/permutation_widget.hpp>
#include <plonk/proof_system/widgets/turbo_arithmetic_widget.hpp>
#include <plonk/proof_system/widgets/turbo_fixed_base_widget.hpp>
#include <plonk/proof_system/widgets/turbo_logic_widget.hpp>
#include <plonk/proof_system/widgets/turbo_range_widget.hpp>
#include <plonk/reference_string/file_reference_string.hpp>
#include <plonk/proof_system/utils/permutation.hpp>

using namespace barretenberg;

namespace waffle {

#define TURBO_SELECTOR_REFS                                                                                            \
    auto& q_m = selectors[TurboSelectors::QM];                                                                         \
    auto& q_c = selectors[TurboSelectors::QC];                                                                         \
    auto& q_1 = selectors[TurboSelectors::Q1];                                                                         \
    auto& q_2 = selectors[TurboSelectors::Q2];                                                                         \
    auto& q_3 = selectors[TurboSelectors::Q3];                                                                         \
    auto& q_4 = selectors[TurboSelectors::Q4];                                                                         \
    auto& q_5 = selectors[TurboSelectors::Q5];                                                                         \
    auto& q_arith = selectors[TurboSelectors::QARITH];                                                                 \
    auto& q_ecc_1 = selectors[TurboSelectors::QECC_1];                                                                 \
    auto& q_range = selectors[TurboSelectors::QRANGE];                                                                 \
    auto& q_logic = selectors[TurboSelectors::QLOGIC];

GenPermComposer::GenPermComposer(const size_t size_hint)
    : TurboComposer(size_hint)
{
    tau.insert({ DUMMY_TAG, DUMMY_TAG });
};

//     void GenPermComposer::assert_equal(const uint32_t a_idx, const uint32_t b_idx)
// {
//     auto& a_cycle = cycle_tags[a_idx];
//     const auto b_cycle = cycle_tags[b_idx];
//     ASSERT(a_cycle == b_cycle || a_cycle == 0 || b_cycle == 0);
//     a_cycle = (a_cycle == 0) ? b_cycle : a_cycle;
//     TurboComposer::assert_equal(a_idx, b_idx);
// }

//  uint32_t GenPermComposer::add_variable(const barretenberg::fr& in)
// {
//     TurboComposer::add_variable(in);
//                     std::cout << "addvar" << std::endl;

//     cycle_tags.push_back(0);
//     tau_of_cycle_tags.push_back(0);
//     return static_cast<uint32_t>(variables.size()) - 1U;
// }

//  uint32_t GenPermComposer::add_public_variable(const barretenberg::fr& in)
// {
//     std::cout << "varsize:" << variables.size() << std::endl;

//     auto index = TurboComposer::add_public_variable(in);
//                     std::cout << "addvarpub" << std::endl;
//     std::cout << "varsize:" << variables.size() << std::endl;
//     cycle_tags.push_back(0);
//     tau_of_cycle_tags.push_back(0);
//     return index;
// }

// GenPermComposer::GenPermComposer()
//     : GenPermComposer("../srs_db", 0)
// {}

// GenPermComposer::GenPermComposer(std::string const& crs_path, const size_t size_hint)
//     : GenPermComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)),
//     size_hint){};

// GenPermComposer::GenPermComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint)
//     : ComposerBase(std::move(crs_factory))
// {
//     w_l.reserve(size_hint);
//     w_r.reserve(size_hint);
//     w_o.reserve(size_hint);
//     w_4.reserve(size_hint);
//     q_m.reserve(size_hint);
//     q_1.reserve(size_hint);
//     q_2.reserve(size_hint);
//     q_3.reserve(size_hint);
//     q_4.reserve(size_hint);
//     q_arith.reserve(size_hint);
//     q_c.reserve(size_hint);
//     q_5.reserve(size_hint);
//     q_ecc_1.reserve(size_hint);
//     q_range.reserve(size_hint);
//     q_logic.reserve(size_hint);

//     zero_idx = put_constant_variable(fr::zero());
//     // zero_idx = add_variable(barretenberg::fr::zero());
// }

// GenPermComposer::GenPermComposer(std::shared_ptr<proving_key> const& p_key,
//                              std::shared_ptr<verification_key> const& v_key,
//                              size_t size_hint)
//     : ComposerBase(p_key, v_key)
// {
//     w_l.reserve(size_hint);
//     w_r.reserve(size_hint);
//     w_o.reserve(size_hint);
//     w_4.reserve(size_hint);
//     q_m.reserve(size_hint);
//     q_1.reserve(size_hint);
//     q_2.reserve(size_hint);
//     q_3.reserve(size_hint);
//     q_4.reserve(size_hint);
//     q_arith.reserve(size_hint);
//     q_c.reserve(size_hint);
//     q_5.reserve(size_hint);
//     q_ecc_1.reserve(size_hint);
//     q_range.reserve(size_hint);
//     q_logic.reserve(size_hint);

//     zero_idx = put_constant_variable(fr::zero());
// };

// void GenPermComposer::create_dummy_gate()
// {
//     gate_flags.push_back(0);
//     uint32_t idx = add_variable(fr{ 1, 1, 1, 1 }.to_montgomery_form());
//     w_l.emplace_back(idx);
//     w_r.emplace_back(idx);
//     w_o.emplace_back(idx);
//     w_4.emplace_back(idx);
//     q_arith.emplace_back(fr::zero());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(fr::zero());
//     q_2.emplace_back(fr::zero());
//     q_3.emplace_back(fr::zero());
//     q_c.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(fourth);

//     ++n;
// }

// void GenPermComposer::create_add_gate(const add_triple& in)
// {
//     gate_flags.push_back(0);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(zero_idx);
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(in.a_scaling);
//     q_2.emplace_back(in.b_scaling);
//     q_3.emplace_back(in.c_scaling);
//     q_c.emplace_back(in.const_scaling);
//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);

//     ++n;
// }

// void GenPermComposer::create_big_add_gate(const add_quad& in)
// {
//     gate_flags.push_back(0);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(in.d);
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(in.a_scaling);
//     q_2.emplace_back(in.b_scaling);
//     q_3.emplace_back(in.c_scaling);
//     q_c.emplace_back(in.const_scaling);
//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(in.d_scaling);
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > in.d);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

//     ++n;
// }

// void GenPermComposer::create_big_add_gate_with_bit_extraction(const add_quad& in)
// {
//     gate_flags.push_back(0);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(in.d);
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(in.a_scaling);
//     q_2.emplace_back(in.b_scaling);
//     q_3.emplace_back(in.c_scaling);
//     q_c.emplace_back(in.const_scaling);
//     q_arith.emplace_back(fr::one() + fr::one());
//     q_4.emplace_back(in.d_scaling);
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > in.d);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

//     ++n;
// }

// void GenPermComposer::create_big_mul_gate(const mul_quad& in)
// {
//     gate_flags.push_back(0);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(in.d);
//     q_m.emplace_back(in.mul_scaling);
//     q_1.emplace_back(in.a_scaling);
//     q_2.emplace_back(in.b_scaling);
//     q_3.emplace_back(in.c_scaling);
//     q_c.emplace_back(in.const_scaling);
//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(in.d_scaling);
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > in.d);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

//     ++n;
// }

// // Creates a width-4 addition gate, where the fourth witness must be a boolean.
// // Can be used to normalize a 32-bit addition
// void GenPermComposer::create_balanced_add_gate(const add_quad& in)
// {
//     gate_flags.push_back(0);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(in.d);
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(in.a_scaling);
//     q_2.emplace_back(in.b_scaling);
//     q_3.emplace_back(in.c_scaling);
//     q_c.emplace_back(in.const_scaling);
//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(in.d_scaling);
//     q_5.emplace_back(fr::one());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > in.d);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

//     ++n;
// }

// void GenPermComposer::create_mul_gate(const mul_triple& in)
// {
//     gate_flags.push_back(0);
//     add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_LEFT_WIRE);
//     add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_RIGHT_WIRE);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(zero_idx);
//     q_m.emplace_back(in.mul_scaling);
//     q_1.emplace_back(fr::zero());
//     q_2.emplace_back(fr::zero());
//     q_3.emplace_back(in.c_scaling);
//     q_c.emplace_back(in.const_scaling);
//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > zero_idx);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);

//     ++n;
// }

// void GenPermComposer::create_bool_gate(const uint32_t variable_index)
// {
//     gate_flags.push_back(0);
//     add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_LEFT_WIRE);
//     add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_RIGHT_WIRE);
//     w_l.emplace_back(variable_index);
//     w_r.emplace_back(variable_index);
//     w_o.emplace_back(variable_index);
//     w_4.emplace_back(zero_idx);
//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());

//     q_m.emplace_back(fr::one());
//     q_1.emplace_back(fr::zero());
//     q_2.emplace_back(fr::zero());
//     q_3.emplace_back(fr::neg_one());
//     q_c.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

//     ASSERT(wire_epicycles.size() > variable_index);
//     wire_epicycles[static_cast<size_t>(variable_index)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(variable_index)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(variable_index)].emplace_back(out);

//     ++n;
// }

// void GenPermComposer::create_poly_gate(const poly_triple& in)
// {
//     gate_flags.push_back(0);
//     add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_LEFT_WIRE);
//     add_gate_flag(gate_flags.size() - 1, GateFlags::FIXED_RIGHT_WIRE);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(zero_idx);
//     q_m.emplace_back(in.q_m);
//     q_1.emplace_back(in.q_l);
//     q_2.emplace_back(in.q_r);
//     q_3.emplace_back(in.q_o);
//     q_c.emplace_back(in.q_c);
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > zero_idx);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);

//     ++n;
// }

// void GenPermComposer::create_fixed_group_add_gate(const fixed_group_add_quad& in)
// {
//     gate_flags.push_back(0);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(in.d);

//     q_arith.emplace_back(fr::zero());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_m.emplace_back(fr::zero());
//     q_c.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     q_1.emplace_back(in.q_x_1);
//     q_2.emplace_back(in.q_x_2);
//     q_3.emplace_back(in.q_y_1);
//     q_ecc_1.emplace_back(in.q_y_2);

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > in.d);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

//     ++n;
// }

// void GenPermComposer::create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in,
//                                                           const fixed_group_init_quad& init)
// {
//     gate_flags.push_back(0);
//     w_l.emplace_back(in.a);
//     w_r.emplace_back(in.b);
//     w_o.emplace_back(in.c);
//     w_4.emplace_back(in.d);

//     q_arith.emplace_back(fr::zero());
//     q_4.emplace_back(init.q_x_1);
//     q_5.emplace_back(init.q_x_2);
//     q_m.emplace_back(init.q_y_1);
//     q_c.emplace_back(init.q_y_2);
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     q_1.emplace_back(in.q_x_1);
//     q_2.emplace_back(in.q_x_2);
//     q_3.emplace_back(in.q_y_1);
//     q_ecc_1.emplace_back(in.q_y_2);

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     ASSERT(wire_epicycles.size() > in.a);
//     ASSERT(wire_epicycles.size() > in.b);
//     ASSERT(wire_epicycles.size() > in.c);
//     ASSERT(wire_epicycles.size() > in.d);

//     wire_epicycles[static_cast<size_t>(in.a)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(in.b)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(in.c)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(in.d)].emplace_back(fourth);

//     ++n;
// }

// void GenPermComposer::fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
// {
//     gate_flags.push_back(0);

//     w_l.emplace_back(witness_index);
//     w_r.emplace_back(zero_idx);
//     w_o.emplace_back(zero_idx);
//     w_4.emplace_back(zero_idx);
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(fr::one());
//     q_2.emplace_back(fr::zero());
//     q_3.emplace_back(fr::zero());
//     q_c.emplace_back(-witness_value);
//     q_arith.emplace_back(fr::one());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };

//     ASSERT(wire_epicycles.size() > witness_index);
//     ASSERT(wire_epicycles.size() > zero_idx);
//     ASSERT(wire_epicycles.size() > zero_idx);
//     wire_epicycles[static_cast<size_t>(witness_index)].emplace_back(left);

//     ++n;
// }

// Check for a sequence of variables that neighboring differences are at most 3 (used for batched range checkj)
void GenPermComposer::create_sort_constraint(const std::vector<uint32_t> variable_index)
{
    TURBO_SELECTOR_REFS
    ASSERT(variable_index.size() % 4 == 0);
    for (size_t i = 0; i < variable_index.size(); i++) {
        ASSERT(static_cast<uint32_t>(variables.size()) > variable_index[i]);
    }

    for (size_t i = 0; i < variable_index.size(); i += 4) {
        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++n;
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
    }
    w_l.emplace_back(variable_index[variable_index.size() - 1]);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
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
    q_range.emplace_back(fr::zero());
}
// std::vector<uint32_t> GenPermComposer::create_range_constraint(const uint32_t witness_index, const size_t num_bits)
// {
//     cycle_tags[witness_index]
//     return accumulators;
// }

std::shared_ptr<proving_key> GenPermComposer::compute_proving_key()
{

    if (circuit_proving_key) {
        return circuit_proving_key;
    }
    ComposerBase::compute_proving_key();
    compute_sigma_permutations<4, true>(circuit_proving_key.get());
    return circuit_proving_key;
}

std::shared_ptr<verification_key> GenPermComposer::compute_verification_key()
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key();
    }
    std::array<fr*, 19> poly_coefficients;
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

    poly_coefficients[15] = circuit_proving_key->id_selectors.at("id_1").get_coefficients();
    poly_coefficients[16] = circuit_proving_key->id_selectors.at("id_2").get_coefficients();
    poly_coefficients[17] = circuit_proving_key->id_selectors.at("id_3").get_coefficients();
    poly_coefficients[18] = circuit_proving_key->id_selectors.at("id_4").get_coefficients();

    scalar_multiplication::pippenger_runtime_state state(circuit_proving_key->n);
    std::vector<barretenberg::g1::affine_element> commitments;
    commitments.resize(19);

    for (size_t i = 0; i < 19; ++i) {
        commitments[i] =
            g1::affine_element(scalar_multiplication::pippenger(poly_coefficients[i],
                                                                circuit_proving_key->reference_string->get_monomials(),
                                                                circuit_proving_key->n,
                                                                state));
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
    circuit_verification_key->id_selectors.insert({ "ID_1", commitments[15] });
    circuit_verification_key->id_selectors.insert({ "ID_2", commitments[16] });
    circuit_verification_key->id_selectors.insert({ "ID_3", commitments[17] });
    circuit_verification_key->id_selectors.insert({ "ID_4", commitments[18] });
    return circuit_verification_key;
}

// std::shared_ptr<program_witness> GenPermComposer::compute_witness()
// {
//     if (computed_witness) {
//         return witness;
//     }
//     const size_t total_num_gates = n + public_inputs.size();
//     size_t log2_n = static_cast<size_t>(numeric::get_msb(total_num_gates + 1));
//     if ((1UL << log2_n) != (total_num_gates + 1)) {
//         ++log2_n;
//     }
//     size_t new_n = 1UL << log2_n;

//     for (size_t i = total_num_gates; i < new_n; ++i) {
//         w_l.emplace_back(zero_idx);
//         w_r.emplace_back(zero_idx);
//         w_o.emplace_back(zero_idx);
//         w_4.emplace_back(zero_idx);
//     }

//     polynomial poly_w_1(new_n);
//     polynomial poly_w_2(new_n);
//     polynomial poly_w_3(new_n);
//     polynomial poly_w_4(new_n);

//     for (size_t i = 0; i < public_inputs.size(); ++i) {
//         fr::__copy(fr::zero(), poly_w_1[i]);
//         fr::__copy(variables[public_inputs[i]], poly_w_2[i]);
//         fr::__copy(fr::zero(), poly_w_3[i]);
//         fr::__copy(fr::zero(), poly_w_4[i]);
//     }
//     for (size_t i = public_inputs.size(); i < new_n; ++i) {
//         fr::__copy(variables[w_l[i - public_inputs.size()]], poly_w_1.at(i));
//         fr::__copy(variables[w_r[i - public_inputs.size()]], poly_w_2.at(i));
//         fr::__copy(variables[w_o[i - public_inputs.size()]], poly_w_3.at(i));
//         fr::__copy(variables[w_4[i - public_inputs.size()]], poly_w_4.at(i));
//     }

//     witness = std::make_shared<program_witness>();
//     witness->wires.insert({ "w_1", std::move(poly_w_1) });
//     witness->wires.insert({ "w_2", std::move(poly_w_2) });
//     witness->wires.insert({ "w_3", std::move(poly_w_3) });
//     witness->wires.insert({ "w_4", std::move(poly_w_4) });

//     computed_witness = true;
//     return witness;
// }

TurboProver GenPermComposer::create_prover()
{
    compute_proving_key();
    compute_witness();

    TurboProver output_state(circuit_proving_key, witness, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get(), witness.get());

    std::unique_ptr<ProverTurboFixedBaseWidget> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverGenPermSortWidget> range_widget =
        std::make_unique<ProverGenPermSortWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget> logic_widget =
        std::make_unique<ProverTurboLogicWidget>(circuit_proving_key.get(), witness.get());

    output_state.widgets.emplace_back(std::move(permutation_widget));
    output_state.widgets.emplace_back(std::move(fixed_base_widget));
    output_state.widgets.emplace_back(std::move(range_widget));
    output_state.widgets.emplace_back(std::move(logic_widget));

    return output_state;
}
// void GenPermComposer::create_dummy_gate()
// {
//     gate_flags.push_back(0);
//     uint32_t idx = add_variable(fr{ 1, 1, 1, 1 }.to_montgomery_form());
//     w_l.emplace_back(idx);
//     w_r.emplace_back(idx);
//     w_o.emplace_back(idx);
//     w_4.emplace_back(idx);
//     q_arith.emplace_back(fr::zero());
//     q_4.emplace_back(fr::zero());
//     q_5.emplace_back(fr::zero());
//     q_ecc_1.emplace_back(fr::zero());
//     q_m.emplace_back(fr::zero());
//     q_1.emplace_back(fr::zero());
//     q_2.emplace_back(fr::zero());
//     q_3.emplace_back(fr::zero());
//     q_c.emplace_back(fr::zero());
//     q_range.emplace_back(fr::zero());
//     q_logic.emplace_back(fr::zero());

//     epicycle left{ static_cast<uint32_t>(n), WireType::LEFT };
//     epicycle right{ static_cast<uint32_t>(n), WireType::RIGHT };
//     epicycle out{ static_cast<uint32_t>(n), WireType::OUTPUT };
//     epicycle fourth{ static_cast<uint32_t>(n), WireType::FOURTH };

//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(left);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(right);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(out);
//     wire_epicycles[static_cast<size_t>(idx)].emplace_back(fourth);

//     ++n;
// }

UnrolledTurboProver GenPermComposer::create_unrolled_prover()
{
    compute_proving_key();
    compute_witness();

    UnrolledTurboProver output_state(circuit_proving_key, witness, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboFixedBaseWidget> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboRangeWidget> range_widget =
        std::make_unique<ProverTurboRangeWidget>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget> logic_widget =
        std::make_unique<ProverTurboLogicWidget>(circuit_proving_key.get(), witness.get());

    output_state.widgets.emplace_back(std::move(permutation_widget));
    output_state.widgets.emplace_back(std::move(fixed_base_widget));
    output_state.widgets.emplace_back(std::move(range_widget));
    output_state.widgets.emplace_back(std::move(logic_widget));

    return output_state;
}

GenPermVerifier GenPermComposer::create_verifier()
{
    compute_verification_key();

    GenPermVerifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    return output_state;
}

UnrolledTurboVerifier GenPermComposer::create_unrolled_verifier()
{
    compute_verification_key();

    UnrolledTurboVerifier output_state(circuit_verification_key, create_unrolled_manifest(public_inputs.size()));

    return output_state;
}

// uint32_t GenPermComposer::put_constant_variable(const barretenberg::fr& variable)
// {
//     if (constant_variables.count(variable) == 1) {
//         return constant_variables.at(variable);
//     } else {
//         uint32_t variable_index = add_variable(variable);
//         fix_witness(variable_index, variable);
//         constant_variables.insert({ variable, variable_index });
//         return variable_index;
//     }
// }

} // namespace waffle