#include "standard_circuit_constructor.hpp"
#include <unordered_set>
#include <unordered_map>

using namespace barretenberg;

namespace waffle {
#define STANDARD_SELECTOR_REFS                                                                                         \
    auto& q_m = selectors[StandardSelectors::QM];                                                                      \
    auto& q_c = selectors[StandardSelectors::QC];                                                                      \
    auto& q_1 = selectors[StandardSelectors::Q1];                                                                      \
    auto& q_2 = selectors[StandardSelectors::Q2];                                                                      \
    auto& q_3 = selectors[StandardSelectors::Q3];

/**
 * Create an addition gate.
 *
 * @param in An add_triple containing the indexes of variables to be placed into the
 * wires w_l, w_r, w_o and addition coefficients to be placed into q_1, q_2, q_3, q_c.
 */
void StandardCircuitConstructor::create_add_gate(const add_triple& in)
{
    STANDARD_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);

    ++n;
}

/**
 * Create a big addition gate.
 * (a*a_c + b*b_c + c*c_c + d*d_c + q_c = 0)
 *
 * @param in An add quad containing the indexes of variables a, b, c, d and
 * the scaling factors.
 * */
void StandardCircuitConstructor::create_big_add_gate(const add_quad& in)
{
    // (a terms + b terms = temp)
    // (c terms + d  terms + temp = 0 )
    fr t0 = get_variable(in.a) * in.a_scaling;
    fr t1 = get_variable(in.b) * in.b_scaling;
    fr temp = t0 + t1;
    uint32_t temp_idx = add_variable(temp);

    create_add_gate(add_triple{ in.a, in.b, temp_idx, in.a_scaling, in.b_scaling, fr::neg_one(), fr::zero() });

    create_add_gate(add_triple{ in.c, in.d, temp_idx, in.c_scaling, in.d_scaling, fr::one(), in.const_scaling });
}

/**
 * Create a balanced addition gate.
 * (a*a_c + b*b_c + c*c_c + d*d_c + q_c = 0, where d is in [0,3])
 *
 * @param in An add quad containing the indexes of variables a, b, c, d and
 * the scaling factors.
 * */
void StandardCircuitConstructor::create_balanced_add_gate(const add_quad& in)
{

    STANDARD_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c, in.d });

    // (a terms + b terms = temp)
    // (c terms + d  terms + temp = 0 )
    fr t0 = get_variable(in.a) * in.a_scaling;
    fr t1 = get_variable(in.b) * in.b_scaling;
    fr temp = t0 + t1;
    uint32_t temp_idx = add_variable(temp);

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(temp_idx);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(fr::neg_one());
    q_c.emplace_back(fr::zero());

    ++n;

    w_l.emplace_back(temp_idx);
    w_r.emplace_back(in.c);
    w_o.emplace_back(in.d);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::one());
    q_2.emplace_back(in.c_scaling);
    q_3.emplace_back(in.d_scaling);
    q_c.emplace_back(in.const_scaling);

    ++n;

    // in.d must be between 0 and 3
    // i.e. in.d * (in.d - 1) * (in.d - 2) = 0
    fr temp_2 = get_variable(in.d).sqr() - get_variable(in.d);
    uint32_t temp_2_idx = add_variable(temp_2);
    w_l.emplace_back(in.d);
    w_r.emplace_back(in.d);
    w_o.emplace_back(temp_2_idx);
    q_m.emplace_back(fr::one());
    q_1.emplace_back(fr::neg_one());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::neg_one());
    q_c.emplace_back(fr::zero());

    ++n;

    constexpr fr neg_two = -fr(2);
    w_l.emplace_back(temp_2_idx);
    w_r.emplace_back(in.d);
    w_o.emplace_back(zero_idx);
    q_m.emplace_back(fr::one());
    q_1.emplace_back(neg_two);
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(fr::zero());

    ++n;
}

