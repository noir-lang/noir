#include "plookup_composer.hpp"

#include <ecc/curves/bn254/scalar_multiplication/scalar_multiplication.hpp>
#include <numeric/bitop/get_msb.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_arithmetic_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_fixed_base_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/turbo_logic_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/genperm_sort_widget.hpp>
#include <plonk/proof_system/widgets/transition_widgets/elliptic_widget.hpp>
#include <plonk/proof_system/widgets/random_widgets/permutation_widget.hpp>
#include <plonk/proof_system/widgets/random_widgets/plookup_widget.hpp>
#include <plonk/proof_system/types/polynomial_manifest.hpp>
#include <plonk/reference_string/file_reference_string.hpp>
#include <plonk/composer/plookup/compute_verification_key.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>

#include "plookup_tables/plookup_tables.hpp"
#include "plookup_tables/aes128.hpp"
#include "plookup_tables/sha256.hpp"

using namespace barretenberg;

namespace waffle {

#define PLOOKUP_SELECTOR_REFS                                                                                          \
    auto& q_m = selectors[PlookupSelectors::QM];                                                                       \
    auto& q_c = selectors[PlookupSelectors::QC];                                                                       \
    auto& q_1 = selectors[PlookupSelectors::Q1];                                                                       \
    auto& q_2 = selectors[PlookupSelectors::Q2];                                                                       \
    auto& q_3 = selectors[PlookupSelectors::Q3];                                                                       \
    auto& q_4 = selectors[PlookupSelectors::Q4];                                                                       \
    auto& q_5 = selectors[PlookupSelectors::Q5];                                                                       \
    auto& q_arith = selectors[PlookupSelectors::QARITH];                                                               \
    auto& q_ecc_1 = selectors[PlookupSelectors::QECC_1];                                                               \
    auto& q_range = selectors[PlookupSelectors::QRANGE];                                                               \
    auto& q_sort = selectors[PlookupSelectors::QSORT];                                                                 \
    auto& q_logic = selectors[PlookupSelectors::QLOGIC];                                                               \
    auto& q_elliptic = selectors[PlookupSelectors::QELLIPTIC];                                                         \
    auto& q_lookup_index = selectors[PlookupSelectors::QLOOKUPINDEX];                                                  \
    auto& q_lookup_type = selectors[PlookupSelectors::QLOOKUPTYPE];
std::vector<ComposerBase::SelectorProperties> plookup_sel_props()
{
    std::vector<ComposerBase::SelectorProperties> result{
        { "q_m", false, true },         { "q_c", false, true },         { "q_1", false, false },
        { "q_2", false, true },         { "q_3", false, false },        { "q_4", false, false },
        { "q_5", false, false },        { "q_arith", false, false },    { "q_ecc_1", false, false },
        { "q_range", false, false },    { "q_sort", false, false },     { "q_logic", false, false },
        { "q_elliptic", false, false }, { "table_index", false, true }, { "table_type", false, true },
    };
    return result;
}

PlookupComposer::PlookupComposer()
    : PlookupComposer("../srs_db", 0)
{}

PlookupComposer::PlookupComposer(std::string const& crs_path, const size_t size_hint)
    : PlookupComposer(std::unique_ptr<ReferenceStringFactory>(new FileReferenceStringFactory(crs_path)), size_hint){};

PlookupComposer::PlookupComposer(std::unique_ptr<ReferenceStringFactory>&& crs_factory, const size_t size_hint)
    : ComposerBase(std::move(crs_factory), NUM_PLOOKUP_SELECTORS, size_hint, plookup_sel_props())
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);
    zero_idx = put_constant_variable(0);
    tau.insert({ DUMMY_TAG, DUMMY_TAG });
}

PlookupComposer::PlookupComposer(std::shared_ptr<proving_key> const& p_key,
                                 std::shared_ptr<verification_key> const& v_key,
                                 size_t size_hint)
    : ComposerBase(p_key, v_key, NUM_PLOOKUP_SELECTORS, size_hint, plookup_sel_props())
{
    w_l.reserve(size_hint);
    w_r.reserve(size_hint);
    w_o.reserve(size_hint);
    w_4.reserve(size_hint);
    zero_idx = put_constant_variable(0);
    tau.insert({ DUMMY_TAG, DUMMY_TAG });
}

void PlookupComposer::create_dummy_gate()
{

    PLOOKUP_SELECTOR_REFS
    uint32_t idx = add_variable(fr(1));
    w_l.emplace_back(idx);
    w_r.emplace_back(idx);
    w_o.emplace_back(idx);
    w_4.emplace_back(idx);
    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_add_gate(const add_triple& in)
{
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT);
    PLOOKUP_SELECTOR_REFS
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(0);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_big_add_gate(const add_quad& in)
{
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT && in.d != IS_CONSTANT);
    PLOOKUP_SELECTOR_REFS
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(0);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_big_add_gate_with_bit_extraction(const add_quad& in)
{
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT && in.d != IS_CONSTANT);
    PLOOKUP_SELECTOR_REFS
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(0);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1 + 1);
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_big_mul_gate(const mul_quad& in)
{
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT && in.d != IS_CONSTANT);
    PLOOKUP_SELECTOR_REFS
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

