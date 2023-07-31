#include "turbo_circuit_builder.hpp"
#include "barretenberg/common/slab_allocator.hpp"
#include "barretenberg/numeric/bitop/get_msb.hpp"

using namespace barretenberg;

namespace proof_system {

/**
 * Turbo circuit initialization, where you can specify the probable number of gates in your circuit.
 *
 * @param size_hint Assumed number of gates. Used to allocate space for various member
 * vectors during initialization.
 * */
template <typename FF>
TurboCircuitBuilder_<FF>::TurboCircuitBuilder_(const size_t size_hint)
    : CircuitBuilderBase<arithmetization::Turbo<FF>>(turbo_selector_names(), size_hint)
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);

    this->zero_idx = put_constant_variable(FF::zero());
}

/**
 * Create an addition gate.
 * The q_m, q_4, q_5, q_fixed_base, q_range, q_logic are zero.
 * q_artith is one. w_4 is set to 0-variable index.
 * Other parameters are received from the argument.
 *
 * @param in Specifies addition gate parameters:
 * w_l, w_r, w_o, q_1, q_2, q_3, q_c.
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::create_add_gate(const add_triple_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(this->zero_idx);
    q_m.emplace_back(FF::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(FF::one());
    q_4.emplace_back(FF::zero());
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * Create an addition gate that adds 4 variables.
 * The q_m, q_5, q_fixed_base, q_range, q_logic are zero.
 * q_arith is one.
 * Other parameters are received from the argument.
 *
 * @param in Specifies addition gate parameters:
 * w_l, w_r, w_o, w_4, q_1, q_2, q_3, q_4, q_c.
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::create_big_add_gate(const add_quad_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(FF::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(FF::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * @brief Create an addition gate that adds 4 variables with bit extraction.
 * The q_m, q_5, q_fixed_base, q_range, q_logic are zero.
 * q_arith is 2, so an additional constraint is imposed in the nonlinear terms.
 * Other parameters are received from the argument.
 *
 * @param in Specifies addition gate parameters:
 * w_l, w_r, w_o, w_4, q_1, q_2, q_3, q_4, q_c.
 *
 * @details Impose the constraint
 * a_scaling . a + b_scaling . b + c_scaling . c + d_scaling . d
 *               + 6 * (high bit of c - 4d)                        == 0.
 * @warning This function assumes that c - 4d lies in the set {0, 1, 2, 3}. The circuit writer should take care to
 * ensure this assumption is backed by a constraint (e.g., c and d could be accumulators produced using the TurboPLONK
 * function `decompose_into_base4_accumulators`).
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::create_big_add_gate_with_bit_extraction(const add_quad_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(FF::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(FF::one() + FF::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

template <typename FF> void TurboCircuitBuilder_<FF>::create_big_mul_gate(const mul_quad_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(FF::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * @brief Create an addition constraint with a range constraint on the fourth witness.
 *
 * @details The constraints imposed by this:
 *     q_1 * w_l + q_2 * w_r + q_3 * w_o  + q_4 * w_4 + q_c == 0
 *  and
 *     w_4 * (w_4 - 1) * (w_4 - 2) == 0   (i.e., w_4 is in {0, 1, 2}).
 *
 * We use this gate to evaluate additions/subtractions of bounded integers. The purpose is to ensure the prover can use
 * the output witness in constraints that require the input to be bounded. For a typical example, look at the function,
 * uint<Composer, Native>::operator+, which calculates addition modulo some number 2**w. This function must
 * calculate a long divison by 2**w and return the remainder. Without the constraint on w_4, the prover could lie about
 * the remainder.
 *
 * @warning Even with the constraint on w_3, it is typically necessary to range constrain the wire value that will be
 * returned.
 */