void StandardCircuitConstructor::create_big_add_gate_with_bit_extraction(const add_quad& in)
{
    // blah.
    // delta = (c - 4d)
    // delta^2 = c^2 + 16d^2 - 8dc
    // r = (-2*delta*delta + 9*delta - 7)*delta
    // r =

    fr delta = get_variable(in.d);
    delta += delta;
    delta += delta;
    delta = get_variable(in.c) - delta;

    uint32_t delta_idx = add_variable(delta);
    constexpr fr neg_four = -(fr(4));
    create_add_gate(add_triple{ in.c, in.d, delta_idx, fr::one(), neg_four, fr::neg_one(), fr::zero() });

    constexpr fr two = fr(2);
    constexpr fr seven = fr(7);
    constexpr fr nine = fr(9);
    const fr r_0 = (delta * nine) - ((delta.sqr() * two) + seven);
    uint32_t r_0_idx = add_variable(r_0);
    create_poly_gate(poly_triple{ delta_idx, delta_idx, r_0_idx, -two, nine, fr::zero(), fr::neg_one(), -seven });

    fr r_1 = r_0 * delta;
    uint32_t r_1_idx = add_variable(r_1);
    create_mul_gate(mul_triple{
        r_0_idx,
        delta_idx,
        r_1_idx,
        fr::one(),
        fr::neg_one(),
        fr::zero(),
    });

    // ain.a1 + bin.b2 + cin.c3 + din.c4 + r_1 = 0

    fr r_2 = (r_1 + (get_variable(in.d) * in.d_scaling));
    uint32_t r_2_idx = add_variable(r_2);
    create_add_gate(add_triple{ in.d, r_1_idx, r_2_idx, in.d_scaling, fr::one(), fr::neg_one(), fr::zero() });

    create_big_add_gate(
        add_quad{ in.a, in.b, in.c, r_2_idx, in.a_scaling, in.b_scaling, in.c_scaling, fr::one(), in.const_scaling });
}

void StandardCircuitConstructor::create_big_mul_gate(const mul_quad& in)
{
    fr temp = ((get_variable(in.c) * in.c_scaling) + (get_variable(in.d) * in.d_scaling));
    uint32_t temp_idx = add_variable(temp);
    create_add_gate(add_triple{ in.c, in.d, temp_idx, in.c_scaling, in.d_scaling, fr::neg_one(), fr::zero() });

    create_poly_gate(
        poly_triple{ in.a, in.b, temp_idx, in.mul_scaling, in.a_scaling, in.b_scaling, fr::one(), in.const_scaling });
}

/**
 * Create a multiplication gate.
 *
 * @param in A mul_tripple containing the indexes of variables to be placed into the
 * wires w_l, w_r, w_o and scaling coefficients to be placed into q_m, q_3, q_c.
 */
void StandardCircuitConstructor::create_mul_gate(const mul_triple& in)
{
    STANDARD_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);

    ++n;
}

/**
 * Create a bool gate.
 * This gate constrains a variable to two possible values: 0 or 1.
 *
 * @param variable_index The index of the variable.
 */
void StandardCircuitConstructor::create_bool_gate(const uint32_t variable_index)
{
    STANDARD_SELECTOR_REFS
    assert_valid_variables({ variable_index });

    w_l.emplace_back(variable_index);
    w_r.emplace_back(variable_index);
    w_o.emplace_back(variable_index);

    q_m.emplace_back(fr::one());
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::neg_one());
    q_c.emplace_back(fr::zero());

    ++n;
}

/**
 * Create a gate where you set all the indexes and coefficients yourself.
 *
 * @param in A poly_triple containing all the information.
 */
void StandardCircuitConstructor::create_poly_gate(const poly_triple& in)
{
    STANDARD_SELECTOR_REFS
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    q_m.emplace_back(in.q_m);
    q_1.emplace_back(in.q_l);
    q_2.emplace_back(in.q_r);
    q_3.emplace_back(in.q_o);
    q_c.emplace_back(in.q_c);

    ++n;
}