// Creates a width-4 addition gate, where the fourth witness must be a boolean.
// Can be used to normalize a 32-bit addition
void PlookupComposer::create_balanced_add_gate(const add_quad& in)
{
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT && in.d != IS_CONSTANT);
    PLOOKUP_SELECTOR_REFS
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);
    q_m.emplace_back(0);
    q_1.emplace_back(in.a_scaling);
    q_2.emplace_back(in.b_scaling);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(in.d_scaling);
    q_5.emplace_back(1);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_mul_gate(const mul_triple& in)
{
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT);
    PLOOKUP_SELECTOR_REFS
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(in.mul_scaling);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(in.c_scaling);
    q_c.emplace_back(in.const_scaling);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_bool_gate(const uint32_t variable_index)
{
    PLOOKUP_SELECTOR_REFS
    ASSERT(variable_index != IS_CONSTANT);
    w_l.emplace_back(variable_index);
    w_r.emplace_back(variable_index);
    w_o.emplace_back(variable_index);
    w_4.emplace_back(zero_idx);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_m.emplace_back(1);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(fr::neg_one());
    q_c.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_poly_gate(const poly_triple& in)
{
    PLOOKUP_SELECTOR_REFS
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(in.q_m);
    q_1.emplace_back(in.q_l);
    q_2.emplace_back(in.q_r);
    q_3.emplace_back(in.q_o);
    q_c.emplace_back(in.q_c);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);

    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

// adds a grumpkin point, from a 2-bit lookup table, into an accumulator point
void PlookupComposer::create_fixed_group_add_gate(const fixed_group_add_quad& in)
{
    PLOOKUP_SELECTOR_REFS
    ASSERT(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT && in.d != IS_CONSTANT);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_m.emplace_back(0);
    q_c.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_ecc_1.emplace_back(in.q_y_2);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

// adds a grumpkin point into an accumulator, while also initializing the accumulator
void PlookupComposer::create_fixed_group_add_gate_with_init(const fixed_group_add_quad& in,
                                                            const fixed_group_init_quad& init)
{
    PLOOKUP_SELECTOR_REFS
    assert(in.a != IS_CONSTANT && in.b != IS_CONSTANT && in.c != IS_CONSTANT && in.d != IS_CONSTANT);
    w_l.emplace_back(in.a);
    w_r.emplace_back(in.b);
    w_o.emplace_back(in.c);
    w_4.emplace_back(in.d);

    q_arith.emplace_back(0);
    q_4.emplace_back(init.q_x_1);
    q_5.emplace_back(init.q_x_2);
    q_m.emplace_back(init.q_y_1);
    q_c.emplace_back(init.q_y_2);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);

    q_1.emplace_back(in.q_x_1);
    q_2.emplace_back(in.q_x_2);
    q_3.emplace_back(in.q_y_1);
    q_ecc_1.emplace_back(in.q_y_2);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::create_ecc_add_gate(const ecc_add_gate& in)
{
    /**
     * | 1  | 2  | 3  | 4  |
     * | a1 | a2 | x1 | y1 |
     * | x2 | y2 | x3 | y3 |
     * | -- | -- | x4 | y4 |
     *
     **/

    ASSERT(in.x1 != IS_CONSTANT && in.y1 != IS_CONSTANT && in.x2 != IS_CONSTANT && in.y2 != IS_CONSTANT &&
           in.x3 != IS_CONSTANT && in.y3 != IS_CONSTANT);
    PLOOKUP_SELECTOR_REFS

    bool can_fuse_into_previous_gate = true;
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (w_r[n - 1] == in.x1);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (w_o[n - 1] == in.y1);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_3[n - 1] == 0);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_4[n - 1] == 0);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_5[n - 1] == 0);
    can_fuse_into_previous_gate = can_fuse_into_previous_gate && (q_arith[n - 1] == 0);

    if (can_fuse_into_previous_gate) {
        ASSERT(w_r[n - 1] == in.x1);
        ASSERT(w_o[n - 1] == in.y1);

        q_3[n - 1] = in.endomorphism_coefficient;
        q_4[n - 1] = in.endomorphism_coefficient.sqr();
        q_5[n - 1] = in.sign_coefficient;
        q_elliptic[n - 1] = 1;
    } else {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(in.x1);
        w_o.emplace_back(in.y1);
        w_4.emplace_back(zero_idx);
        q_3.emplace_back(in.endomorphism_coefficient);
        q_4.emplace_back(in.endomorphism_coefficient.sqr());
        q_5.emplace_back(in.sign_coefficient);

        q_arith.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_m.emplace_back(0);
        q_c.emplace_back(0);
        q_ecc_1.emplace_back(0);
        q_range.emplace_back(0);
        q_sort.emplace_back(0);
        q_logic.emplace_back(0);
        q_lookup_index.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_elliptic.emplace_back(1);
        ++n;
    }

    w_l.emplace_back(in.x2);
    w_4.emplace_back(in.y2);
    w_r.emplace_back(in.x3);
    w_o.emplace_back(in.y3);
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

void PlookupComposer::fix_witness(const uint32_t witness_index, const barretenberg::fr& witness_value)
{
    PLOOKUP_SELECTOR_REFS
    ASSERT(witness_index != IS_CONSTANT);
    w_l.emplace_back(witness_index);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    q_m.emplace_back(0);
    q_1.emplace_back(1);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(-witness_value);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_logic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    q_elliptic.emplace_back(0);
    ++n;
}

std::vector<uint32_t> PlookupComposer::create_range_constraint(const uint32_t witness_index,
                                                               const size_t num_bits,
                                                               std::string const& msg)
{
    PLOOKUP_SELECTOR_REFS
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
    fr accumulator = 0;

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
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_5.emplace_back(0);
        q_ecc_1.emplace_back(0);
        q_logic.emplace_back(0);
        q_range.emplace_back(1);
        q_sort.emplace_back(0);
        q_lookup_index.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_elliptic.emplace_back(0);
    }

    q_range[q_range.size() - 1] = 0;

    w_l.emplace_back(zero_idx);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);

    assert_equal(accumulators[accumulators.size() - 1], witness_index, msg);
    accumulators[accumulators.size() - 1] = witness_index;

    n += used_gates;
    return accumulators;
}