template <typename FF> void TurboCircuitBuilder_<FF>::create_balanced_add_gate(const add_quad_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(FF::zero());
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(FF::one());
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(FF::one());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * Create multiplication gate.
 * w_4 is set to the index of zero variable.
 * q_1, q_2, q_4, q_4, q_fixed_base, q_range and q_logic are set to zero.
 * q_arith is set to 1.
 *
 * @param in Contains the values for w_l, w_r, w_o,
 * q_m, q_3, q_c.
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::create_mul_gate(const mul_triple_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(this->zero_idx);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(FF::zero());
    q_2.emplace_back(FF::zero());
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(FF::one());
    q_4.emplace_back(FF::zero());
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * Create a gate contraining the variable to 0 or 1.
 * We set selectors in such a way that we get the
 * equation x^2-x=0.
 *
 * @param variable_index The index of the variable.
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::create_bool_gate(const uint32_t variable_index)
{
    this->assert_valid_variables({ variable_index });

    w_l.emplace_back(variable_index);
    w_r.emplace_back(variable_index);
    w_o.emplace_back(variable_index);
    w_4.emplace_back(this->zero_idx);
    q_arith.emplace_back(FF::one());
    q_4.emplace_back(FF::zero());
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());

    q_m.emplace_back(FF::one());
    q_1.emplace_back(FF::zero());
    q_2.emplace_back(FF::zero());
    q_3.emplace_back(FF::neg_one());
    q_c.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * Create poly gate as in standard composer.
 * w_4 is set to zero variable.
 * q_range, q_logic, q_4, q_5, q_fixed_base are set to 0.
 * q_arith is set to 1.
 *
 * @param in Contains the values for
 * w_l, w_r, w_o, q_m, q_1, q_2, q_3, q_c.
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::create_poly_gate(const poly_triple_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(this->zero_idx);
    q_m.emplace_back(in.q_m);
    q_1.emplace_back(in.q_l);
    q_2.emplace_back(in.q_r);
    q_3.emplace_back(in.q_o);
    q_c.emplace_back(in.q_c);
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());

    q_arith.emplace_back(FF::one());
    q_4.emplace_back(FF::zero());
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * Add a grumpkin point, from a 2-bit lookup table, into an accumulator point.
 *
 * @param in Witnesses and values of two points.
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::create_fixed_group_add_gate(const fixed_group_add_quad_<FF>& in)
{
    this->assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    q_arith.emplace_back(FF::zero());
    q_4.emplace_back(FF::zero());
    q_5.emplace_back(FF::zero());
    q_m.emplace_back(FF::zero());
    q_c.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_fixed_base.emplace_back(in.q_y_2);
    ++this->num_gates;
}

/**
 * Add a grumpkin point into an accumulator, while also initializing the accumulator.
 *
 * @param in Addition parameters (points and coefficients).
 * @param init Initialization parameters (points).
 * */
template <typename FF>
void TurboCircuitBuilder_<FF>::create_fixed_group_add_gate_with_init(const fixed_group_add_quad_<FF>& in,
                                                                     const fixed_group_init_quad_<FF>& init)
{
    this->assert_valid_variables({ in.a, in.b, in.c, in.d });

    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    q_arith.emplace_back(FF::zero());
    q_4.emplace_back(init.q_x_1);
    q_5.emplace_back(init.q_x_2);
    q_m.emplace_back(init.q_y_1);
    q_c.emplace_back(init.q_y_2);
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_fixed_base.emplace_back(in.q_y_2);
    ++this->num_gates;
}

template <typename FF> void TurboCircuitBuilder_<FF>::create_fixed_group_add_gate_final(const add_quad_<FF>& in)
{
    create_big_add_gate(in);
}

/**
 * Add a gate that will fix the witness (make it public).
 *
 * @param witness_index Witness variable index.
 * @param witness_value Witness variable value.
 * */
template <typename FF> void TurboCircuitBuilder_<FF>::fix_witness(const uint32_t witness_index, const FF& witness_value)
{
    this->assert_valid_variables({ witness_index });

    w_l.emplace_back(witness_index);
    w_r.emplace_back(this->zero_idx);
    w_o.emplace_back(this->zero_idx);
    w_4.emplace_back(this->zero_idx);
    q_m.emplace_back(FF::zero());
    q_1.emplace_back(FF::one());
    q_2.emplace_back(FF::zero());
    q_3.emplace_back(FF::zero());
    q_c.emplace_back(-witness_value);
    q_arith.emplace_back(FF::one());
    q_4.emplace_back(FF::zero());
    q_5.emplace_back(FF::zero());
    q_fixed_base.emplace_back(FF::zero());
    q_range.emplace_back(FF::zero());
    q_logic.emplace_back(FF::zero());
    ++this->num_gates;
}

/**
 * Create a constraint placing the witness in 2^{num_bits} range.
 *
 * @param witness_index The index of the witness variable to constrain.
 * @param num_bits Constraint size.
 *
 * @return Vector of variable indexes for accumulator variables used in
 * the constraint.
 * */
template <typename FF>
std::vector<uint32_t> TurboCircuitBuilder_<FF>::decompose_into_base4_accumulators(const uint32_t witness_index,
                                                                                  const size_t num_bits,
                                                                                  std::string const& msg)
{
    this->assert_valid_variables({ witness_index });

    ASSERT(num_bits > 0);

    /*
     * The range constraint accumulates base 4 values into a sum.
     * We do this by evaluating a kind of 'raster scan', where we compare adjacent elements
     * and validate that their weighted differences lie in a base for value expansion in powers of 4.
     * Let's say that we want to perform a 32-bit range constraint on a field element x.
     * We can expand x to the desired length via 16 constituent base-4 'quads' {q_0, ..., q_15}:
     *
     *          15
     *          ===
     *          \          i
     *     x =  /    q  . 4
     *          ===   i
     *         i = 0
     *
     * In program memory, we place an accumulating base-4 sum of x {a_0, ..., a_15}, where
     *
     *
     *            i                         |
     *           ===                        | a_0  =                         q_15
     *           \                  i - j   | a_1  =              q_15 . 4 + q_14
     *    a   =  /    q         .  4        | a_2  = q_15 . 4^2 + q_14 . 4 + q_13
     *     i     ===   (15 - j)             |   ...
     *          j = 0                       | a_15 = x
     *
     *
     * From this, we can use our range transition constraint to validate that
     *
     *
     *  a      - 4 . a  ϵ {0, 1, 2, 3}  (for the a_i above, we have
     *   i + 1        i                  a_{i+1} - 4.a_i = q_{14-i}, for i = 0, ..., 15),
     *
     * setting a_{-1} = 0.
     *
     * We place our accumulating sums in program memory in the following sequence:
     *
     * +-----+-----+-----+-----+
     * |  A  |  B  |  C  |  D  |
     * +-----+-----+-----+-----+
     * | a2  | a1  | a0  | 0   |
     * | a6  | a5  | a4  | a3  |
     * | a10 | a9  | a8  | a7  |
     * | a14 | a13 | a12 | a11 |
     * | --- | --- | --- | a15 |
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
     * We will prepend 0 quads to our sequence of accumulator values so that the final accumulator value (equal to the
     * witness value if the range constraint holds) will be in the 4th column of an otherwise unused row. More
     * explicitly, in general (num_bits > 0), a_0 will be placed in column:
     *    - C if num_bits = 7, 8 mod 8
     *    - D if num_bits = 1, 2 mod 8
     *    - A if num_bits = 3, 4 mod 8
     *    - B if num_bits = 5, 6 mod 8
     *
     *  Examples:
     *          7,8-bit                   9,10-bit                11,12-bit                 13,14-bit
     * +-----+-----+-----+-----+ +-----+-----+-----+-----+ +-----+-----+-----+-----+ +-----+-----+-----+-----+
     * |  A  |  B  |  C  |  D  | |  A  |  B  |  C  |  D  | |  A  |  B  |  C  |  D  | |  A  |  B  |  C  |  D  |
     * +-----+-----+-----+-----+ +-----+-----+-----+-----+ +-----+-----+-----+-----+ +-----+-----+-----+-----+
     * | a2  | a1  | a0  | 0   | |  0  |  0  |  0  | 0   | | a0  |  0  |  0  | 0   | | a1  | a0  |  0  | 0   |
     * |  0  |  0  |  0  | a3  | | a3  | a2  | a1  | a0  | | a4  | a3  | a2  | a1  | | a5  | a4  | a3  | a2  |
     * | --- | --- | --- | --- | |  0  |  0  |  0  | a4  | |  0  |  0  |  0  | a5  | |  0  |  0  |  0  | a6  |
     * +-----+-----+-----+-----+ +-----+-----+-----+-----+ +-----+-----+-----+-----+ +-----+-----+-----+-----+
     *
     *
     **/

    const uint256_t witness_value(this->get_variable(witness_index));

    if (witness_value.get_msb() >= num_bits && !this->failed()) {
        this->failure(msg);
    }
    /* num_quad_gates is the minimum number of gates needed to record num_bits-many bits in a table, putting two-bits (a
     * quad) at each position. Since our table has width 4, we can fit 8 bits on a row, hence num_quad_gates is
     * num_bits/8 when num_bits is a multiple of 8, and otherwise it is 1 + (num_bits/8). Because we will always pre-pad
     * with 0 quads to ensure that the final accumulator is in the fourth column, num_quad_gates is also equal to one
     * less than the total number of rows that will be used to record the accumulator values. */
    size_t num_quad_gates = (num_bits >> 3);
    num_quad_gates = (num_quad_gates << 3 == num_bits) ? num_quad_gates : num_quad_gates + 1;

    std::vector<uint32_t, ContainerSlabAllocator<uint32_t>>* wires[4]{ &w_4, &w_o, &w_r, &w_l };

    // num_quads = the number of accumulators used in the table, not including the output row.
    const size_t num_quads = (num_quad_gates << 2);
    // (num_quads << 1) is the number of bits in the non-output row, including 0 quads.
    // ((num_quads << 1) - num_bits) >> 1 is the number of padding 0 quads.
    const size_t forced_zero_threshold = 1 + (((num_quads << 1) - num_bits) >> 1);

    std::vector<uint32_t> accumulators;
    FF accumulator(0);
    uint32_t most_significant_segment = 0;
    // iterate through entries of all but final row
    for (size_t i = 0; i < num_quads + 1; ++i) {
        uint32_t accumulator_index;
        // prepend padding 0 quads
        if (i < forced_zero_threshold) {
            accumulator_index = this->zero_idx;
        } else {
            // accumulate quad
            const size_t bit_index = (num_quads - i) << 1;
            const uint64_t quad = static_cast<uint64_t>(witness_value.get_bit(bit_index)) +
                                  2ULL * static_cast<uint64_t>(witness_value.get_bit(bit_index + 1));
            const FF quad_element = fr{ quad, 0, 0, 0 }.to_montgomery_form();
            accumulator += accumulator;
            accumulator += accumulator;
            accumulator += quad_element;

            accumulator_index = this->add_variable(accumulator);
            accumulators.emplace_back(accumulator_index);

            if (i == forced_zero_threshold) {
                // mark this to constrain top bit to 0 in case num_bits is odd
                most_significant_segment = accumulator_index;
            }
        }

        (*(wires + (i & 3)))->emplace_back(accumulator_index);
    }

    // we use one additional gate to record the final accumulator value.
    size_t used_gates = 1 + num_quad_gates;
    for (size_t i = 0; i < used_gates; ++i) {
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_5.emplace_back(0);
        q_fixed_base.emplace_back(0);
        q_logic.emplace_back(0);
        q_range.emplace_back(1);
    }

    // switch off range widget for final row; fill wire values not in use with zeros
    q_range[q_range.size() - 1] = 0;
    w_l.emplace_back(this->zero_idx);
    w_r.emplace_back(this->zero_idx);
    w_o.emplace_back(this->zero_idx);

    this->assert_equal(witness_index, accumulators[accumulators.size() - 1], msg);

    accumulators[accumulators.size() - 1] = witness_index;

    this->num_gates += used_gates;

    // constrain top bit of top quad to zero in case num_bits is odd
    if ((num_bits & 1ULL) == 1ULL) {
        create_bool_gate(most_significant_segment);
    }
    return accumulators;
}

/**
 * @brief Implements AND and XOR.
 *
 * @returns A triple of vectors of accumulator values.
 *
 * @details If T is the returned triple, then the last element of T.left is guaranteed to be
 * our input a, regardless of a's relation with the rest of T.left. For instance, if num_bits is
 * smaller than the bit length of a, then the constraint that a is reproduced by T.left will fail:
 * for u = T.left[T.left.size()-2], u will be too small to express a in the form a = 4u + quad.
 * The same holds, mutatis mutandis, for T.right.
 */
template <typename FF>
accumulator_triple_<FF> TurboCircuitBuilder_<FF>::create_logic_constraint(const uint32_t a,
                                                                          const uint32_t b,
                                                                          const size_t num_bits,
                                                                          const bool is_xor_gate)
{
    this->assert_valid_variables({ a, b });

    ASSERT(((num_bits >> 1U) << 1U) == num_bits); // Do not allow constraint for an odd number of bits.

    // one gate accmulates 1 quads, or 2 bits.
    // # gates = (bits / 2)
    const size_t num_quads = (num_bits >> 1);

    /*
     * The LOGIC constraint accumulates 3 base-4 values (a, b, c) into a sum, where c = a & b OR c = a ^ b
     *
     * In program memory, we place an accumulating base-4 sum of a, b, c {a_0, ..., a_{(num_bits/2)-1}}, where
     *         i
     *        ===
     *        \                              i - j
     * a   =  /    q                     .  4
     *  i     ===   (num_bits/2 - 1 - j)
     *       j = 0
     *
     * Note that a_0 = q_15, a_1 = 4.q_15 + q_14 = 4.a_0 + q_14, and, in general, we have the
     * accumulator relation
     *
     * a       =  4 a  + q                     (for i > 0).
     *  i + 1        i    num_bits/2 - 1 -i
     *
     * We can use our logic transition constraint to validate that
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
     *                    /                 \          /                 \
     *  c      - 4 . c  = | a      - 4 . a  | (& OR ^) | b      - 4 . b  |
     *   i + 1        i   \  i + 1        i /          \  i + 1        i /
     *
     *
     * We also need the following temporary, w, stored in program memory:
     *
     *      /                 \   /                 \
     * w  = | a      - 4 . a  | * | b      - 4 . b  |
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
     * | 0   | 0   | w0  | 0   |
     * | a0  | b0  | w1  | c0  |
     * | a1  | b1  | w2  | c1  |
     * |  :  |  :  |  :  |  :  |
     * | aN  | bN  |  0  | cN  |     where N = num_bits/2 - 1  (num_bits is assumed even)
     * +-----+-----+-----+-----+
     *
     * Typically we will set num_bits = max(num_bits(a), num_bits(b)), so that c computes the AND or XOR
     * of a and b, depending on the value of is_xor_gate.
     *
     * Our transition constraint extracts quads by taking the difference between two accumulating sums,
     * so we need to start the chain with a row of zeroes
     *
     * One additional benefit of this constraint, is that both our inputs and output are in 'native' uint32 form.
     * This means we *never* have to decompose a uint32 into bits and back in order to chain together
     * addition and logic operations.
     **/

    const uint256_t left_witness_value(this->get_variable(a));
    const uint256_t right_witness_value(this->get_variable(b));

    accumulator_triple_<FF> accumulators;
    FF left_accumulator = FF::zero();
    FF right_accumulator = FF::zero();
    FF out_accumulator = FF::zero();

    // Step 1: populate 1st row accumulators with zero
    w_l.emplace_back(this->zero_idx);
    w_r.emplace_back(this->zero_idx);
    w_4.emplace_back(this->zero_idx);

    // w_l, w_r, w_4 now point to 1 gate ahead of w_o
    for (size_t j = 0; j < num_quads; ++j) {
        uint32_t left_accumulator_index;
        uint32_t right_accumulator_index;
        uint32_t out_accumulator_index;
        uint32_t product_index;

        const size_t bit_index = (num_quads - 1 - j) << 1; // subscript of q as defined above
        // get quad coeffs of 4^{num_quads - 1 - j} in a and b, respectively
        const uint64_t left_quad = static_cast<uint64_t>(left_witness_value.get_bit(bit_index)) +
                                   2ULL * static_cast<uint64_t>(left_witness_value.get_bit(bit_index + 1));
        const uint64_t right_quad = static_cast<uint64_t>(right_witness_value.get_bit(bit_index)) +
                                    2ULL * static_cast<uint64_t>(right_witness_value.get_bit(bit_index + 1));

        const FF left_quad_element = fr{ left_quad, 0, 0, 0 }.to_montgomery_form();
        const FF right_quad_element = fr{ right_quad, 0, 0, 0 }.to_montgomery_form();
        FF out_quad_element;
        if (is_xor_gate) {
            out_quad_element = fr{ left_quad ^ right_quad, 0, 0, 0 }.to_montgomery_form();
        } else {
            out_quad_element = fr{ left_quad & right_quad, 0, 0, 0 }.to_montgomery_form();
        }

        const FF product_quad_element = fr{ left_quad * right_quad, 0, 0, 0 }.to_montgomery_form();

        // replace accumulator by 4.accumulator + quad for a, b and c
        left_accumulator += left_accumulator;
        left_accumulator += left_accumulator;
        left_accumulator += left_quad_element;

        right_accumulator += right_accumulator;
        right_accumulator += right_accumulator;
        right_accumulator += right_quad_element;

        out_accumulator += out_accumulator;
        out_accumulator += out_accumulator;
        out_accumulator += out_quad_element;

        left_accumulator_index = this->add_variable(left_accumulator);
        accumulators.left.emplace_back(left_accumulator_index);

        right_accumulator_index = this->add_variable(right_accumulator);
        accumulators.right.emplace_back(right_accumulator_index);

        out_accumulator_index = this->add_variable(out_accumulator);
        accumulators.out.emplace_back(out_accumulator_index);

        product_index = this->add_variable(product_quad_element);

        w_l.emplace_back(left_accumulator_index);
        w_r.emplace_back(right_accumulator_index);
        w_4.emplace_back(out_accumulator_index);
        w_o.emplace_back(product_index);
    }
    w_o.emplace_back(this->zero_idx);

    for (size_t i = 0; i < num_quads + 1; ++i) {
        q_m.emplace_back(FF::zero());
        q_1.emplace_back(FF::zero());
        q_2.emplace_back(FF::zero());
        q_3.emplace_back(FF::zero());
        q_arith.emplace_back(FF::zero());
        q_4.emplace_back(FF::zero());
        q_5.emplace_back(FF::zero());
        q_fixed_base.emplace_back(FF::zero());
        q_range.emplace_back(FF::zero());
        if (is_xor_gate) {
            q_c.emplace_back(FF::neg_one());
            q_logic.emplace_back(FF::neg_one());
        } else {
            q_c.emplace_back(FF::one());
            q_logic.emplace_back(FF::one());
        }
    }

    q_c[q_c.size() - 1] = FF::zero();         // last gate is a noop
    q_logic[q_logic.size() - 1] = FF::zero(); // last gate is a noop

    this->assert_equal(
        a, accumulators.left[accumulators.left.size() - 1], "cannot reproduce `a` value using accumulator.");

    accumulators.left[accumulators.left.size() - 1] = a;

    this->assert_equal(
        b, accumulators.right[accumulators.right.size() - 1], "cannot reproduce `b` value using accumulator.");

    accumulators.right[accumulators.right.size() - 1] = b;

    this->num_gates += (num_quads + 1);

    return accumulators;
}

template <typename FF>
accumulator_triple_<FF> TurboCircuitBuilder_<FF>::create_and_constraint(const uint32_t a,
                                                                        const uint32_t b,
                                                                        const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, false);
}

template <typename FF>
accumulator_triple_<FF> TurboCircuitBuilder_<FF>::create_xor_constraint(const uint32_t a,
                                                                        const uint32_t b,
                                                                        const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, true);
}