void StandardCircuitConstructor::create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in,
                                                                       const fixed_group_init_quad& init)
{
    uint32_t x_0_idx = in.a;
    uint32_t y_0_idx = in.b;
    uint32_t x_alpha_idx = in.c;
    uint32_t a_0_idx = in.d;

    fr x_alpha = get_variable(x_alpha_idx);
    fr a_0 = get_variable(a_0_idx);

    // weird names here follow the Turbo notation
    fr q_4 = init.q_x_1;
    fr q_5 = init.q_x_2;
    fr q_m = init.q_y_1;
    fr q_c = init.q_y_2;

    // We will think of s = 1-a_0 as an auxiliary "switch" which is equal to either -x_alpha or 0
    // during the initialization step, but we will not add this variable to the composer for reasons of efficiency.

    // (ɑ^4 identity) impose 1-a_0 = 0 or -x_alpha
    // // first check formula for sx_alpha
    fr sx_alpha = (fr(1) - a_0) * x_alpha;
    uint32_t sx_alpha_idx = add_variable(sx_alpha);
    create_poly_gate(
        { .a = a_0_idx, .b = x_alpha_idx, .c = sx_alpha_idx, .q_m = 1, .q_l = 0, .q_r = -1, .q_o = 1, .q_c = 0 });

    // // now add the desired constraint on sx_alpha
    // // s(s + x_alpha) = s*s + s*x_alpha = 0
    create_poly_gate(
        { .a = a_0_idx, .b = a_0_idx, .c = sx_alpha_idx, .q_m = 1, .q_l = -2, .q_r = 0, .q_o = 1, .q_c = 1 });

    // (ɑ^5 identity)
    create_poly_gate(
        { .a = x_0_idx, .b = x_alpha_idx, .c = a_0_idx, .q_m = -1, .q_l = 0, .q_r = q_4, .q_o = -q_5, .q_c = q_5 });

    // (ɑ^6 identity)
    create_poly_gate(
        { .a = y_0_idx, .b = x_alpha_idx, .c = a_0_idx, .q_m = -1, .q_l = 0, .q_r = q_m, .q_o = -q_c, .q_c = q_c });

    // There is no previous add quad.
    previous_add_quad = in;
}

