#include "standard_circuit_builder.hpp"
#include "barretenberg/ecc/curves/bn254/bn254.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"
#include <unordered_map>
#include <unordered_set>

#include "barretenberg/serialize/cbind.hpp"
#include "barretenberg/serialize/msgpack.hpp"

using namespace bb;

namespace bb {

/**
 * Create an addition gate.
 *
 * @param in An add_triple containing the indexes of variables to be placed into the
 * wires w_l, w_r, w_o and addition coefficients to be placed into q_1, q_2, q_3, q_c().
 */
template <typename FF> void StandardCircuitBuilder_<FF>::create_add_gate(const add_triple_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c });

    w_l().emplace_back(in.a);
    w_r().emplace_back(in.b);
    w_o().emplace_back(in.c);
    q_m().emplace_back(FF::zero());
    q_1().emplace_back(in.a_scaling);
    q_2().emplace_back(in.b_scaling);
    q_3().emplace_back(in.c_scaling);
    q_c().emplace_back(in.const_scaling);

    ++this->num_gates;
}

/**
 * Create a big addition gate.
 * (a*a_c + b*b_c + c*c_c + d*d_c + q_c = 0)
 *
 * @param in An add quad containing the indexes of variables a, b, c, d and
 * the scaling factors.
 * */
template <typename FF> void StandardCircuitBuilder_<FF>::create_big_add_gate(const add_quad_<FF>& in)
{
    // (a terms + b terms = temp)
    // (c terms + d  terms + temp = 0 )
    FF t0 = this->get_variable(in.a) * in.a_scaling;
    FF t1 = this->get_variable(in.b) * in.b_scaling;
    FF temp = t0 + t1;
    uint32_t temp_idx = this->add_variable(temp);

    create_add_gate({ in.a, in.b, temp_idx, in.a_scaling, in.b_scaling, FF::neg_one(), FF::zero() });

    create_add_gate({ in.c, in.d, temp_idx, in.c_scaling, in.d_scaling, FF::one(), in.const_scaling });
}

/**
 * Create a balanced addition gate.
 * (a*a_c + b*b_c + c*c_c + d*d_c + q_c = 0, where d is in [0,3])
 *
 * @param in An add quad containing the indexes of variables a, b, c, d and
 * the scaling factors.
 * */
template <typename FF> void StandardCircuitBuilder_<FF>::create_balanced_add_gate(const add_quad_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c, in.d });

    // (a terms + b terms = temp)
    // (c terms + d  terms + temp = 0 )
    FF t0 = this->get_variable(in.a) * in.a_scaling;
    FF t1 = this->get_variable(in.b) * in.b_scaling;
    FF temp = t0 + t1;
    uint32_t temp_idx = this->add_variable(temp);

    w_l().emplace_back(in.a);
    w_r().emplace_back(in.b);
    w_o().emplace_back(temp_idx);
    q_m().emplace_back(FF::zero());
    q_1().emplace_back(in.a_scaling);
    q_2().emplace_back(in.b_scaling);
    q_3().emplace_back(FF::neg_one());
    q_c().emplace_back(FF::zero());

    ++this->num_gates;

    w_l().emplace_back(temp_idx);
    w_r().emplace_back(in.c);
    w_o().emplace_back(in.d);
    q_m().emplace_back(FF::zero());
    q_1().emplace_back(FF::one());
    q_2().emplace_back(in.c_scaling);
    q_3().emplace_back(in.d_scaling);
    q_c().emplace_back(in.const_scaling);

    ++this->num_gates;

    // in.d must be between 0 and 3
    // i.e. in.d * (in.d - 1) * (in.d - 2) = 0
    FF temp_2 = this->get_variable(in.d).sqr() - this->get_variable(in.d);
    uint32_t temp_2_idx = this->add_variable(temp_2);
    w_l().emplace_back(in.d);
    w_r().emplace_back(in.d);
    w_o().emplace_back(temp_2_idx);
    q_m().emplace_back(FF::one());
    q_1().emplace_back(FF::neg_one());
    q_2().emplace_back(FF::zero());
    q_3().emplace_back(FF::neg_one());
    q_c().emplace_back(FF::zero());

    ++this->num_gates;

    constexpr FF neg_two = -FF(2);
    w_l().emplace_back(temp_2_idx);
    w_r().emplace_back(in.d);
    w_o().emplace_back(this->zero_idx);
    q_m().emplace_back(FF::one());
    q_1().emplace_back(neg_two);
    q_2().emplace_back(FF::zero());
    q_3().emplace_back(FF::zero());
    q_c().emplace_back(FF::zero());

    ++this->num_gates;
}