waffle::accumulator_triple PlookupComposer::create_logic_constraint(const uint32_t a,
                                                                    const uint32_t b,
                                                                    const size_t num_bits,
                                                                    const bool is_xor_gate)
{
    PLOOKUP_SELECTOR_REFS
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

    const fr left_witness_value = get_variable(a).from_montgomery_form();
    const fr right_witness_value = get_variable(b).from_montgomery_form();

    // one gate accmulates 1 quads, or 2 bits.
    // # gates = (bits / 2)
    const size_t num_quads = (num_bits >> 1);

    waffle::accumulator_triple accumulators;
    fr left_accumulator = 0;
    fr right_accumulator = 0;
    fr out_accumulator = 0;

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
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_5.emplace_back(0);
        q_ecc_1.emplace_back(0);
        q_range.emplace_back(0);
        q_sort.emplace_back(0);
        if (is_xor_gate) {
            q_c.emplace_back(fr::neg_one());
            q_logic.emplace_back(fr::neg_one());
        } else {
            q_c.emplace_back(1);
            q_logic.emplace_back(1);
        }
        q_lookup_index.emplace_back(0);
        q_lookup_type.emplace_back(0);
        q_elliptic.emplace_back(0);
    }
    q_c[q_c.size() - 1] = 0;         // last gate is a noop
    q_logic[q_logic.size() - 1] = 0; // last gate is a noop

    assert_equal(accumulators.left[accumulators.left.size() - 1], a);
    accumulators.left[accumulators.left.size() - 1] = a;

    assert_equal(accumulators.right[accumulators.right.size() - 1], b);
    accumulators.right[accumulators.right.size() - 1] = b;

    n += (num_quads + 1);
    return accumulators;
}

waffle::accumulator_triple PlookupComposer::create_and_constraint(const uint32_t a,
                                                                  const uint32_t b,
                                                                  const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, false);
}

waffle::accumulator_triple PlookupComposer::create_xor_constraint(const uint32_t a,
                                                                  const uint32_t b,
                                                                  const size_t num_bits)
{
    return create_logic_constraint(a, b, num_bits, true);
}

uint32_t PlookupComposer::put_constant_variable(const barretenberg::fr& variable)
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

void PlookupComposer::add_lookup_selector(polynomial& small, const std::string& tag)
{
    polynomial lagrange_base(small, circuit_proving_key->small_domain.size);
    small.ifft(circuit_proving_key->small_domain);
    polynomial large(small, circuit_proving_key->n * 4);
    large.coset_fft(circuit_proving_key->large_domain);

    circuit_proving_key->constraint_selectors.insert({ tag, std::move(small) });
    circuit_proving_key->constraint_selectors_lagrange_base.insert({ tag, std::move(lagrange_base) });
    circuit_proving_key->constraint_selector_ffts.insert({ tag + "_fft", std::move(large) });
}