void StandardCircuitConstructor::create_fixed_group_add_gate(const fixed_group_add_quad& in)
{
    assert_valid_variables({ in.a, in.b, in.c, in.d });

    auto row_1 = previous_add_quad;
    auto row_2 = in;
    previous_add_quad = in;

    fr a_1 = get_variable(row_1.d);
    fr a_2 = get_variable(row_2.d);
    fr x_1 = get_variable(row_1.a);
    fr y_1 = get_variable(row_1.b);
    fr x_2 = get_variable(row_2.a);
    fr y_2 = get_variable(row_2.b);
    fr x_alpha = get_variable(row_2.c);

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
    uint32_t delta_idx = add_variable(delta);
    create_add_gate({ .a = a_2_idx,
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
    uint32_t delta_sqr_idx = add_variable(delta_sqr);
    create_mul_gate(
        { .a = delta_idx, .b = delta_idx, .c = delta_sqr_idx, .mul_scaling = 1, .c_scaling = -1, .const_scaling = 0 });
    // // next (δ^2 - 9)( δ^2 - 1) = δ^2*δ^2 - 10 * δ^2 + 9 = 0
    create_mul_gate({ .a = delta_sqr_idx,
                      .b = delta_sqr_idx,
                      .c = delta_sqr_idx,
                      .mul_scaling = 1,
                      .c_scaling = -10,
                      .const_scaling = 9 });

    // validate correctness of x_ɑ
    // constraint: (δ^2) * q_x_ɑ,1 + q_x_ɑ,2 - x,ɑ = 0
    create_add_gate({ .a = delta_sqr_idx,
                      .b = x_alpha_idx,
                      .c = zero_idx,
                      .a_scaling = q_x_alpha_1,
                      .b_scaling = -1,
                      .c_scaling = 0,
                      .const_scaling = q_x_alpha_2 });

    // compute y_alpha using lookup formula, instantiate as witness and validate
    fr y_alpha = (x_alpha * q_y_alpha_1 + q_y_alpha_2) * delta;
    uint32_t y_alpha_idx = add_variable(y_alpha);
    create_poly_gate({ .a = delta_idx,
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
    uint32_t diff_x_alpha_x_1_idx = add_variable(diff_x_alpha_x_1);
    create_add_gate({ .a = diff_x_alpha_x_1_idx,
                      .b = x_1_idx,
                      .c = x_alpha_idx,
                      .a_scaling = 1,
                      .b_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });

    fr diff_y_alpha_y_1 = y_alpha - y_1;
    uint32_t diff_y_alpha_y_1_idx = add_variable(diff_y_alpha_y_1);
    create_add_gate({ .a = diff_y_alpha_y_1_idx,
                      .b = y_1_idx,
                      .c = y_alpha_idx,
                      .a_scaling = 1,
                      .b_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });

    // // // now the squares of these 2 differences
    fr diff_x_alpha_x_1_sqr = diff_x_alpha_x_1 * diff_x_alpha_x_1;
    uint32_t diff_x_alpha_x_1_sqr_idx = add_variable(diff_x_alpha_x_1_sqr);
    create_mul_gate({ .a = diff_x_alpha_x_1_idx,
                      .b = diff_x_alpha_x_1_idx,
                      .c = diff_x_alpha_x_1_sqr_idx,
                      .mul_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });

    fr diff_y_alpha_y_1_sqr = diff_y_alpha_y_1 * diff_y_alpha_y_1;
    uint32_t diff_y_alpha_y_1_sqr_idx = add_variable(diff_y_alpha_y_1_sqr);
    create_mul_gate({ .a = diff_y_alpha_y_1_idx,
                      .b = diff_y_alpha_y_1_idx,
                      .c = diff_y_alpha_y_1_sqr_idx,
                      .mul_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });

    // // 3 gates to build identity for x_2
    // // // compute x_2 + x_ɑ + x_1 using 2 poly_gates via create_big_add_gate
    fr sum_x_1_2_alpha = x_2 + x_alpha + x_1;
    uint32_t sum_x_1_2_alpha_idx = add_variable(sum_x_1_2_alpha);
    create_big_add_gate({ .a = x_2_idx,
                          .b = x_alpha_idx,
                          .c = x_1_idx,
                          .d = sum_x_1_2_alpha_idx,
                          .a_scaling = 1,
                          .b_scaling = 1,
                          .c_scaling = 1,
                          .d_scaling = -1,
                          .const_scaling = 0 });

    // // // constraint: identity for x_2
    create_poly_gate({ .a = sum_x_1_2_alpha_idx,
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
    uint32_t sum_y_1_y_2_idx = add_variable(sum_y_1_y_2);
    create_add_gate({ .a = y_1_idx,
                      .b = y_2_idx,
                      .c = sum_y_1_y_2_idx,
                      .a_scaling = 1,
                      .b_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });

    fr diff_x_1_x_2 = x_1 - x_2;
    uint32_t diff_x_1_x_2_idx = add_variable(diff_x_1_x_2);
    create_add_gate({ .a = diff_x_1_x_2_idx,
                      .b = x_2_idx,
                      .c = x_1_idx,
                      .a_scaling = 1,
                      .b_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });

    fr prod_y_diff_x_diff = diff_y_alpha_y_1 * diff_x_1_x_2;
    uint32_t prod_y_diff_x_diff_idx = add_variable(prod_y_diff_x_diff);
    create_mul_gate({ .a = diff_y_alpha_y_1_idx,
                      .b = diff_x_1_x_2_idx,
                      .c = prod_y_diff_x_diff_idx,
                      .mul_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });

    // // // identity for y_2
    create_mul_gate({ .a = sum_y_1_y_2_idx,
                      .b = diff_x_alpha_x_1_idx,
                      .c = prod_y_diff_x_diff_idx,
                      .mul_scaling = 1,
                      .c_scaling = -1,
                      .const_scaling = 0 });
}

void StandardCircuitConstructor::create_fixed_group_add_gate_final(const add_quad& in)
{
    waffle::fixed_group_add_quad final_round_quad{ .a = in.a,
                                                   .b = in.b,
                                                   .c = in.c,
                                                   .d = in.d,
                                                   .q_x_1 = fr::zero(),
                                                   .q_x_2 = fr::zero(),
                                                   .q_y_1 = fr::zero(),
                                                   .q_y_2 = fr::zero() };
    create_fixed_group_add_gate(final_round_quad);
}

std::vector<uint32_t> StandardCircuitConstructor::decompose_into_base4_accumulators(const uint32_t witness_index,
                                                                                    const size_t num_bits,
                                                                                    std::string const& msg)
{
    ASSERT(num_bits > 0);
    const uint256_t target(get_variable(witness_index));

    std::vector<uint32_t> accumulators;

    size_t num_quads = (num_bits >> 1);
    num_quads = (num_quads << 1 == num_bits) ? num_quads : num_quads + 1;
    const auto is_edge_case = [&num_quads, &num_bits](size_t idx) {
        return (idx == num_quads - 1 && ((num_bits & 1ULL) == 1ULL));
    };
    constexpr fr four = fr{ 4, 0, 0, 0 }.to_montgomery_form();
    fr accumulator = fr::zero();
    uint32_t accumulator_idx = 0;
    for (size_t i = num_quads - 1; i < num_quads; --i) {

        bool lo = target.get_bit(2 * i);
        uint32_t lo_idx = add_variable(lo ? fr::one() : fr::zero());
        create_bool_gate(lo_idx);

        uint32_t quad_idx;

        if (is_edge_case(i)) {
            quad_idx = lo_idx;
        } else {
            bool hi = target.get_bit(2 * i + 1);
            uint32_t hi_idx = add_variable(hi ? fr::one() : fr::zero());
            create_bool_gate(hi_idx);

            uint64_t quad = (lo ? 1U : 0U) + (hi ? 2U : 0U);
            quad_idx = add_variable(fr{ quad, 0, 0, 0 }.to_montgomery_form());

            create_add_gate(
                add_triple{ lo_idx, hi_idx, quad_idx, fr::one(), fr::one() + fr::one(), fr::neg_one(), fr::zero() });
        }

        if (i == num_quads - 1) {
            accumulators.push_back(quad_idx);
            accumulator = get_variable(quad_idx);
            accumulator_idx = quad_idx;
        } else {
            fr new_accumulator = accumulator + accumulator;
            new_accumulator = new_accumulator + new_accumulator;
            new_accumulator = new_accumulator + get_variable(quad_idx);
            uint32_t new_accumulator_idx = add_variable(new_accumulator);
            create_add_gate(add_triple{
                accumulator_idx, quad_idx, new_accumulator_idx, four, fr::one(), fr::neg_one(), fr::zero() });
            accumulators.push_back(new_accumulator_idx);
            accumulator = new_accumulator;
            accumulator_idx = new_accumulator_idx;
        }
    }

    assert_equal(witness_index, accumulator_idx, msg);
    return accumulators;
}

waffle::accumulator_triple StandardCircuitConstructor::create_logic_constraint(const uint32_t a,
                                                                               const uint32_t b,
                                                                               const size_t num_bits,
                                                                               const bool is_xor_gate)
{
    assert_valid_variables({ a, b });

    waffle::accumulator_triple accumulators;

    const uint256_t left_witness_value(get_variable(a));
    const uint256_t right_witness_value(get_variable(b));

    fr left_accumulator = fr::zero();
    fr right_accumulator = fr::zero();
    fr out_accumulator = fr::zero();

    uint32_t left_accumulator_idx = zero_idx;
    uint32_t right_accumulator_idx = zero_idx;
    uint32_t out_accumulator_idx = zero_idx;
    constexpr fr four = fr(4);
    constexpr fr neg_two = -fr(2);
    for (size_t i = num_bits - 1; i < num_bits; i -= 2) {
        bool left_hi_val = left_witness_value.get_bit(i);
        bool left_lo_val = left_witness_value.get_bit(i - 1);
        bool right_hi_val = right_witness_value.get_bit((i));
        bool right_lo_val = right_witness_value.get_bit(i - 1);

        uint32_t left_hi_idx = add_variable(left_hi_val ? fr::one() : fr::zero());
        uint32_t left_lo_idx = add_variable(left_lo_val ? fr::one() : fr::zero());
        uint32_t right_hi_idx = add_variable(right_hi_val ? fr::one() : fr::zero());
        uint32_t right_lo_idx = add_variable(right_lo_val ? fr::one() : fr::zero());

        bool out_hi_val = is_xor_gate ? left_hi_val ^ right_hi_val : left_hi_val & right_hi_val;
        bool out_lo_val = is_xor_gate ? left_lo_val ^ right_lo_val : left_lo_val & right_lo_val;

        uint32_t out_hi_idx = add_variable(out_hi_val ? fr::one() : fr::zero());
        uint32_t out_lo_idx = add_variable(out_lo_val ? fr::one() : fr::zero());

        create_bool_gate(left_hi_idx);
        create_bool_gate(right_hi_idx);
        create_bool_gate(out_hi_idx);

        create_bool_gate(left_lo_idx);
        create_bool_gate(right_lo_idx);
        create_bool_gate(out_lo_idx);

        // a & b = ab
        // a ^ b = a + b - ab
        create_poly_gate(poly_triple{ left_hi_idx,
                                      right_hi_idx,
                                      out_hi_idx,
                                      is_xor_gate ? neg_two : fr::one(),
                                      is_xor_gate ? fr::one() : fr::zero(),
                                      is_xor_gate ? fr::one() : fr::zero(),
                                      fr::neg_one(),
                                      fr::zero() });

        create_poly_gate(poly_triple{ left_lo_idx,
                                      right_lo_idx,
                                      out_lo_idx,
                                      is_xor_gate ? neg_two : fr::one(),
                                      is_xor_gate ? fr::one() : fr::zero(),
                                      is_xor_gate ? fr::one() : fr::zero(),
                                      fr::neg_one(),
                                      fr::zero() });

        fr left_quad = get_variable(left_lo_idx) + get_variable(left_hi_idx) + get_variable(left_hi_idx);
        fr right_quad = get_variable(right_lo_idx) + get_variable(right_hi_idx) + get_variable(right_hi_idx);
        fr out_quad = get_variable(out_lo_idx) + get_variable(out_hi_idx) + get_variable(out_hi_idx);

        uint32_t left_quad_idx = add_variable(left_quad);
        uint32_t right_quad_idx = add_variable(right_quad);
        uint32_t out_quad_idx = add_variable(out_quad);

        fr new_left_accumulator = left_accumulator + left_accumulator;
        new_left_accumulator = new_left_accumulator + new_left_accumulator;
        new_left_accumulator = new_left_accumulator + left_quad;
        uint32_t new_left_accumulator_idx = add_variable(new_left_accumulator);

        create_add_gate(add_triple{ left_accumulator_idx,
                                    left_quad_idx,
                                    new_left_accumulator_idx,
                                    four,
                                    fr::one(),
                                    fr::neg_one(),
                                    fr::zero() });

        fr new_right_accumulator = right_accumulator + right_accumulator;
        new_right_accumulator = new_right_accumulator + new_right_accumulator;
        new_right_accumulator = new_right_accumulator + right_quad;
        uint32_t new_right_accumulator_idx = add_variable(new_right_accumulator);

        create_add_gate(add_triple{ right_accumulator_idx,
                                    right_quad_idx,
                                    new_right_accumulator_idx,
                                    four,
                                    fr::one(),
                                    fr::neg_one(),
                                    fr::zero() });

        fr new_out_accumulator = out_accumulator + out_accumulator;
        new_out_accumulator = new_out_accumulator + new_out_accumulator;
        new_out_accumulator = new_out_accumulator + out_quad;
        uint32_t new_out_accumulator_idx = add_variable(new_out_accumulator);

        create_add_gate(add_triple{
            out_accumulator_idx, out_quad_idx, new_out_accumulator_idx, four, fr::one(), fr::neg_one(), fr::zero() });

        accumulators.left.emplace_back(new_left_accumulator_idx);
        accumulators.right.emplace_back(new_right_accumulator_idx);
        accumulators.out.emplace_back(new_out_accumulator_idx);

        left_accumulator = new_left_accumulator;
        left_accumulator_idx = new_left_accumulator_idx;

        right_accumulator = new_right_accumulator;
        right_accumulator_idx = new_right_accumulator_idx;

        out_accumulator = new_out_accumulator;
        out_accumulator_idx = new_out_accumulator_idx;
    }
    return accumulators;
}

void StandardCircuitConstructor::fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
{
    STANDARD_SELECTOR_REFS
    assert_valid_variables({ witness_index });

    w_l.emplace_back(witness_index);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::one());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(-witness_value);
    ++n;
}

uint32_t StandardCircuitConstructor::put_constant_variable(const barretenberg::fr& variable)
{
    if (constant_variable_indices.contains(variable)) {
        return constant_variable_indices.at(variable);
    } else {

        uint32_t variable_index = add_variable(variable);
        fix_witness(variable_index, variable);
        constant_variable_indices.insert({ variable, variable_index });
        return variable_index;
    }
}

waffle::accumulator_triple StandardCircuitConstructor::create_and_constraint(const uint32_t a,
                                                                             const uint32_t b,
                                                                             const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, false);
}

waffle::accumulator_triple StandardCircuitConstructor::create_xor_constraint(const uint32_t a,
                                                                             const uint32_t b,
                                                                             const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, true);
}

void StandardCircuitConstructor::assert_equal_constant(uint32_t const a_idx, fr const& b, std::string const& msg)
{
    if (variables[a_idx] != b && !failed()) {
        failure(msg);
    }
    auto b_idx = put_constant_variable(b);
    assert_equal(a_idx, b_idx, msg);
}

/**
 * Check if all the circuit gates are correct given the witnesses.
 * Goes through each gates and checks if the identity holds.
 *
 * @return true if the circuit is correct.
 * */
bool StandardCircuitConstructor::check_circuit()
{
    STANDARD_SELECTOR_REFS

    fr gate_sum;
    fr left, right, output;
    for (size_t i = 0; i < n; i++) {
        gate_sum = fr::zero();
        left = get_variable(w_l[i]);
        right = get_variable(w_r[i]);
        output = get_variable(w_o[i]);
        gate_sum = q_m[i] * left * right + q_1[i] * left + q_2[i] * right + q_3[i] * output + q_c[i];
        if (!gate_sum.is_zero())
            return false;
    }
    return true;
}
} // namespace waffle