template <typename FF>
void StandardCircuitBuilder_<FF>::create_big_add_gate_with_bit_extraction(const add_quad_<FF>& in)
{
    // blah.
    // delta = (c - 4d)
    // delta^2 = c^2 + 16d^2 - 8dc
    // r = (-2*delta*delta + 9*delta - 7)*delta
    // r =

    FF delta = this->get_variable(in.d);
    delta += delta;
    delta += delta;
    delta = this->get_variable(in.c) - delta;

    uint32_t delta_idx = this->add_variable(delta);
    constexpr FF neg_four = -(FF(4));
    create_add_gate({ in.c, in.d, delta_idx, FF::one(), neg_four, FF::neg_one(), FF::zero() });

    constexpr FF two = FF(2);
    constexpr FF seven = FF(7);
    constexpr FF nine = FF(9);
    const FF r_0 = (delta * nine) - ((delta.sqr() * two) + seven);
    uint32_t r_0_idx = this->add_variable(r_0);
    create_poly_gate({ delta_idx, delta_idx, r_0_idx, -two, nine, FF::zero(), FF::neg_one(), -seven });

    FF r_1 = r_0 * delta;
    uint32_t r_1_idx = this->add_variable(r_1);
    create_mul_gate(mul_triple_<FF>{
        r_0_idx,
        delta_idx,
        r_1_idx,
        FF::one(),
        FF::neg_one(),
        FF::zero(),
    });

    // ain.a1 + bin.b2 + cin.c3 + din.c4 + r_1 = 0

    FF r_2 = (r_1 + (this->get_variable(in.d) * in.d_scaling));
    uint32_t r_2_idx = this->add_variable(r_2);
    create_add_gate({ in.d, r_1_idx, r_2_idx, in.d_scaling, FF::one(), FF::neg_one(), FF::zero() });

    create_big_add_gate(add_quad_<FF>{
        in.a, in.b, in.c, r_2_idx, in.a_scaling, in.b_scaling, in.c_scaling, FF::one(), in.const_scaling });
}

template <typename FF> void StandardCircuitBuilder_<FF>::create_big_mul_gate(const mul_quad_<FF>& in)
{
    FF temp = ((this->get_variable(in.c) * in.c_scaling) + (this->get_variable(in.d) * in.d_scaling));
    uint32_t temp_idx = this->add_variable(temp);
    create_add_gate({ in.c, in.d, temp_idx, in.c_scaling, in.d_scaling, FF::neg_one(), FF::zero() });

    create_poly_gate({ in.a, in.b, temp_idx, in.mul_scaling, in.a_scaling, in.b_scaling, FF::one(), in.const_scaling });
}

/**
 * Create a multiplication gate.
 *
 * @param in A mul_tripple containing the indexes of variables to be placed into the
 * wires w_l, w_r, w_o and scaling coefficients to be placed into q_m, q_3, q_c().
 */
template <typename FF> void StandardCircuitBuilder_<FF>::create_mul_gate(const mul_triple_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c });

    w_l().emplace_back(in.a);
    w_r().emplace_back(in.b);
    w_o().emplace_back(in.c);
    q_m().emplace_back(in.mul_scaling);
    q_1().emplace_back(FF::zero());
    q_2().emplace_back(FF::zero());
    q_3().emplace_back(in.c_scaling);
    q_c().emplace_back(in.const_scaling);

    ++this->num_gates;
}

/**
 * Create a bool gate.
 * This gate constrains a variable to two possible values: 0 or 1.
 *
 * @param variable_index The index of the variable.
 */
template <typename FF> void StandardCircuitBuilder_<FF>::create_bool_gate(const uint32_t variable_index)
{
    this->assert_valid_variables({ variable_index });

    w_l().emplace_back(variable_index);
    w_r().emplace_back(variable_index);
    w_o().emplace_back(variable_index);

    q_m().emplace_back(FF::one());
    q_1().emplace_back(FF::zero());
    q_2().emplace_back(FF::zero());
    q_3().emplace_back(FF::neg_one());
    q_c().emplace_back(FF::zero());

    ++this->num_gates;
}

