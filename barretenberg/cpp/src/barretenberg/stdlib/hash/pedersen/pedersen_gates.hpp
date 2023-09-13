#pragma once
#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/circuit_builders/circuit_builders_fwd.hpp"
#include "../../primitives/field/field.hpp"
#include "../../primitives/point/point.hpp"
#include "barretenberg/proof_system/arithmetization/gate_data.hpp"

namespace proof_system::plonk {
namespace stdlib {

/**
 * @brief Creates constraints required for TurboPlonk pedersen hash algorithm
 * (see https://hackmd.io/@aztec-network/S1mRod9wF?type=view for details)
 *
 * StandardPlonk and UltraPlonk do not have support the custom TurboPlonk pedersen hash gate.
 * This class reduces the TP gate to a sequence of regular arithmetic gates for compatability purposes.
 *
 * N.B. wherever possible, UltraPlonk should use pedersen_plookup as it is MUCH more efficient!
 * pedersen_plookup produces different hash outputs to the TurboPlonk pedersen hash, use this if interoperability
 * between proof systems is required
 * @tparam Composer
 */
template <typename Composer> class pedersen_gates {
  public:
    using FF = typename Composer::FF;
    using fixed_group_add_quad = proof_system::fixed_group_add_quad_<FF>;
    using fixed_group_init_quad = proof_system::fixed_group_init_quad_<FF>;
    using add_quad = proof_system::add_quad_<FF>;

    Composer* context;
    fixed_group_add_quad previous_add_quad;

    pedersen_gates(Composer* input_context = nullptr)
        : context(input_context)
    {}

    void create_fixed_group_add_gate(const fixed_group_add_quad& in)
    {
        if constexpr (std::same_as<Composer, TurboCircuitBuilder>) {
            context->create_fixed_group_add_gate(in);
        } else {

            // TODO: not supported by honk composer?
            // context->assert_valid_variables({ in.a, in.b, in.c, in.d });

            auto row_1 = previous_add_quad;
            auto row_2 = in;
            previous_add_quad = in;

            fr a_1 = context->get_variable(row_1.d);
            fr a_2 = context->get_variable(row_2.d);
            fr x_1 = context->get_variable(row_1.a);
            fr y_1 = context->get_variable(row_1.b);
            fr x_2 = context->get_variable(row_2.a);
            fr y_2 = context->get_variable(row_2.b);
            fr x_alpha = context->get_variable(row_2.c);

            fr q_x_alpha_1 = row_1.q_x_1;
            fr q_x_alpha_2 = row_1.q_x_2;
            fr q_y_alpha_1 = row_1.q_y_1;
            fr q_y_alpha_2 = row_1.q_y_2;

            uint32_t a_1_idx = row_1.d;
            uint32_t a_2_idx = row_2.d;
            uint32_t x_1_idx = row_1.a;
            uint32_t y_1_idx = row_1.b;
            uint32_t x_2_idx = row_2.a;
            uint32_t y_2_idx = row_2.b;
            uint32_t x_alpha_idx = row_2.c;

            // add variable δ = a_2 - 4a_1
            fr delta = a_2 - (a_1 + a_1 + a_1 + a_1);
            uint32_t delta_idx = context->add_variable(delta);
            context->create_add_gate({ .a = a_2_idx,
                                       .b = a_1_idx,
                                       .c = delta_idx,
                                       .a_scaling = 1,
                                       .b_scaling = -4,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            // constraint: (δ + 3)(δ + 1)(δ - 1)(δ - 3)
            // (δ + 3)(δ + 1)(δ - 1)(δ - 3) = (δ^2 - 9)(δ^2 - 1)=0
            // // first: (δ^2 -  δ_sqr = 0)
            fr delta_sqr = delta * delta;
            uint32_t delta_sqr_idx = context->add_variable(delta_sqr);
            context->create_mul_gate({ .a = delta_idx,
                                       .b = delta_idx,
                                       .c = delta_sqr_idx,
                                       .mul_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });
            // // next (δ^2 - 9)( δ^2 - 1) = δ^2*δ^2 - 10 * δ^2 + 9 = 0
            context->create_mul_gate({ .a = delta_sqr_idx,
                                       .b = delta_sqr_idx,
                                       .c = delta_sqr_idx,
                                       .mul_scaling = 1,
                                       .c_scaling = -10,
                                       .const_scaling = 9 });

            // validate correctness of x_ɑ
            // constraint: (δ^2) * q_x_ɑ,1 + q_x_ɑ,2 - x,ɑ = 0
            context->create_add_gate({ .a = delta_sqr_idx,
                                       .b = x_alpha_idx,
                                       .c = context->zero_idx,
                                       .a_scaling = q_x_alpha_1,
                                       .b_scaling = -1,
                                       .c_scaling = 0,
                                       .const_scaling = q_x_alpha_2 });

            // compute y_alpha using lookup formula, instantiate as witness and validate
            fr y_alpha = (x_alpha * q_y_alpha_1 + q_y_alpha_2) * delta;
            uint32_t y_alpha_idx = context->add_variable(y_alpha);
            context->create_poly_gate({ .a = delta_idx,
                                        .b = x_alpha_idx,
                                        .c = y_alpha_idx,
                                        .q_m = q_y_alpha_1,
                                        .q_l = q_y_alpha_2,
                                        .q_r = 0,
                                        .q_o = -1,
                                        .q_c = 0 });

            // show that (x_1, y_1) + (x_ɑ, y_ɑ) = (x_2, y_2) in 11 gates
            // // 4 gates to compute commonly used expressions
            // // // 2 differences:
            fr diff_x_alpha_x_1 = x_alpha - x_1;
            uint32_t diff_x_alpha_x_1_idx = context->add_variable(diff_x_alpha_x_1);
            context->create_add_gate({ .a = diff_x_alpha_x_1_idx,
                                       .b = x_1_idx,
                                       .c = x_alpha_idx,
                                       .a_scaling = 1,
                                       .b_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            fr diff_y_alpha_y_1 = y_alpha - y_1;
            uint32_t diff_y_alpha_y_1_idx = context->add_variable(diff_y_alpha_y_1);
            context->create_add_gate({ .a = diff_y_alpha_y_1_idx,
                                       .b = y_1_idx,
                                       .c = y_alpha_idx,
                                       .a_scaling = 1,
                                       .b_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            // // // now the squares of these 2 differences
            fr diff_x_alpha_x_1_sqr = diff_x_alpha_x_1 * diff_x_alpha_x_1;
            uint32_t diff_x_alpha_x_1_sqr_idx = context->add_variable(diff_x_alpha_x_1_sqr);
            context->create_mul_gate({ .a = diff_x_alpha_x_1_idx,
                                       .b = diff_x_alpha_x_1_idx,
                                       .c = diff_x_alpha_x_1_sqr_idx,
                                       .mul_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            fr diff_y_alpha_y_1_sqr = diff_y_alpha_y_1 * diff_y_alpha_y_1;
            uint32_t diff_y_alpha_y_1_sqr_idx = context->add_variable(diff_y_alpha_y_1_sqr);
            context->create_mul_gate({ .a = diff_y_alpha_y_1_idx,
                                       .b = diff_y_alpha_y_1_idx,
                                       .c = diff_y_alpha_y_1_sqr_idx,
                                       .mul_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            // // 3 gates to build identity for x_2
            // // // compute x_2 + x_ɑ + x_1 using 2 poly_gates via create_big_add_gate
            fr sum_x_1_2_alpha = x_2 + x_alpha + x_1;
            uint32_t sum_x_1_2_alpha_idx = context->add_variable(sum_x_1_2_alpha);
            context->create_big_add_gate({ .a = x_2_idx,
                                           .b = x_alpha_idx,
                                           .c = x_1_idx,
                                           .d = sum_x_1_2_alpha_idx,
                                           .a_scaling = 1,
                                           .b_scaling = 1,
                                           .c_scaling = 1,
                                           .d_scaling = -1,
                                           .const_scaling = 0 });

            // // // constraint: identity for x_2
            context->create_poly_gate({ .a = sum_x_1_2_alpha_idx,
                                        .b = diff_x_alpha_x_1_sqr_idx,
                                        .c = diff_y_alpha_y_1_sqr_idx,
                                        .q_m = 1,
                                        .q_l = 0,
                                        .q_r = 0,
                                        .q_o = -1,
                                        .q_c = 0 });

            // // 4 gates to build identity for y_2:
            // // // 3 auxiliary
            fr sum_y_1_y_2 = y_1 + y_2;
            uint32_t sum_y_1_y_2_idx = context->add_variable(sum_y_1_y_2);
            context->create_add_gate({ .a = y_1_idx,
                                       .b = y_2_idx,
                                       .c = sum_y_1_y_2_idx,
                                       .a_scaling = 1,
                                       .b_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            fr diff_x_1_x_2 = x_1 - x_2;
            uint32_t diff_x_1_x_2_idx = context->add_variable(diff_x_1_x_2);
            context->create_add_gate({ .a = diff_x_1_x_2_idx,
                                       .b = x_2_idx,
                                       .c = x_1_idx,
                                       .a_scaling = 1,
                                       .b_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            fr prod_y_diff_x_diff = diff_y_alpha_y_1 * diff_x_1_x_2;
            uint32_t prod_y_diff_x_diff_idx = context->add_variable(prod_y_diff_x_diff);
            context->create_mul_gate({ .a = diff_y_alpha_y_1_idx,
                                       .b = diff_x_1_x_2_idx,
                                       .c = prod_y_diff_x_diff_idx,
                                       .mul_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });

            // // // identity for y_2
            context->create_mul_gate({ .a = sum_y_1_y_2_idx,
                                       .b = diff_x_alpha_x_1_idx,
                                       .c = prod_y_diff_x_diff_idx,
                                       .mul_scaling = 1,
                                       .c_scaling = -1,
                                       .const_scaling = 0 });
        }
    }

    void create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in, const fixed_group_init_quad& init)
    {
        if constexpr (std::same_as<Composer, TurboCircuitBuilder>) {
            context->create_fixed_group_add_gate_with_init(in, init);
        } else {
            uint32_t x_0_idx = in.a;
            uint32_t y_0_idx = in.b;
            uint32_t x_alpha_idx = in.c;
            uint32_t a_0_idx = in.d;

            fr x_alpha = context->get_variable(x_alpha_idx);
            fr a_0 = context->get_variable(a_0_idx);

            // weird names here follow the Turbo notation
            fr q_4 = init.q_x_1;
            fr q_5 = init.q_x_2;
            fr q_m = init.q_y_1;
            fr q_c = init.q_y_2;

            // We will think of s = 1-a_0 as an auxiliary "switch" which is equal to either -x_alpha or 0
            // during the initialization step, but we will not add this variable to the composer for reasons of
            // efficiency.

            // (ɑ^4 identity) impose 1-a_0 = 0 or -x_alpha
            // // first check formula for sx_alpha
            fr sx_alpha = (fr(1) - a_0) * x_alpha;
            uint32_t sx_alpha_idx = context->add_variable(sx_alpha);
            context->create_poly_gate({ .a = a_0_idx,
                                        .b = x_alpha_idx,
                                        .c = sx_alpha_idx,
                                        .q_m = 1,
                                        .q_l = 0,
                                        .q_r = -1,
                                        .q_o = 1,
                                        .q_c = 0 });

            // // now add the desired constraint on sx_alpha
            // // s(s + x_alpha) = s*s + s*x_alpha = 0
            context->create_poly_gate(
                { .a = a_0_idx, .b = a_0_idx, .c = sx_alpha_idx, .q_m = 1, .q_l = -2, .q_r = 0, .q_o = 1, .q_c = 1 });

            // (ɑ^5 identity)
            context->create_poly_gate({ .a = x_0_idx,
                                        .b = x_alpha_idx,
                                        .c = a_0_idx,
                                        .q_m = -1,
                                        .q_l = 0,
                                        .q_r = q_4,
                                        .q_o = -q_5,
                                        .q_c = q_5 });

            // (ɑ^6 identity)
            context->create_poly_gate({ .a = y_0_idx,
                                        .b = x_alpha_idx,
                                        .c = a_0_idx,
                                        .q_m = -1,
                                        .q_l = 0,
                                        .q_r = q_m,
                                        .q_o = -q_c,
                                        .q_c = q_c });

            // There is no previous add quad.
            previous_add_quad = in;
        }
    }

    void create_fixed_group_add_gate_final(const add_quad& in)
    {
        if constexpr (std::same_as<Composer, TurboCircuitBuilder>) {
            context->create_fixed_group_add_gate_final(in);
        } else {

            fixed_group_add_quad final_round_quad{ .a = in.a,
                                                   .b = in.b,
                                                   .c = in.c,
                                                   .d = in.d,
                                                   .q_x_1 = fr::zero(),
                                                   .q_x_2 = fr::zero(),
                                                   .q_y_1 = fr::zero(),
                                                   .q_y_2 = fr::zero() };
            create_fixed_group_add_gate(final_round_quad);
        }
    }
};

} // namespace stdlib
} // namespace proof_system::plonk