std::shared_ptr<proving_key> PlookupComposer::compute_proving_key()
{
    PLOOKUP_SELECTOR_REFS;
    if (circuit_proving_key) {
        return circuit_proving_key;
    }

    ASSERT(n == q_m.size());
    ASSERT(n == q_c.size());
    ASSERT(n == q_1.size());
    ASSERT(n == q_2.size());
    ASSERT(n == q_3.size());
    ASSERT(n == q_3.size());
    ASSERT(n == q_4.size());
    ASSERT(n == q_5.size());
    ASSERT(n == q_arith.size());
    ASSERT(n == q_ecc_1.size());
    ASSERT(n == q_elliptic.size());
    ASSERT(n == q_range.size());
    ASSERT(n == q_sort.size());
    ASSERT(n == q_logic.size());
    ASSERT(n == q_lookup_index.size());
    ASSERT(n == q_lookup_type.size());

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    ComposerBase::compute_proving_key_base(tables_size + lookups_size, NUM_RESERVED_GATES);

    const size_t subgroup_size = circuit_proving_key->n;

    polynomial poly_q_table_1(subgroup_size);
    polynomial poly_q_table_2(subgroup_size);
    polynomial poly_q_table_3(subgroup_size);
    polynomial poly_q_table_4(subgroup_size);
    size_t offset = subgroup_size - tables_size - s_randomness;

    for (size_t i = 0; i < offset; ++i) {
        poly_q_table_1[i] = 0;
        poly_q_table_2[i] = 0;
        poly_q_table_3[i] = 0;
        poly_q_table_4[i] = 0;
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

    // Initialise the last `s_randomness` positions in table polynomials with 0.
    // These will be the positions where we will be adding random scalars to add zero knowledge
    // to plookup.
    for (size_t i = 0; i < s_randomness; ++i) {
        poly_q_table_1[offset] = 0;
        poly_q_table_2[offset] = 0;
        poly_q_table_3[offset] = 0;
        poly_q_table_4[offset] = 0;
        ++offset;
    }

    add_lookup_selector(poly_q_table_1, "table_value_1");
    add_lookup_selector(poly_q_table_2, "table_value_2");
    add_lookup_selector(poly_q_table_3, "table_value_3");
    add_lookup_selector(poly_q_table_4, "table_value_4");

    polynomial z_lookup_fft(subgroup_size * 4, subgroup_size * 4);
    polynomial s_fft(subgroup_size * 4, subgroup_size * 4);
    circuit_proving_key->wire_ffts.insert({ "z_lookup_fft", std::move(z_lookup_fft) });
    circuit_proving_key->wire_ffts.insert({ "s_fft", std::move(s_fft) });

    compute_sigma_permutations<4, true>(circuit_proving_key.get());

    std::copy(plookup_polynomial_manifest,
              plookup_polynomial_manifest + 34,
              std::back_inserter(circuit_proving_key->polynomial_manifest));

    return circuit_proving_key;
}

std::shared_ptr<verification_key> PlookupComposer::compute_verification_key()
{
    if (circuit_verification_key) {
        return circuit_verification_key;
    }
    if (!circuit_proving_key) {
        compute_proving_key();
    }
    circuit_verification_key =
        plookup_composer::compute_verification_key(circuit_proving_key, crs_factory_->get_verifier_crs());

    return circuit_verification_key;
}

std::shared_ptr<program_witness> PlookupComposer::compute_witness()
{
    if (witness) {
        return witness;
    }

    size_t tables_size = 0;
    size_t lookups_size = 0;
    for (const auto& table : lookup_tables) {
        tables_size += table.size;
        lookups_size += table.lookup_gates.size();
    }

    const size_t filled_gates = n + public_inputs.size();
    const size_t total_num_gates = std::max(filled_gates, tables_size + lookups_size);

    const size_t subgroup_size = get_circuit_subgroup_size(total_num_gates + NUM_RESERVED_GATES);

    for (size_t i = filled_gates; i < subgroup_size; ++i) {
        w_l.emplace_back(zero_idx);
        w_r.emplace_back(zero_idx);
        w_o.emplace_back(zero_idx);
        w_4.emplace_back(zero_idx);
    }

    polynomial poly_w_1(subgroup_size);
    polynomial poly_w_2(subgroup_size);
    polynomial poly_w_3(subgroup_size);
    polynomial poly_w_4(subgroup_size);
    polynomial s_1(subgroup_size);
    polynomial s_2(subgroup_size);
    polynomial s_3(subgroup_size);
    polynomial s_4(subgroup_size);
    polynomial z_lookup(subgroup_size + 1);
    for (size_t i = 0; i < public_inputs.size(); ++i) {
        poly_w_1[i] = 0;
        poly_w_2[i] = variables[public_inputs[i]];
        poly_w_3[i] = 0;
        poly_w_4[i] = 0;
    }
    for (size_t i = public_inputs.size(); i < subgroup_size; ++i) {
        poly_w_1[i] = variables[w_l[i - public_inputs.size()]];
        poly_w_2[i] = variables[w_r[i - public_inputs.size()]];
        poly_w_3[i] = variables[w_o[i - public_inputs.size()]];
        poly_w_4[i] = variables[w_4[i - public_inputs.size()]];
    }

    // Save space for adding random scalars in s polynomial later
    // We need to make space for adding randomness into witness polynomials and
    // lookup polynomials.
    size_t count = subgroup_size - tables_size - lookups_size - s_randomness;
    for (size_t i = 0; i < count; ++i) {
        s_1[i] = 0;
        s_2[i] = 0;
        s_3[i] = 0;
        s_4[i] = 0;
    }

    for (auto& table : lookup_tables) {
        const fr table_index(table.table_index);
        auto& lookup_gates = table.lookup_gates;
        for (size_t i = 0; i < table.size; ++i) {
            if (table.use_twin_keys) {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        table.column_2[i].from_montgomery_form().data[0],
                    },
                    {
                        table.column_3[i],
                        0,
                    },
                });
            } else {
                lookup_gates.push_back({
                    {
                        table.column_1[i].from_montgomery_form().data[0],
                        0,
                    },
                    {
                        table.column_2[i],
                        table.column_3[i],
                    },
                });
            }
        }

        std::sort(lookup_gates.begin(), lookup_gates.end());

        for (const auto& entry : lookup_gates) {
            const auto components = entry.to_sorted_list_components(table.use_twin_keys);
            s_1[count] = components[0];
            s_2[count] = components[1];
            s_3[count] = components[2];
            s_4[count] = table_index;
            ++count;
        }
    }

    // Initialise the last `s_randomness` positions in s polynomials with 0.
    // These will be the positions where we will be adding random scalars to add zero knowledge
    // to plookup.
    for (size_t i = 0; i < s_randomness; ++i) {
        s_1[count] = 0;
        s_2[count] = 0;
        s_3[count] = 0;
        s_4[count] = 0;
        ++count;
    }

    witness = std::make_shared<program_witness>();
    witness->wires.insert({ "w_1", std::move(poly_w_1) });
    witness->wires.insert({ "w_2", std::move(poly_w_2) });
    witness->wires.insert({ "w_3", std::move(poly_w_3) });
    witness->wires.insert({ "w_4", std::move(poly_w_4) });
    witness->wires.insert({ "s", std::move(s_1) });
    witness->wires.insert({ "s_2", std::move(s_2) });
    witness->wires.insert({ "s_3", std::move(s_3) });
    witness->wires.insert({ "s_4", std::move(s_4) });
    witness->wires.insert({ "z_lookup", std::move(z_lookup) });

    return witness;
}