/**
 * Create a gate where you set all the indexes and coefficients yourself.
 *
 * @param in A poly_triple containing all the information.
 */
template <typename FF> void StandardCircuitBuilder_<FF>::create_poly_gate(const poly_triple_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c });

    w_l().emplace_back(in.a);
    w_r().emplace_back(in.b);
    w_o().emplace_back(in.c);
    q_m().emplace_back(in.q_m);
    q_1().emplace_back(in.q_l);
    q_2().emplace_back(in.q_r);
    q_3().emplace_back(in.q_o);
    q_c().emplace_back(in.q_c);

    ++this->num_gates;
}

template <typename FF>
std::vector<uint32_t> StandardCircuitBuilder_<FF>::decompose_into_base4_accumulators(const uint32_t witness_index,
                                                                                     const size_t num_bits,
                                                                                     std::string const& msg)
{
    ASSERT(num_bits > 0);
    const uint256_t target(this->get_variable(witness_index));

    std::vector<uint32_t> accumulators;

    size_t num_quads = (num_bits >> 1);
    num_quads = (num_quads << 1 == num_bits) ? num_quads : num_quads + 1;
    const auto is_edge_case = [&num_quads, &num_bits](size_t idx) {
        return (idx == num_quads - 1 && ((num_bits & 1ULL) == 1ULL));
    };
    constexpr FF four = FF{ 4, 0, 0, 0 }.to_montgomery_form();
    FF accumulator = FF::zero();
    uint32_t accumulator_idx = 0;
    for (size_t i = num_quads - 1; i < num_quads; --i) {

        bool lo = target.get_bit(2 * i);
        uint32_t lo_idx = this->add_variable(lo ? FF::one() : FF::zero());
        create_bool_gate(lo_idx);

        uint32_t quad_idx;

        if (is_edge_case(i)) {
            quad_idx = lo_idx;
        } else {
            bool hi = target.get_bit(2 * i + 1);
            uint32_t hi_idx = this->add_variable(hi ? FF::one() : FF::zero());
            create_bool_gate(hi_idx);

            uint64_t quad = (lo ? 1U : 0U) + (hi ? 2U : 0U);
            quad_idx = this->add_variable(FF{ quad, 0, 0, 0 }.to_montgomery_form());

            create_add_gate({ lo_idx, hi_idx, quad_idx, FF::one(), FF::one() + FF::one(), FF::neg_one(), FF::zero() });
        }

        if (i == num_quads - 1) {
            accumulators.push_back(quad_idx);
            accumulator = this->get_variable(quad_idx);
            accumulator_idx = quad_idx;
        } else {
            FF new_accumulator = accumulator + accumulator;
            new_accumulator = new_accumulator + new_accumulator;
            new_accumulator = new_accumulator + this->get_variable(quad_idx);
            uint32_t new_accumulator_idx = this->add_variable(new_accumulator);
            create_add_gate(
                { accumulator_idx, quad_idx, new_accumulator_idx, four, FF::one(), FF::neg_one(), FF::zero() });
            accumulators.push_back(new_accumulator_idx);
            accumulator = new_accumulator;
            accumulator_idx = new_accumulator_idx;
        }
    }

    this->assert_equal(witness_index, accumulator_idx, msg);
    return accumulators;
}