template <typename FF> uint32_t TurboCircuitBuilder_<FF>::put_constant_variable(const FF& variable)
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
/**
 * @brief Just an arithemtic gate zero-equality test, but with base set to 1
 *
 * @param gate_index
 * @return true Evaluation is zero
 * @return false Evaluation is not zero
 */
template <typename FF> inline bool TurboCircuitBuilder_<FF>::lazy_arithmetic_gate_check(const size_t gate_index)
{
    return arithmetic_gate_evaluation(gate_index, FF::one()).is_zero();
}

/**
 * @brief Separately checks individual conditions that make up the fixed base gate
 *
 * @param gate_index Gate index
 * @return bool
 * TODO(luke/kesha): Add some comments explaining in what sense each of these checks are "lazy"
 */
template <typename FF> inline bool TurboCircuitBuilder_<FF>::lazy_fixed_base_gate_check(const size_t gate_index)
{
    ASSERT(gate_index < this->num_gates);

    constexpr FF grumpkin_curve_b(-17);
    constexpr FF nine(9);

    // Get witness values
    FF wire_1_shifted;
    FF wire_2_shifted;
    FF wire_3_shifted;
    FF wire_4_shifted;
    const FF wire_1_value = this->get_variable(w_l[gate_index]);
    const FF wire_2_value = this->get_variable(w_r[gate_index]);
    const FF wire_3_value = this->get_variable(w_o[gate_index]);
    const FF wire_4_value = this->get_variable(w_4[gate_index]);
    if ((gate_index + 1) < this->num_gates) {
        wire_1_shifted = this->get_variable(w_l[gate_index + 1]);
        wire_2_shifted = this->get_variable(w_r[gate_index + 1]);
        wire_3_shifted = this->get_variable(w_o[gate_index + 1]);
        wire_4_shifted = this->get_variable(w_4[gate_index + 1]);
    } else {
        wire_1_shifted = FF::zero();
        wire_2_shifted = FF::zero();
        wire_3_shifted = FF::zero();
        wire_4_shifted = FF::zero();
    }

    // Get selector values
    const FF q_c_value = q_c[gate_index];
    const FF q_fixed_base_value = q_fixed_base[gate_index];
    const FF q_m_value = q_m[gate_index];
    const FF q_1_value = q_1[gate_index];
    const FF q_2_value = q_2[gate_index];
    const FF q_3_value = q_3[gate_index];
    const FF q_4_value = q_4[gate_index];
    const FF q_5_value = q_5[gate_index];

    // Compute, optimizing multiplications (different fromt the way we used in widgets, since the linearization
    // trick is no more)

    FF delta = wire_4_shifted - (wire_4_value + wire_4_value + wire_4_value + wire_4_value);
    FF delta_squared = delta.sqr();

    // accumulator_identity = (δ + 3)(δ + 1)(δ - 1)(δ - 3)
    if (delta_squared != nine && delta_squared != FF::one()) {
        return false;
    }

    // Check x_alpha_identity
    if (!(delta_squared * q_1_value + q_2_value - wire_3_shifted).is_zero()) {
        return false;
    }

    FF T0 = wire_1_shifted + wire_1_value + wire_3_shifted;
    FF T1 = (wire_3_shifted - wire_1_value).sqr();
    T0 = T0 * T1;

    T1 = wire_3_shifted.sqr() * wire_3_shifted;
    FF T2 = wire_2_value.sqr();
    T1 = T1 + T2;
    T1 = -(T1 + grumpkin_curve_b);

    T2 = delta * wire_2_value * q_fixed_base_value;
    T2 = T2 + T2;
    FF T3_part = delta * wire_3_shifted * q_3_value;
    FF T3 = T3_part * wire_2_value;
    T3 = T3 + T3;

    // x_accumulator_identity = α^2 *
    // [(w_1,ω + w_1 + w_3,ω) * (w_3,ω - w_1)^2 - (b + w_3,ω^3 + w_2^2) +  2δ * w_2 * q_fixed_base]
    if (!(T0 + T1 + T2 + T3).is_zero()) {
        return false;
    }
    T0 = (wire_2_shifted + wire_2_value) * (wire_3_shifted - wire_1_value);
    T1 = wire_1_value - wire_1_shifted;
    T2 = wire_2_value - (q_fixed_base_value * delta);
    T1 = T1 * (T2 - T3_part);

    // y_accumulator_identity = α^3 *
    // [(w_2,ω + w_2) * (w_3,ω - w_1) + (w_1 - w_1,ω) * (w_2 - q_fixed_base * δ)]

    if (!(T0 + T1).is_zero()) {
        return false;
    }

    if (!q_c_value.is_zero()) {
        T0 = wire_4_value - FF::one();
        T1 = T0 - wire_3_value;

        if (!T0.is_zero() && !T1.is_zero()) {
            return false;
        }
        T0 = wire_3_value * (q_4_value - wire_1_value) + (FF::one() - wire_4_value) * q_5_value;
        if (!T0.is_zero()) {
            return false;
        }
        T0 = q_c_value * (FF::one() - wire_4_value);
        T1 = wire_2_value * wire_3_value;
        FF y_init_identity = (T0 - T1 + q_m_value * wire_3_value);
        if (!y_init_identity.is_zero()) {
            return false;
        }
    }
    return true;
}