PlookupProver PlookupComposer::create_prover()
{
    compute_proving_key();
    compute_witness();
    PlookupProver output_state(circuit_proving_key, witness, create_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverPlookupWidget<>> plookup_widget =
        std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get(), witness.get());

    std::unique_ptr<ProverTurboArithmeticWidget<plookup_settings>> arithmetic_widget =
        std::make_unique<ProverTurboArithmeticWidget<plookup_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboFixedBaseWidget<plookup_settings>> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget<plookup_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverGenPermSortWidget<plookup_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<plookup_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget<plookup_settings>> logic_widget =
        std::make_unique<ProverTurboLogicWidget<plookup_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverEllipticWidget<plookup_settings>> elliptic_widget =
        std::make_unique<ProverEllipticWidget<plookup_settings>>(circuit_proving_key.get(), witness.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(logic_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));

    std::unique_ptr<KateCommitmentScheme<plookup_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<plookup_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

UnrolledPlookupProver PlookupComposer::create_unrolled_prover()
{
    compute_proving_key();
    compute_witness();

    UnrolledPlookupProver output_state(circuit_proving_key, witness, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<ProverPermutationWidget<4, true>> permutation_widget =
        std::make_unique<ProverPermutationWidget<4, true>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverPlookupWidget<>> plookup_widget =
        std::make_unique<ProverPlookupWidget<>>(circuit_proving_key.get(), witness.get());

    std::unique_ptr<ProverTurboArithmeticWidget<unrolled_turbo_settings>> arithmetic_widget =
        std::make_unique<ProverTurboArithmeticWidget<unrolled_turbo_settings>>(circuit_proving_key.get(),
                                                                               witness.get());
    std::unique_ptr<ProverTurboFixedBaseWidget<unrolled_turbo_settings>> fixed_base_widget =
        std::make_unique<ProverTurboFixedBaseWidget<unrolled_turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverGenPermSortWidget<unrolled_turbo_settings>> sort_widget =
        std::make_unique<ProverGenPermSortWidget<unrolled_turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverTurboLogicWidget<unrolled_turbo_settings>> logic_widget =
        std::make_unique<ProverTurboLogicWidget<unrolled_turbo_settings>>(circuit_proving_key.get(), witness.get());
    std::unique_ptr<ProverEllipticWidget<unrolled_turbo_settings>> elliptic_widget =
        std::make_unique<ProverEllipticWidget<unrolled_turbo_settings>>(circuit_proving_key.get(), witness.get());

    output_state.random_widgets.emplace_back(std::move(permutation_widget));
    output_state.random_widgets.emplace_back(std::move(plookup_widget));

    output_state.transition_widgets.emplace_back(std::move(arithmetic_widget));
    output_state.transition_widgets.emplace_back(std::move(fixed_base_widget));
    output_state.transition_widgets.emplace_back(std::move(sort_widget));
    output_state.transition_widgets.emplace_back(std::move(logic_widget));
    output_state.transition_widgets.emplace_back(std::move(elliptic_widget));

    std::unique_ptr<KateCommitmentScheme<unrolled_turbo_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<unrolled_turbo_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

PlookupVerifier PlookupComposer::create_verifier()
{
    compute_verification_key();

    PlookupVerifier output_state(circuit_verification_key, create_manifest(public_inputs.size()));

    std::unique_ptr<KateCommitmentScheme<turbo_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<turbo_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

UnrolledPlookupVerifier PlookupComposer::create_unrolled_verifier()
{
    compute_verification_key();

    UnrolledPlookupVerifier output_state(circuit_verification_key, create_unrolled_manifest(public_inputs.size()));

    std::unique_ptr<KateCommitmentScheme<unrolled_turbo_settings>> kate_commitment_scheme =
        std::make_unique<KateCommitmentScheme<unrolled_turbo_settings>>();

    output_state.commitment_scheme = std::move(kate_commitment_scheme);

    return output_state;
}

void PlookupComposer::initialize_precomputed_table(
    const PlookupBasicTableId id,
    bool (*generator)(std::vector<fr>&, std ::vector<fr>&, std::vector<fr>&),
    std::array<fr, 2> (*get_values_from_key)(const std::array<uint64_t, 2>))
{
    for (auto table : lookup_tables) {
        ASSERT(table.id != id);
    }
    PlookupBasicTable new_table;
    new_table.id = id;
    new_table.table_index = lookup_tables.size() + 1;
    new_table.use_twin_keys = generator(new_table.column_1, new_table.column_2, new_table.column_3);
    new_table.size = new_table.column_1.size();
    new_table.get_values_from_key = get_values_from_key;
    lookup_tables.emplace_back(new_table);
}

PlookupBasicTable& PlookupComposer::get_table(const PlookupBasicTableId id)
{
    for (PlookupBasicTable& table : lookup_tables) {
        if (table.id == id) {
            return table;
        }
    }
    // Hmm. table doesn't exist! try to create it
    lookup_tables.emplace_back(plookup::create_basic_table(id, lookup_tables.size()));
    return lookup_tables[lookup_tables.size() - 1];
}

std::array<std::vector<uint32_t>, 3> PlookupComposer::read_sequence_from_multi_table(const PlookupMultiTableId& id,
                                                                                     const PlookupReadData& read_values,
                                                                                     const uint32_t key_a_index,
                                                                                     const uint32_t key_b_index)

{
    PLOOKUP_SELECTOR_REFS;
    const auto& multi_table = plookup::create_table(id);
    const size_t num_lookups = read_values.column_1_accumulator_values.size();
    std::array<std::vector<uint32_t>, 3> column_indices;
    for (size_t i = 0; i < num_lookups; ++i) {
        auto& table = get_table(multi_table.lookup_ids[i]);

        table.lookup_gates.emplace_back(read_values.key_entries[i]);

        const auto first_idx = (i == 0) ? key_a_index : add_variable(read_values.column_1_accumulator_values[i]);
        const auto second_idx = (i == 0 && key_b_index != IS_CONSTANT)
                                    ? key_b_index
                                    : add_variable(read_values.column_2_accumulator_values[i]);
        const auto third_idx = add_variable(read_values.column_3_accumulator_values[i]);

        column_indices[0].push_back(first_idx);
        column_indices[1].push_back(second_idx);
        column_indices[2].push_back(third_idx);
        ASSERT(first_idx != IS_CONSTANT && second_idx != IS_CONSTANT && third_idx != IS_CONSTANT);
        q_lookup_type.emplace_back(fr(1));
        q_lookup_index.emplace_back(fr(table.table_index));
        w_l.emplace_back(first_idx);
        w_r.emplace_back(second_idx);
        w_o.emplace_back(third_idx);
        w_4.emplace_back(zero_idx);
        q_1.emplace_back(0);
        q_2.emplace_back((i == (num_lookups - 1) ? 0 : -multi_table.column_1_step_sizes[i + 1]));
        q_3.emplace_back(0);
        q_m.emplace_back((i == (num_lookups - 1) ? 0 : -multi_table.column_2_step_sizes[i + 1]));
        q_c.emplace_back((i == (num_lookups - 1) ? 0 : -multi_table.column_3_step_sizes[i + 1]));
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_5.emplace_back(0);
        q_ecc_1.emplace_back(0);
        q_range.emplace_back(0);
        q_sort.emplace_back(0);
        q_logic.emplace_back(0);
        q_elliptic.emplace_back(0);

        ++n;
    }
    return column_indices;
}

/**
 * Generalized Permutation Methods
 **/

PlookupComposer::RangeList PlookupComposer::create_range_list(const uint64_t target_range)
{
    RangeList result;
    const auto range_tag = get_new_tag(); // current_tag + 1;
    const auto tau_tag = get_new_tag();   // current_tag + 2;
    create_tag(range_tag, tau_tag);
    create_tag(tau_tag, range_tag);
    result.target_range = target_range;
    result.range_tag = range_tag;
    result.tau_tag = tau_tag;

    uint64_t num_multiples_of_three = (target_range / 3);

    result.variable_indices.reserve((uint32_t)num_multiples_of_three);
    for (uint64_t i = 0; i <= num_multiples_of_three; ++i) {
        const uint32_t index = add_variable(i * 3);
        result.variable_indices.emplace_back(index);
        assign_tag(index, result.range_tag);
    }
    {
        const uint32_t index = add_variable(target_range);
        result.variable_indices.emplace_back(index);
        assign_tag(index, result.range_tag);
    }
    // Need this because these variables will not appear in the witness otherwise
    create_dummy_constraints(result.variable_indices);

    return result;
}
// range constraint a value by decomposing it into limbs whose size should be the default range constraint size
std::vector<uint32_t> PlookupComposer::decompose_into_default_range(const uint32_t variable_index,
                                                                    const size_t num_bits)
{
    ASSERT(variable_index != IS_CONSTANT);
    std::vector<uint32_t> sums;
    const size_t limb_num = (size_t)num_bits / DEFAULT_PLOOKUP_RANGE_BITNUM;
    const size_t last_limb_size = num_bits - (limb_num * DEFAULT_PLOOKUP_RANGE_BITNUM);
    if (limb_num < 2) {
        std::cout << "number of bits in range must be at least twice default range size" << std::endl;
        return sums;
    }

    const uint256_t val = (uint256_t)(get_variable(variable_index));
    // check witness value is indeed in range (commented out cause interferes with negative tests)
    // ASSERT(val < ((uint256_t)1 << num_bits) - 1); // Q:ask Zac what happens with wrapping when converting fr to
    std::vector<uint32_t> val_limbs;
    std::vector<fr> val_slices;
    for (size_t i = 0; i < limb_num; i++) {
        val_slices.emplace_back(
            barretenberg::fr(val.slice(DEFAULT_PLOOKUP_RANGE_BITNUM * i, DEFAULT_PLOOKUP_RANGE_BITNUM * (i + 1))));
        val_limbs.emplace_back(add_variable(val_slices[i]));
        create_new_range_constraint(val_limbs[i], DEFAULT_PLOOKUP_RANGE_SIZE);
    }

    uint64_t last_limb_range = ((uint64_t)1 << last_limb_size) - 1;
    size_t total_limb_num = limb_num;
    if (last_limb_size > 0) {
        val_slices.emplace_back(fr(val.slice(num_bits - last_limb_size, num_bits)));

        val_limbs.emplace_back(add_variable(val_slices[val_slices.size() - 1]));
        create_new_range_constraint(val_limbs[val_limbs.size() - 1], last_limb_range);
        total_limb_num++;
    }
    // pad slices and limbs in case there are odd num of them
    if (total_limb_num % 2 == 1) {
        val_limbs.emplace_back(zero_idx); // TODO: check this is zero
        val_slices.emplace_back(0);
        total_limb_num++;
    }
    fr shift = fr(1 << DEFAULT_PLOOKUP_RANGE_BITNUM);
    fr second_shift = shift * shift;
    sums.emplace_back(add_variable(val_slices[0] + shift * val_slices[1]));
    create_add_gate({ val_limbs[0], val_limbs[1], sums[0], 1, shift, -1, 0 });
    fr cur_shift = (second_shift);
    fr cur_second_shift = cur_shift * shift;
    for (size_t i = 2; i < total_limb_num; i = i + 2) {
        sums.emplace_back(add_variable(get_variable(sums[sums.size() - 1]) + cur_shift * val_slices[i] +
                                       cur_second_shift * val_slices[i + 1]));
        create_big_add_gate({ sums[sums.size() - 2],
                              val_limbs[i],
                              val_limbs[i + 1],
                              sums[sums.size() - 1],
                              1,
                              cur_shift,
                              cur_second_shift,
                              -1,
                              0 });
        cur_shift *= second_shift;
        cur_second_shift *= second_shift;
    }
    assert_equal(sums[sums.size() - 1], variable_index);
    return sums;
}
void PlookupComposer::create_new_range_constraint(const uint32_t variable_index, const uint64_t target_range)
{
    if (range_lists.count(target_range) == 0) {
        range_lists.insert({ target_range, create_range_list(target_range) });
    }

    auto& list = range_lists[target_range];
    assign_tag(variable_index, list.range_tag);
    list.variable_indices.emplace_back(variable_index);
}
void PlookupComposer::process_range_list(const RangeList& list)
{
    ASSERT(list.variable_indices.size() > 0);
    // go over variables
    // for each variable, create mirror variable with same value - with tau tag
    // need to make sure that, in original list, increments of at most 3
    std::vector<uint64_t> sorted_list;
    sorted_list.reserve(list.variable_indices.size());
    for (const auto variable_index : list.variable_indices) {
        const auto& field_element = get_variable(variable_index);
        const uint64_t shrinked_value = field_element.from_montgomery_form().data[0];
        sorted_list.emplace_back(shrinked_value);
    }
    std::sort(sorted_list.begin(), sorted_list.end());
    std::vector<uint32_t> indices;

    // list must be padded to a multipe of 4 and larger than 4
    size_t padding = (4 - (list.variable_indices.size() % 4)) % 4; // TODO: this 4 maybe tied to program_width
    if (list.variable_indices.size() == 4)
        padding += 4;
    for (size_t i = 0; i < padding; ++i) {
        indices.emplace_back(zero_idx);
    }
    for (const auto sorted_value : sorted_list) {
        const uint32_t index = add_variable(sorted_value);
        assign_tag(index, list.tau_tag);
        indices.emplace_back(index);
    }
    create_sort_constraint_with_edges(indices, 0, list.target_range);
}
void PlookupComposer::process_range_lists()
{
    for (const auto& i : range_lists)
        process_range_list(i.second);
}
/*
 Create range constraint:
  * add variable index to a list of range constrained variables
  * data structures: vector of lists, each list contains:
  *    - the range size
  *    - the list of variables in the range
  *    - a generalised permutation tag
  *
  * create range constraint parameters: variable index && range size
  *
  * std::map<uint64_t, RangeList> range_lists;
*/
// Check for a sequence of variables that neighboring differences are at most 3 (used for batched range checkj)
void PlookupComposer::create_sort_constraint(const std::vector<uint32_t>& variable_index)
{
    PLOOKUP_SELECTOR_REFS
    ASSERT(variable_index.size() % 4 == 0);
    for (size_t i = 0; i < variable_index.size(); i++) {
        ASSERT(static_cast<uint32_t>(variables.size()) > variable_index[i]);
    }

    for (size_t i = 0; i < variable_index.size(); i += 4) {
        ASSERT(variable_index[i] != IS_CONSTANT);
        ASSERT(variable_index[i + 1] != IS_CONSTANT);
        ASSERT(variable_index[i + 2] != IS_CONSTANT);
        ASSERT(variable_index[i + 3] != IS_CONSTANT);
        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++n;
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_5.emplace_back(0);
        q_ecc_1.emplace_back(0);
        q_logic.emplace_back(0);
        q_range.emplace_back(0);
        q_sort.emplace_back(1);
        q_elliptic.emplace_back(0);
        q_lookup_index.emplace_back(0);
        q_lookup_type.emplace_back(0);
    }
    // dummy gate needed because of sort widget's check of next row
    ASSERT(variable_index[variable_index.size() - 1] != IS_CONSTANT);
    w_l.emplace_back(variable_index[variable_index.size() - 1]);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    ++n;
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_logic.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
}
// useful to put variables in the witness that aren't already used - e.g. the dummy variables of the range constraint in
// multiples of three
void PlookupComposer::create_dummy_constraints(const std::vector<uint32_t>& variable_index)
{
    PLOOKUP_SELECTOR_REFS
    // ASSERT(variable_index.size() % 4 == 0);
    std::vector<uint32_t> padded_list = variable_index;
    const uint64_t padding = (4 - (padded_list.size() % 4)) % 4;
    for (uint64_t i = 0; i < padding; ++i) {
        padded_list.emplace_back(zero_idx);
    }
    for (size_t i = 0; i < variable_index.size(); i++) {
        assert(static_cast<uint32_t>(variables.size()) > variable_index[i]);
    }

    for (size_t i = 0; i < padded_list.size(); i += 4) {
        ASSERT(padded_list[i] != IS_CONSTANT);
        ASSERT(padded_list[i + 1] != IS_CONSTANT);
        ASSERT(padded_list[i + 2] != IS_CONSTANT);
        ASSERT(padded_list[i + 3] != IS_CONSTANT);
        w_l.emplace_back(padded_list[i]);
        w_r.emplace_back(padded_list[i + 1]);
        w_o.emplace_back(padded_list[i + 2]);
        w_4.emplace_back(padded_list[i + 3]);
        ++n;
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_5.emplace_back(0);
        q_ecc_1.emplace_back(0);
        q_logic.emplace_back(0);
        q_range.emplace_back(0);
        q_sort.emplace_back(0);
        q_elliptic.emplace_back(0);
        q_lookup_index.emplace_back(0);
        q_lookup_type.emplace_back(0);
    }
}
// Check for a sequence of variables that neighboring differences are at most 3 (used for batched range checks)
void PlookupComposer::create_sort_constraint_with_edges(const std::vector<uint32_t>& variable_index,
                                                        const fr& start,
                                                        const fr& end)
{
    PLOOKUP_SELECTOR_REFS
    // Convenient to assume size is at least 8 for separate gates for start and end conditions
    ASSERT(variable_index.size() % 4 == 0 && variable_index.size() > 4);
    for (size_t i = 0; i < variable_index.size(); i++) {
        ASSERT(static_cast<uint32_t>(variables.size()) > variable_index[i]);
    }
    // enforce range checks of first row and starting at start
    w_l.emplace_back(variable_index[0]);
    w_r.emplace_back(variable_index[1]);
    w_o.emplace_back(variable_index[2]);
    w_4.emplace_back(variable_index[3]);
    ++n;
    q_m.emplace_back(0);
    q_1.emplace_back(1);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(-start);
    q_arith.emplace_back(1);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_logic.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(1);
    q_elliptic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    // enforce range check for middle rows
    for (size_t i = 4; i < variable_index.size() - 4; i += 4) {

        w_l.emplace_back(variable_index[i]);
        w_r.emplace_back(variable_index[i + 1]);
        w_o.emplace_back(variable_index[i + 2]);
        w_4.emplace_back(variable_index[i + 3]);
        ++n;
        q_m.emplace_back(0);
        q_1.emplace_back(0);
        q_2.emplace_back(0);
        q_3.emplace_back(0);
        q_c.emplace_back(0);
        q_arith.emplace_back(0);
        q_4.emplace_back(0);
        q_5.emplace_back(0);
        q_ecc_1.emplace_back(0);
        q_logic.emplace_back(0);
        q_range.emplace_back(0);
        q_sort.emplace_back(1);
        q_elliptic.emplace_back(0);
        q_lookup_index.emplace_back(0);
        q_lookup_type.emplace_back(0);
    }
    // enforce range checks of last row and ending at end
    w_l.emplace_back(variable_index[variable_index.size() - 4]);
    w_r.emplace_back(variable_index[variable_index.size() - 3]);
    w_o.emplace_back(variable_index[variable_index.size() - 2]);
    w_4.emplace_back(variable_index[variable_index.size() - 1]);
    ++n;
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(-end);
    q_arith.emplace_back(1);
    q_4.emplace_back(1);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_logic.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(1);
    q_elliptic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
    // dummy gate needed because of sort widget's check of next row
    w_l.emplace_back(variable_index[variable_index.size() - 1]);
    w_r.emplace_back(zero_idx);
    w_o.emplace_back(zero_idx);
    w_4.emplace_back(zero_idx);
    ++n;
    q_m.emplace_back(0);
    q_1.emplace_back(0);
    q_2.emplace_back(0);
    q_3.emplace_back(0);
    q_c.emplace_back(0);
    q_arith.emplace_back(0);
    q_4.emplace_back(0);
    q_5.emplace_back(0);
    q_ecc_1.emplace_back(0);
    q_logic.emplace_back(0);
    q_range.emplace_back(0);
    q_sort.emplace_back(0);
    q_elliptic.emplace_back(0);
    q_lookup_index.emplace_back(0);
    q_lookup_type.emplace_back(0);
}

// range constraint a value by decomposing it into limbs whose size should be the default range constraint size
std::vector<uint32_t> PlookupComposer::decompose_into_default_range_better_for_oddlimbnum(const uint32_t variable_index,
                                                                                          const size_t num_bits)
{
    std::vector<uint32_t> sums;
    const size_t limb_num = (size_t)num_bits / DEFAULT_PLOOKUP_RANGE_BITNUM;
    const size_t last_limb_size = num_bits - (limb_num * DEFAULT_PLOOKUP_RANGE_BITNUM);
    if (limb_num < 3) {
        std::cout
            << "number of bits in range must be an integer multipe of DEFAULT_PLOOKUP_RANGE_BITNUM of size at least 3"
            << std::endl;
        return sums;
    }

    const uint256_t val = (uint256_t)(get_variable(variable_index));
    // check witness value is indeed in range (commented out cause interferes with negative tests)
    // ASSERT(val < ((uint256_t)1 << num_bits) - 1); // Q:ask Zac what happens with wrapping when converting fr to
    // uint256
    // ASSERT(limb_num % 3 == 0); // TODO: write version of method that doesn't need this
    std::vector<uint32_t> val_limbs;
    std::vector<fr> val_slices;
    for (size_t i = 0; i < limb_num; i++) {
        val_slices.emplace_back(
            barretenberg::fr(val.slice(DEFAULT_PLOOKUP_RANGE_BITNUM * i, DEFAULT_PLOOKUP_RANGE_BITNUM * (i + 1) - 1)));
        val_limbs.emplace_back(add_variable(val_slices[i]));
        create_new_range_constraint(val_limbs[i], DEFAULT_PLOOKUP_RANGE_SIZE);
    }

    uint64_t last_limb_range = ((uint64_t)1 << last_limb_size) - 1;
    fr last_slice(0);
    uint32_t last_limb(zero_idx);
    size_t total_limb_num = limb_num;
    if (last_limb_size > 0) {
        val_slices.emplace_back(fr(val.slice(num_bits - last_limb_size, num_bits)));
        val_limbs.emplace_back(add_variable(last_slice));
        create_new_range_constraint(last_limb, last_limb_range);
        total_limb_num++;
    }
    // pad slices and limbs in case they are not 2 mod 3
    if (total_limb_num % 3 == 1) {
        val_limbs.emplace_back(zero_idx); // TODO: check this is zero
        val_slices.emplace_back(0);
        total_limb_num++;
    }
    fr shift = fr(1 << DEFAULT_PLOOKUP_RANGE_BITNUM);
    fr second_shift = shift * shift;
    sums.emplace_back(add_variable(val_slices[0] + shift * val_slices[1] + second_shift * val_slices[2]));
    create_big_add_gate({ val_limbs[0], val_limbs[1], val_limbs[2], sums[0], 1, shift, second_shift, -1, 0 });
    fr cur_shift = (shift * second_shift);
    fr cur_second_shift = cur_shift * shift;
    for (size_t i = 3; i < total_limb_num; i = i + 2) {
        sums.emplace_back(add_variable(get_variable(sums[sums.size() - 1]) + cur_shift * val_slices[i] +
                                       cur_second_shift * val_slices[i + 1]));
        create_big_add_gate({ sums[sums.size() - 2],
                              val_limbs[i],
                              val_limbs[i + 1],
                              sums[sums.size() - 1],
                              1,
                              cur_shift,
                              cur_second_shift,
                              -1,
                              0 });
        cur_shift *= second_shift;
        cur_second_shift *= second_shift;
    }
    // std::cout << "variable_ind:" << get_variable(variable_index) << " sum:" << get_variable(sums[1]) << std::endl;
    assert_equal(sums[sums.size() - 1], variable_index);
    return sums;
}

} // namespace waffle
