#include "turbo_composer.hpp"
#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <numeric/bitop/get_msb.hpp>
#include <plonk/proof_system/widgets/permutation_widget.hpp>
#include <plonk/proof_system/widgets/turbo_arithmetic_widget.hpp>
#include <plonk/proof_system/widgets/turbo_fixed_base_widget.hpp>
#include <plonk/proof_system/widgets/turbo_logic_widget.hpp>
#include <plonk/proof_system/widgets/turbo_range_widget.hpp>
#include <plonk/reference_string/file_reference_string.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>

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

#define TURBO_SEL_NAMES                                                                                                \
    {                                                                                                                  \
        "q_m", "q_c", "q_1", "q_2", "q_3", "q_4", "q_5", "q_arith", "q_ecc_1", "q_range", "q_logic"                    \
    }
TurboComposer::TurboComposer()
    : TurboComposer("../srs_db", 0)
{}

TurboComposer::TurboComposer(std::string const& crs_path, const size_t size_hint)
    : TurboComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)), size_hint){};
TurboComposer::TurboComposer(const size_t size_hint)
    : TurboComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory("../srs_db")), size_hint){};

TurboComposer::TurboComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint)
    : ComposerBase(std::move(crs_factory), 11, size_hint, TURBO_SEL_NAMES)
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);

    zero_idx = put_constant_variable(fr::zero());
}

TurboComposer::TurboComposer(std::shared_ptr<proving_key> const& p_key,
                             std::shared_ptr<verification_key> const& v_key,
                             size_t size_hint)
    : ComposerBase(p_key, v_key, 11, size_hint, TURBO_SEL_NAMES)
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);
    zero_idx = put_constant_variable(fr::zero());
};

void TurboComposer::create_dummy_gate()
{

    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::create_add_gate(const add_triple& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::create_big_add_gate(const add_quad& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::create_big_add_gate_with_bit_extraction(const add_quad& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::create_big_mul_gate(const mul_quad& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

// Creates a width-4 addition gate, where the fourth witness must be a boolean.
// Can be used to normalize a 32-bit addition
void TurboComposer::create_balanced_add_gate(const add_quad& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::create_mul_gate(const mul_triple& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::create_bool_gate(const uint32_t variable_index)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::create_poly_gate(const poly_triple& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

// adds a grumpkin point, from a 2-bit lookup table, into an accumulator point
void TurboComposer::create_fixed_group_add_gate(const fixed_group_add_quad& in)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

// adds a grumpkin point into an accumulator, while also initializing the accumulator
void TurboComposer::create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in,
                                                          const fixed_group_init_quad& init)
{
    TURBO_SELECTOR_REFS
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
    ++n;
}

void TurboComposer::fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
{
    TURBO_SELECTOR_REFS

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
    ++n;
}

std::vector<uint32_t> TurboComposer::create_range_constraint(const uint32_t witness_index, const size_t num_bits)
{
    TURBO_SELECTOR_REFS
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

    const fr witness_value = get_variable(witness_index).from_montgomery_form();

    // one gate accmulates 4 quads, or 8 bits.
    // # gates = (bits / 8)
    size_t num_quad_gates = (num_bits >> 3);

    num_quad_gates = (num_quad_gates << 3 == num_bits) ? num_quad_gates : num_quad_gates + 1;

    // hmm
    std::vector<uint32_t>* wires[4]{ &w_4, &w_o, &w_r, &w_l };

    const size_t num_quads = (num_quad_gates << 2);
    const size_t forced_zero_threshold = 1 + (((num_quads << 1) - num_bits) >> 1);
    std::vector<uint32_t> accumulators;
    fr accumulator = fr::zero();

    for (size_t i = 0; i < num_quads + 1; ++i) {
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

waffle::accumulator_triple TurboComposer::create_logic_constraint(const uint32_t a,
                                                                  const uint32_t b,
                                                                  const size_t num_bits,
                                                                  const bool is_xor_gate)
{
    TURBO_SELECTOR_REFS
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
     * | a1  | a1  | w2  | c1  |
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

    const fr left_witness_value = get_variable(a).from_montgomery_form();
    const fr right_witness_value = get_variable(b).from_montgomery_form();

    // one gate accmulates 1 quads, or 2 bits.
    // # gates = (bits / 2)
    const size_t num_quads = (num_bits >> 1);

    waffle::accumulator_triple accumulators;
    fr left_accumulator = fr::zero();
    fr right_accumulator = fr::zero();
    fr out_accumulator = fr::zero();

    // Step 1: populate 1st row accumulators with zero
    w_l.emplace_back(zero_idx);
    w_r.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);

    // w_l, w_r, w_4 should now point to 1 gate ahead of w_o
    for (size_t i = 0; i < num_quads; ++i) {
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

waffle::accumulator_triple TurboComposer::create_and_constraint(const uint32_t a,
                                                                const uint32_t b,
                                                                const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, false);
}

waffle::accumulator_triple TurboComposer::create_xor_constraint(const uint32_t a,
                                                                const uint32_t b,
                                                                const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, true);
}

uint32_t TurboComposer::put_constant_variable(const barretenberg::fr& variable)
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

std::shared_ptr<proving_key> TurboComposer::compute_proving_key()
{
    if (circuit_proving_key) {
        return circuit_proving_key;
    }
    create_dummy_gate();

    ComposerBase::compute_proving_key();
    compute_sigma_permutations<4, false>(circuit_proving_key.get());
    return circuit_proving_key;
}

std::shared_ptr<verification_key> TurboComposer::compute_verification_key()
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key();
    }

    circuit_verification_key =
        turbo_composer::compute_verification_key(circuit_proving_key, crs_factory_->get_verifier_crs());

    return circuit_verification_key;
}

std::shared_ptr<program_witness> TurboComposer::compute_witness()
{

    return ComposerBase::compute_witness_base<turbo_settings>();
}

TurboProver TurboComposer::create_prover()
{
    compute_proving_key();

    compute_witness();

    TurboProver output_state(circuit_proving_key, witness, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, false>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, false>>(circuit_proving_key.get(), witness.get());
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

UnrolledTurboProver TurboComposer::create_unrolled_prover()
{
    compute_proving_key();
    compute_witness();

    UnrolledTurboProver output_state(circuit_proving_key, witness, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, false>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, false>>(circuit_proving_key.get(), witness.get());
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

TurboVerifier TurboComposer::create_verifier()
{
    compute_verification_key();

    TurboVerifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    return output_state;
}

UnrolledTurboVerifier TurboComposer::create_unrolled_verifier()
{
    compute_verification_key();

    UnrolledTurboVerifier output_state(circuit_verification_key, create_unrolled_manifest(public_inputs.size()));

    return output_state;
}
} // namespace waffle