/**
 * @brief Check if the logic gate should pass. (Checks xor or and of values)
 *
 * @param gate_index Gate index
 * @return fr
 */
template <typename FF> inline bool TurboCircuitBuilder_<FF>::lazy_logic_gate_check(const size_t gate_index)
{

    ASSERT(gate_index < this->num_gates);

    FF wire_1_shifted;
    FF wire_2_shifted;
    FF wire_4_shifted;
    const FF wire_1_value = this->get_variable(w_l[gate_index]);
    const FF wire_2_value = this->get_variable(w_r[gate_index]);
    const FF wire_4_value = this->get_variable(w_4[gate_index]);
    if ((gate_index + 1) < this->num_gates) {
        wire_1_shifted = this->get_variable(w_l[gate_index + 1]);
        wire_2_shifted = this->get_variable(w_r[gate_index + 1]);
        wire_4_shifted = this->get_variable(w_4[gate_index + 1]);
    } else {
        wire_1_shifted = FF::zero();
        wire_2_shifted = FF::zero();
        wire_4_shifted = FF::zero();
    }

    // Get selector values
    const FF q_c_value = q_c[gate_index];
    const FF q_logic_value = q_logic[gate_index];
    constexpr FF two(2);
    constexpr FF three(3);
    constexpr FF minus_one = -FF::one();

    FF T0;
    FF T1;
    FF T2;

    // T0 = a
    T0 = wire_1_value + wire_1_value;
    T0 += T0;
    T0 = wire_1_shifted - T0;

    if (!T0.is_zero() && T0 != FF::one() && T0 != two && T0 != three) {
        return false;
    }
    // T1 = b
    T1 = wire_2_value + wire_2_value;
    T1 += T1;
    T1 = wire_2_shifted - T1;

    if (!T1.is_zero() && T1 != FF::one() && T1 != two && T1 != three) {
        return false;
    }

    // T2 = c
    T2 = wire_4_value + wire_4_value;
    T2 += T2;
    T2 = wire_4_shifted - T2;

    if (!T2.is_zero() && T2 != FF::one() && T2 != two && T2 != three) {
        return false;
    }
    uint64_t a = uint256_t(T0).data[0];
    uint64_t b = uint256_t(T1).data[0];
    uint64_t c = uint256_t(T2).data[0];

    if (q_c_value == FF::one() && q_logic_value == FF::one()) {
        return (a & b) == c;
    }

    if (q_c_value == minus_one && q_logic_value == minus_one) {
        return (a ^ b) == c;
    }
    return false;
}
/**
 * @brief Check if the range gate should pass (checks that all the differences are 0,1,2 or 3)
 *
 * @param gate_index Gate index
 * @return bool
 */