template <typename FF>
accumulator_triple_<FF> StandardCircuitBuilder_<FF>::create_logic_constraint(const uint32_t a,
                                                                             const uint32_t b,
                                                                             const size_t num_bits,
                                                                             const bool is_xor_gate)
{
    this->assert_valid_variables({ a, b });

    accumulator_triple_<FF> accumulators;

    const uint256_t left_witness_value(this->get_variable(a));
    const uint256_t right_witness_value(this->get_variable(b));

    FF left_accumulator = FF::zero();
    FF right_accumulator = FF::zero();
    FF out_accumulator = FF::zero();

    uint32_t left_accumulator_idx = this->zero_idx;
    uint32_t right_accumulator_idx = this->zero_idx;
    uint32_t out_accumulator_idx = this->zero_idx;
    constexpr FF four = FF(4);
    constexpr FF neg_two = -FF(2);
    for (size_t i = num_bits - 1; i < num_bits; i -= 2) {
        bool left_hi_val = left_witness_value.get_bit(i);
        bool left_lo_val = left_witness_value.get_bit(i - 1);
        bool right_hi_val = right_witness_value.get_bit((i));
        bool right_lo_val = right_witness_value.get_bit(i - 1);

        uint32_t left_hi_idx = this->add_variable(left_hi_val ? FF::one() : FF::zero());
        uint32_t left_lo_idx = this->add_variable(left_lo_val ? FF::one() : FF::zero());
        uint32_t right_hi_idx = this->add_variable(right_hi_val ? FF::one() : FF::zero());
        uint32_t right_lo_idx = this->add_variable(right_lo_val ? FF::one() : FF::zero());

        bool out_hi_val = is_xor_gate ? left_hi_val ^ right_hi_val : left_hi_val & right_hi_val;
        bool out_lo_val = is_xor_gate ? left_lo_val ^ right_lo_val : left_lo_val & right_lo_val;

        uint32_t out_hi_idx = this->add_variable(out_hi_val ? FF::one() : FF::zero());
        uint32_t out_lo_idx = this->add_variable(out_lo_val ? FF::one() : FF::zero());

        create_bool_gate(left_hi_idx);
        create_bool_gate(right_hi_idx);
        create_bool_gate(out_hi_idx);

        create_bool_gate(left_lo_idx);
        create_bool_gate(right_lo_idx);
        create_bool_gate(out_lo_idx);

        // a & b = ab
        // a ^ b = a + b - 2ab
        create_poly_gate({ left_hi_idx,
                           right_hi_idx,
                           out_hi_idx,
                           is_xor_gate ? neg_two : FF::one(),
                           is_xor_gate ? FF::one() : FF::zero(),
                           is_xor_gate ? FF::one() : FF::zero(),
                           FF::neg_one(),
                           FF::zero() });

        create_poly_gate({ left_lo_idx,
                           right_lo_idx,
                           out_lo_idx,
                           is_xor_gate ? neg_two : FF::one(),
                           is_xor_gate ? FF::one() : FF::zero(),
                           is_xor_gate ? FF::one() : FF::zero(),
                           FF::neg_one(),
                           FF::zero() });

        FF left_quad =
            this->get_variable(left_lo_idx) + this->get_variable(left_hi_idx) + this->get_variable(left_hi_idx);
        FF right_quad =
            this->get_variable(right_lo_idx) + this->get_variable(right_hi_idx) + this->get_variable(right_hi_idx);
        FF out_quad = this->get_variable(out_lo_idx) + this->get_variable(out_hi_idx) + this->get_variable(out_hi_idx);

        uint32_t left_quad_idx = this->add_variable(left_quad);
        uint32_t right_quad_idx = this->add_variable(right_quad);
        uint32_t out_quad_idx = this->add_variable(out_quad);

        FF new_left_accumulator = left_accumulator + left_accumulator;
        new_left_accumulator = new_left_accumulator + new_left_accumulator;
        new_left_accumulator = new_left_accumulator + left_quad;
        uint32_t new_left_accumulator_idx = this->add_variable(new_left_accumulator);

        create_add_gate({ left_accumulator_idx,
                          left_quad_idx,
                          new_left_accumulator_idx,
                          four,
                          FF::one(),
                          FF::neg_one(),
                          FF::zero() });

        FF new_right_accumulator = right_accumulator + right_accumulator;
        new_right_accumulator = new_right_accumulator + new_right_accumulator;
        new_right_accumulator = new_right_accumulator + right_quad;
        uint32_t new_right_accumulator_idx = this->add_variable(new_right_accumulator);

        create_add_gate({ right_accumulator_idx,
                          right_quad_idx,
                          new_right_accumulator_idx,
                          four,
                          FF::one(),
                          FF::neg_one(),
                          FF::zero() });

        FF new_out_accumulator = out_accumulator + out_accumulator;
        new_out_accumulator = new_out_accumulator + new_out_accumulator;
        new_out_accumulator = new_out_accumulator + out_quad;
        uint32_t new_out_accumulator_idx = this->add_variable(new_out_accumulator);

        create_add_gate(
            { out_accumulator_idx, out_quad_idx, new_out_accumulator_idx, four, FF::one(), FF::neg_one(), FF::zero() });

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

template <typename FF>
void StandardCircuitBuilder_<FF>::fix_witness(const uint32_t witness_index, const FF& witness_value)
{
    this->assert_valid_variables({ witness_index });

    w_l().emplace_back(witness_index);
    w_r().emplace_back(this->zero_idx);
    w_o().emplace_back(this->zero_idx);
    q_m().emplace_back(FF::zero());
    q_1().emplace_back(FF::one());
    q_2().emplace_back(FF::zero());
    q_3().emplace_back(FF::zero());
    q_c().emplace_back(-witness_value);
    ++this->num_gates;
}

template <typename FF> uint32_t StandardCircuitBuilder_<FF>::put_constant_variable(const FF& variable)
{
    if (constant_variable_indices.contains(variable)) {
        return constant_variable_indices.at(variable);
    } else {

        uint32_t variable_index = this->add_variable(variable);
        fix_witness(variable_index, variable);
        constant_variable_indices.insert({ variable, variable_index });
        return variable_index;
    }
}

template <typename FF>
accumulator_triple_<FF> StandardCircuitBuilder_<FF>::create_and_constraint(const uint32_t a,
                                                                           const uint32_t b,
                                                                           const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, false);
}

template <typename FF>
accumulator_triple_<FF> StandardCircuitBuilder_<FF>::create_xor_constraint(const uint32_t a,
                                                                           const uint32_t b,
                                                                           const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, true);
}

