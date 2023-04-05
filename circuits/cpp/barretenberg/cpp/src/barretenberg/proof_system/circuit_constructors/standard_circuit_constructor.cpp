#include "standard_circuit_constructor.hpp"
#include <unordered_set>
#include <unordered_map>

using namespace barretenberg;

namespace proof_system {

/**
 * Create an addition gate.
 *
 * @param in An add_triple containing the indexes of variables to be placed into the
 * wires w_l, w_r, w_o and addition coefficients to be placed into q_1, q_2, q_3, q_c.
 */
void StandardCircuitConstructor::create_add_gate(const add_triple& in)
{
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);

    ++num_gates;
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

    ++num_gates;

    w_l.emplace_back(temp_idx);
    w_r.emplace_back(in.c);
    w_o.emplace_back(in.d);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::one());
    q_2.emplace_back(in.c_scaling);
    q_3.emplace_back(in.d_scaling);
    q_c.emplace_back(in.const_scaling);

    ++num_gates;

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

    ++num_gates;

    constexpr fr neg_two = -fr(2);
    w_l.emplace_back(temp_2_idx);
    w_r.emplace_back(in.d);
    w_o.emplace_back(zero_idx);
    q_m.emplace_back(fr::one());
    q_1.emplace_back(neg_two);
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(fr::zero());

    ++num_gates;
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
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);

    ++num_gates;
}

/**
 * Create a bool gate.
 * This gate constrains a variable to two possible values: 0 or 1.
 *
 * @param variable_index The index of the variable.
 */
void StandardCircuitConstructor::create_bool_gate(const uint32_t variable_index)
{
    assert_valid_variables({ variable_index });

    w_l.emplace_back(variable_index);
    w_r.emplace_back(variable_index);
    w_o.emplace_back(variable_index);

    q_m.emplace_back(fr::one());
    q_1.emplace_back(fr::zero());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::neg_one());
    q_c.emplace_back(fr::zero());

    ++num_gates;
}

/**
 * Create a gate where you set all the indexes and coefficients yourself.
 *
 * @param in A poly_triple containing all the information.
 */
void StandardCircuitConstructor::create_poly_gate(const poly_triple& in)
{
    assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    q_m.emplace_back(in.q_m);
    q_1.emplace_back(in.q_l);
    q_2.emplace_back(in.q_r);
    q_3.emplace_back(in.q_o);
    q_c.emplace_back(in.q_c);

    ++num_gates;
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

accumulator_triple StandardCircuitConstructor::create_logic_constraint(const uint32_t a,
                                                                       const uint32_t b,
                                                                       const size_t num_bits,
                                                                       const bool is_xor_gate)
{
    assert_valid_variables({ a, b });

    accumulator_triple accumulators;

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
    assert_valid_variables({ witness_index });

    w_l.emplace_back(witness_index);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    q_m.emplace_back(fr::zero());
    q_1.emplace_back(fr::one());
    q_2.emplace_back(fr::zero());
    q_3.emplace_back(fr::zero());
    q_c.emplace_back(-witness_value);
    ++num_gates;
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

accumulator_triple StandardCircuitConstructor::create_and_constraint(const uint32_t a,
                                                                     const uint32_t b,
                                                                     const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, false);
}

accumulator_triple StandardCircuitConstructor::create_xor_constraint(const uint32_t a,
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

    fr gate_sum;
    fr left, right, output;
    for (size_t i = 0; i < num_gates; i++) {
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
} // namespace proof_system