template <typename FF> inline bool TurboCircuitBuilder_<FF>::lazy_range_gate_check(const size_t gate_index)
{

    ASSERT(gate_index < this->num_gates);

    FF wire_4_shifted;
    const FF wire_1_value = this->get_variable(w_l[gate_index]);
    const FF wire_2_value = this->get_variable(w_r[gate_index]);
    const FF wire_3_value = this->get_variable(w_o[gate_index]);
    const FF wire_4_value = this->get_variable(w_4[gate_index]);
    if ((gate_index + 1) < this->num_gates) {
        wire_4_shifted = this->get_variable(w_4[gate_index + 1]);
    } else {
        wire_4_shifted = FF::zero();
    }
    constexpr FF two(2);
    constexpr FF three(3);

    FF delta_1 = wire_4_value + wire_4_value;
    delta_1 += delta_1;
    delta_1 = (wire_3_value - delta_1).reduce_once();
    if (!delta_1.is_zero() && delta_1 != FF::one() && delta_1 != two && delta_1 != three) {
        return false;
    }

    FF delta_2 = wire_3_value + wire_3_value;
    delta_2 += delta_2;
    delta_2 = wire_2_value - delta_2;

    if (!delta_2.is_zero() && delta_2 != FF::one() && delta_2 != two && delta_2 != three) {
        return false;
    }
    FF delta_3 = wire_2_value + wire_2_value;
    delta_3 += delta_3;
    delta_3 = wire_1_value - delta_3;

    if (!delta_3.is_zero() && delta_3 != FF::one() && delta_3 != two && delta_3 != three) {
        return false;
    }
    FF delta_4 = wire_1_value + wire_1_value;
    delta_4 += delta_4;
    delta_4 = wire_4_shifted - delta_4;

    if (!delta_4.is_zero() && delta_4 != FF::one() && delta_4 != two && delta_4 != three) {
        return false;
    }

    return true;
}
/**
 * @brief Evaluate the contribution of the arithmetic gate constraint
 *
 * @param gate_index Gate index
 * @param alpha_base The base value that the whole evaluation is multiplied by
 * @return fr
 */