template <typename FF>
void StandardCircuitBuilder_<FF>::assert_equal_constant(uint32_t const a_idx, FF const& b, std::string const& msg)
{
    if (this->variables[a_idx] != b && !this->failed()) {
        this->failure(msg);
    }
    auto b_idx = put_constant_variable(b);
    this->assert_equal(a_idx, b_idx, msg);
}

/**
 * Check if all the circuit gates are correct given the witnesses.
 * Goes through each gates and checks if the identity holds.
 *
 * @return true if the circuit is correct.
 * */
template <typename FF> bool StandardCircuitBuilder_<FF>::check_circuit()
{

    FF gate_sum;
    FF left, right, output;
    for (size_t i = 0; i < this->num_gates; i++) {

        gate_sum = FF::zero();
        left = this->get_variable(w_l()[i]);
        right = this->get_variable(w_r()[i]);
        output = this->get_variable(w_o()[i]);
        gate_sum = q_m()[i] * left * right + q_1()[i] * left + q_2()[i] * right + q_3()[i] * output + q_c()[i];
        if (!gate_sum.is_zero()) {
            info("gate number", i);
            return false;
        }
    }
    return true;
}

/**
 * Export the existing circuit as msgpack compatible buffer.
 *
 * @return msgpack compatible buffer
 */
template <typename FF> msgpack::sbuffer StandardCircuitBuilder_<FF>::export_circuit()
{
    using base = CircuitBuilderBase<FF>;
    CircuitSchema cir;

    uint64_t modulus[4] = {
        FF::Params::modulus_0, FF::Params::modulus_1, FF::Params::modulus_2, FF::Params::modulus_3
    };
    std::stringstream buf;
    buf << std::hex << std::setfill('0') << std::setw(16) << modulus[3] << std::setw(16) << modulus[2] << std::setw(16)
        << modulus[1] << std::setw(16) << modulus[0];

    cir.modulus = buf.str();

    for (uint32_t i = 0; i < this->get_num_public_inputs(); i++) {
        cir.public_inps.push_back(this->real_variable_index[this->public_inputs[i]]);
    }

    for (auto& tup : base::variable_names) {
        cir.vars_of_interest.insert({ this->real_variable_index[tup.first], tup.second });
    }

    for (auto var : this->variables) {
        cir.variables.push_back(var);
    }

    for (size_t i = 0; i < this->num_gates; i++) {
        std::vector<FF> tmp_sel = { q_m()[i], q_1()[i], q_2()[i], q_3()[i], q_c()[i] };
        std::vector<uint32_t> tmp_w = {
            this->real_variable_index[w_l()[i]],
            this->real_variable_index[w_r()[i]],
            this->real_variable_index[w_o()[i]],
        };
        cir.selectors.push_back(tmp_sel);
        cir.wires.push_back(tmp_w);
    }

    msgpack::sbuffer buffer;
    msgpack::pack(buffer, cir);
    return buffer;
}

template class StandardCircuitBuilder_<bb::fr>;
template class StandardCircuitBuilder_<grumpkin::fr>;

} // namespace bb