template <typename FF>
inline FF TurboCircuitBuilder_<FF>::arithmetic_gate_evaluation(const size_t gate_index, const FF alpha_base)
{
    ASSERT(gate_index < this->num_gates);

    constexpr FF minus_seven(-7);

    constexpr FF two = FF::one() + FF::one();
    const FF wire_1_value = this->get_variable(w_l[gate_index]);
    const FF wire_2_value = this->get_variable(w_r[gate_index]);
    const FF wire_3_value = this->get_variable(w_o[gate_index]);
    const FF wire_4_value = this->get_variable(w_4[gate_index]);

    // T2  = Δ
    FF T2 = wire_4_value + wire_4_value;
    T2 += T2;
    T2 = wire_3_value - T2;

    // T3 = 2Δ^2
    FF T3 = T2.sqr();
    T3 += T3;

    // T4 = 9.Δ
    FF T4 = T2 + T2;
    T4 += T2;
    // // T5 = 6.Δ
    FF T5 = T4 + T4;
    T4 += T5;

    // T4 = 9.Δ - 2.Δ^2 - 7
    T4 -= T3;
    T4 += minus_seven;

    // T2 = 9.Δ^2 - 2.Δ^3 - 7.Δ
    T2 *= T4;

    return alpha_base * q_arith[gate_index] *
           (wire_1_value * (q_m[gate_index] * wire_2_value + q_1[gate_index]) + q_2[gate_index] * wire_2_value +
            q_3[gate_index] * wire_3_value +
            wire_4_value * (q_4[gate_index] + q_5[gate_index] * (wire_4_value - two) * (wire_4_value - FF::one())) +
            q_c[gate_index] + (q_arith[gate_index] - 1) * T2);
}
/**
 * @brief Evaluate the contribution of the range gate constraint
 *
 * @param gate_index Gate index
 * @param alpha_base The base value that the whole evaluation is multiplied by
 * @param alpha An element used as a separator of individual subrelations
 * @return fr
 */
template <typename FF>
inline FF TurboCircuitBuilder_<FF>::range_gate_evaluation(const size_t gate_index, const FF alpha_base, const FF alpha)
{

    ASSERT(gate_index < this->num_gates);

    FF wire_4_shifted;
    const FF wire_1_value = this->get_variable(w_l[gate_index]);
    const FF wire_2_value = this->get_variable(w_r[gate_index]);
    const FF wire_3_value = this->get_variable(w_o[gate_index]);
    const FF wire_4_value = this->get_variable(w_4[gate_index]);
    if ((gate_index + 1) < this->num_gates) {
        wire_4_shifted = this->get_variable(w_4[gate_index + 1]);
    } else {
        wire_4_shifted = FF::zero();
    }
    constexpr FF minus_two(-2);
    constexpr FF minus_three(-3);
    FF alpha_a = alpha_base;
    FF alpha_b = alpha_a * alpha;
    FF alpha_c = alpha_b * alpha;
    FF alpha_d = alpha_c * alpha;

    FF delta_1 = wire_4_value + wire_4_value;
    delta_1 += delta_1;
    delta_1 = wire_3_value - delta_1;

    FF delta_2 = wire_3_value + wire_3_value;
    delta_2 += delta_2;
    delta_2 = wire_2_value - delta_2;

    FF delta_3 = wire_2_value + wire_2_value;
    delta_3 += delta_3;
    delta_3 = wire_1_value - delta_3;

    FF delta_4 = wire_1_value + wire_1_value;
    delta_4 += delta_4;
    delta_4 = wire_4_shifted - delta_4;

    // D(D - 1)(D - 2)(D - 3).alpha
    FF T0 = delta_1.sqr();
    T0 -= delta_1;
    FF T1 = delta_1 + minus_two;
    T0 *= T1;
    T1 = delta_1 + minus_three;
    T0 *= T1;
    FF range_accumulator = T0 * alpha_a;

    T0 = delta_2.sqr();
    T0 -= delta_2;
    T1 = delta_2 + minus_two;
    T0 *= T1;
    T1 = delta_2 + minus_three;
    T0 *= T1;
    T0 *= alpha_b;
    range_accumulator += T0;

    T0 = delta_3.sqr();
    T0 -= delta_3;
    T1 = delta_3 + minus_two;
    T0 *= T1;
    T1 = delta_3 + minus_three;
    T0 *= T1;
    T0 *= alpha_c;
    range_accumulator += T0;

    T0 = delta_4.sqr();
    T0 -= delta_4;
    T1 = delta_4 + minus_two;
    T0 *= T1;
    T1 = delta_4 + minus_three;
    T0 *= T1;
    T0 *= alpha_d;
    range_accumulator += T0;

    return q_range[gate_index] * range_accumulator;
}
/**
 * @brief Evaluate the contribution of the logic gate constraint
 *
 * @param gate_index Gate index
 * @param alpha_base The base value the whole evaluation is multiplied by
 * @param alpha The element used as separator for individual subrelations
 * @return fr
 */
template <typename FF>
inline FF TurboCircuitBuilder_<FF>::logic_gate_evaluation(const size_t gate_index, const FF alpha_base, const FF alpha)
{

    ASSERT(gate_index < this->num_gates);

    FF wire_1_shifted;
    FF wire_2_shifted;
    FF wire_4_shifted;
    const FF wire_1_value = this->get_variable(w_l[gate_index]);
    const FF wire_2_value = this->get_variable(w_r[gate_index]);
    const FF wire_3_value = this->get_variable(w_o[gate_index]);
    const FF wire_4_value = this->get_variable(w_4[gate_index]);
    if ((gate_index + 1) < this->num_gates) {
        wire_1_shifted = this->get_variable(w_l[gate_index + 1]);
        wire_2_shifted = this->get_variable(w_r[gate_index + 1]);
        wire_4_shifted = this->get_variable(w_4[gate_index + 1]);
    } else {
        wire_1_shifted = FF::zero();
        wire_2_shifted = FF::zero();
        wire_4_shifted = FF::zero();
    }

    // Get selector values
    const FF q_c_value = q_c[gate_index];
    constexpr FF six(6);
    constexpr FF eighty_one(81);
    constexpr FF eighty_three(83);

    FF delta_sum;
    FF delta_squared_sum;
    FF T0;
    FF T1;
    FF T2;
    FF T3;
    FF T4;
    FF identity;

    T0 = wire_1_value + wire_1_value;
    T0 += T0;
    T0 = wire_1_shifted - T0;

    // T1 = b
    T1 = wire_2_value + wire_2_value;
    T1 += T1;
    T1 = wire_2_shifted - T1;

    // delta_sum = a + b
    delta_sum = T0 + T1;

    // T2 = a^2, T3 = b^2
    T2 = T0.sqr();
    T3 = T1.sqr();

    delta_squared_sum = T2 + T3;

    // identity = a^2 + b^2 + 2ab
    identity = delta_sum.sqr();
    // identity = 2ab
    identity -= delta_squared_sum;

    // identity = 2(ab - w)
    T4 = wire_3_value + wire_3_value;
    identity -= T4;
    identity *= alpha;

    // T4 = 4w
    T4 += T4;

    // T2 = a^2 - a
    T2 -= T0;

    // T0 = a^2 - 5a + 6
    T0 += T0;
    T0 += T0;
    T0 = T2 - T0;
    T0 += six;

    // identity = (identity + a(a - 1)(a - 2)(a - 3)) * alpha
    T0 *= T2;
    identity += T0;
    identity *= alpha;

    // T3 = b^2 - b
    T3 -= T1;

    // T1 = b^2 - 5b + 6
    T1 += T1;
    T1 += T1;
    T1 = T3 - T1;
    T1 += six;

    // identity = (identity + b(b - 1)(b - 2)(b - 3)) * alpha
    T1 *= T3;
    identity += T1;
    identity *= alpha;

    // T0 = 3(a + b)
    T0 = delta_sum + delta_sum;
    T0 += delta_sum;

    // T1 = 9(a + b)
    T1 = T0 + T0;
    T1 += T0;

    // delta_sum = 18(a + b)
    delta_sum = T1 + T1;

    // T1 = 81(a + b)
    T2 = delta_sum + delta_sum;
    T2 += T2;
    T1 += T2;

    // delta_squared_sum = 18(a^2 + b^2)
    T2 = delta_squared_sum + delta_squared_sum;
    T2 += delta_squared_sum;
    delta_squared_sum = T2 + T2;
    delta_squared_sum += T2;
    delta_squared_sum += delta_squared_sum;

    // delta_sum = w(4w - 18(a + b) + 81)
    delta_sum = T4 - delta_sum;
    delta_sum += eighty_one;
    delta_sum *= wire_3_value;

    // T1 = 18(a^2 + b^2) - 81(a + b) + 83
    T1 = delta_squared_sum - T1;
    T1 += eighty_three;

    // delta_sum = w ( w ( 4w - 18(a + b) + 81) + 18(a^2 + b^2) - 81(a + b) + 83)
    delta_sum += T1;
    delta_sum *= wire_3_value;

    // T2 = 3c
    T2 = wire_4_value + wire_4_value;
    T2 += T2;
    T2 = wire_4_shifted - T2;
    T3 = T2 + T2;
    T2 += T3;

    // T3 = 9c
    T3 = T2 + T2;
    T3 += T2;

    // T3 = q_c * (9c - 3(a + b))
    T3 -= T0;
    T3 *= q_c_value;

    // T2 = 3c + 3(a + b) - 2 * delta_sum
    T2 += T0;
    delta_sum += delta_sum;
    T2 -= delta_sum;

    // T2 = T2 + T3
    T2 += T3;

    // identity = q_logic * alpha_base * (identity + T2)
    identity += T2;
    identity *= alpha_base;
    return identity * q_logic[gate_index];
}

/**
 * @brief Evaluates the contribution of the fixed_base constraint
 *
 * @param gate_index Gate index
 * @param alpha_powers A vector with alpha_base and alpha_base*α^{i} for enough i
 * @return fr
 */
template <typename FF>
inline FF TurboCircuitBuilder_<FF>::fixed_base_gate_evaluation(const size_t gate_index,
                                                               const std::vector<FF>& alpha_powers)
{
    ASSERT(gate_index < this->num_gates);

    constexpr FF grumpkin_curve_b(-17);
    constexpr FF three(3);

    // Get witness values
    FF wire_1_shifted;
    FF wire_2_shifted;
    FF wire_3_shifted;
    FF wire_4_shifted;
    const FF wire_1_value = this->get_variable(w_l[gate_index]);
    const FF wire_2_value = this->get_variable(w_r[gate_index]);
    const FF wire_3_value = this->get_variable(w_o[gate_index]);
    const FF wire_4_value = this->get_variable(w_4[gate_index]);
    if ((gate_index + 1) < this->num_gates) {
        wire_1_shifted = this->get_variable(w_l[gate_index + 1]);
        wire_2_shifted = this->get_variable(w_r[gate_index + 1]);
        wire_3_shifted = this->get_variable(w_o[gate_index + 1]);
        wire_4_shifted = this->get_variable(w_4[gate_index + 1]);
    } else {
        wire_1_shifted = FF::zero();
        wire_2_shifted = FF::zero();
        wire_3_shifted = FF::zero();
        wire_4_shifted = FF::zero();
    }

    // Get selector values
    const FF q_c_value = q_c[gate_index];
    const FF q_fixed_base_value = q_fixed_base[gate_index];
    const FF q_m_value = q_m[gate_index];
    const FF q_1_value = q_1[gate_index];
    const FF q_2_value = q_2[gate_index];
    const FF q_3_value = q_3[gate_index];
    const FF q_4_value = q_4[gate_index];
    const FF q_5_value = q_5[gate_index];

    // Compute, optimizing multiplications (different from the way we used in widgets, since the linearization
    // trick is no more)

    FF delta = wire_4_shifted - (wire_4_value + wire_4_value + wire_4_value + wire_4_value);
    FF delta_squared = delta.sqr();

    // accumulator_identity = (δ + 3)(δ + 1)(δ - 1)(δ - 3)
    FF result = (delta - three) * (delta - FF::one()) * (delta + FF::one()) * (delta + three) * alpha_powers[0];

    // Originaly q_1 and q_2 multiplicands with x_alpha_identity
    result += (delta_squared * q_1_value + q_2_value - wire_3_shifted) * alpha_powers[1];
    FF T1_part = wire_2_value * alpha_powers[2];
    // Added q_3 multiplicand
    result +=
        ((T1_part + T1_part) + (wire_1_shifted - wire_1_value) * alpha_powers[3]) * delta * wire_3_shifted * q_3_value;

    FF T0 = wire_1_shifted + wire_1_value + wire_3_shifted;
    FF T1 = (wire_3_shifted - wire_1_value).sqr();
    T0 = T0 * T1;

    T1 = wire_3_shifted.sqr() * wire_3_shifted;
    FF T2 = wire_2_value.sqr();
    T1 = T1 + T2;
    T1 = -(T1 + grumpkin_curve_b);

    T2 = delta * wire_2_value * q_fixed_base_value;
    T2 = T2 + T2;

    // x_accumulator_identity = α^2 *
    // [(w_1,ω + w_1 + w_3,ω) * (w_3,ω - w_1)^2 - (b + w_3,ω^3 + w_2^2) +  2δ * w_2 * q_fixed_base]
    result += (T0 + T1 + T2) * alpha_powers[2];

    T0 = (wire_2_shifted + wire_2_value) * (wire_3_shifted - wire_1_value);
    T1 = wire_1_value - wire_1_shifted;
    T2 = wire_2_value - (q_fixed_base_value * delta);
    T1 = T1 * T2;

    // y_accumulator_identity = α^3 *
    // [(w_2,ω + w_2) * (w_3,ω - w_1) + (w_1 - w_1,ω) * (w_2 - q_fixed_base * δ)]

    result += (T0 + T1) * alpha_powers[3];

    T0 = wire_4_value - FF::one();
    T1 = T0 - wire_3_value;
    FF accumulator_init_identity = T0 * T1 * alpha_powers[4];

    // q_4 and q_5
    result += (((wire_3_value * q_4_value + (FF::one() - wire_4_value) * q_5_value) * alpha_powers[5]) +
               (q_m_value * wire_3_value * alpha_powers[6])) *
              q_c_value;

    // x_init_identity = -α^5 * w_1 * w_3
    FF x_init_identity = -(wire_1_value * wire_3_value) * alpha_powers[5];

    // y_init_identity = α^6 * (q_c * (1 - w_4) - w_2 * w_3)
    T0 = FF::one() - wire_4_value;
    T0 = T0 * q_c_value;
    T1 = wire_2_value * wire_3_value;
    FF y_init_identity = (T0 - T1) * alpha_powers[6];

    result += (accumulator_init_identity + x_init_identity + y_init_identity) * q_c_value;
    return result * q_fixed_base_value;
}
/**
 * Check if the circuit constraints hold.
 *
 * @return true if circuit is correct, false if not.
 * */
template <typename FF> bool TurboCircuitBuilder_<FF>::check_circuit()
{
// #define LAZY_CIRCUIT_CHECKS
#ifdef LAZY_CIRCUIT_CHECKS
    for (size_t i = 0; i < this->num_gates; i++) {
        if (!q_arith[i].is_zero() && !lazy_arithmetic_gate_check(i)) {
            return false;
        }
        if (!q_fixed_base[i].is_zero() && !lazy_fixed_base_gate_check(i)) {
            return false;
        }
        if (!q_range[i].is_zero() && !lazy_range_gate_check(i)) {
            return false;
        }
        if (!q_logic[i].is_zero() && !lazy_logic_gate_check(i)) {
            return false;
        }
    }
    return true;
#else
    // Initialize each of the kernels
    const FF alpha_base = FF::random_element();
    const FF alpha = FF::random_element();
    std::vector<FF> alpha_powers;
    alpha_powers.push_back(alpha_base);
    for (size_t i = 1; i < 7; i++) {
        alpha_powers.push_back(alpha_powers[i - 1] * alpha);
    }

    for (size_t i = 0; i < this->get_num_gates(); i++) {
        if (!arithmetic_gate_evaluation(i, alpha_base).is_zero()) {
#ifndef FUZZING
            info("Arithmetic gate ", i, " failed");
#endif
            return false;
        }

        if (!logic_gate_evaluation(i, alpha_base, alpha).is_zero()) {
#ifndef FUZZING
            info("Logic gate ", i, " failed");
#endif
            return false;
        }

        if (!range_gate_evaluation(i, alpha_base, alpha).is_zero()) {
#ifndef FUZZING
            info("Range gate ", i, " failed");
#endif
            return false;
        }

        if (!fixed_base_gate_evaluation(i, alpha_powers).is_zero()) {
#ifndef FUZZING
            info("Arithmetic gate ", i, " failed");
#endif
            return false;
        }
    }
    return true;

#endif
}
template class TurboCircuitBuilder_<barretenberg::fr>;
} // namespace proof_system
