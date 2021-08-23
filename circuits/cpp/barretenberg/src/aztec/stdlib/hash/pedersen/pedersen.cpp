#include "pedersen.hpp"
#include "pedersen_plookup.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

#include "../../primitives/composers/composers.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace plonk {
namespace stdlib {

using namespace barretenberg;
using namespace crypto::pedersen;

namespace {
template <typename C> point<C> add_points(const point<C>& first, const point<C>& second)
{
    field_t<C> lhs = second.y - first.y;
    field_t<C> rhs = second.x - first.x;
    field_t<C> lambda = lhs / rhs;
    field_t<C> x_3 = lambda * lambda - second.x - first.x;
    field_t<C> y_3 = lambda * (first.x - x_3) - first.y;
    return { x_3, y_3 };
}
} // namespace

/**
 * edge cases are if scalar multiplier is 1 or 0
 * not neccessary to check if scalar multiplier is the output of a PRNG (e.g. SHA256)
 * Description of function:
 * We begin with an fr element in, and create a wnaf representation of it (see validate_wnaf_is_in_field for detail on
 * this presentation, or page 4 in https://docs.zkproof.org/pages/standards/accepted-workshop3/proposal-turbo_plonk.pdf)
 * This representation gives a sequence of 127 quads q_0,...,q_126 in the range {-1,1,-3,3} and an additional skew bit.
 * We take two generators g1,g2 according to hash_index.
 * Define the scalars A = sum_{i=0}^124 q_i 4^{124-i} + 4^{125},   B=q_{125}*4 +q_{126}+skew
 * We output A*[g1] + B*[g2]. Since A is smaller than p/2, p being the grumpkin curve order, this can be
 * shown to be CR under DL even when later outputting only the x coordinate.
 **/

template <typename C>
point<C> pedersen<C>::hash_single(const field_t& in,
                                  const generator_index_t hash_index,
                                  const bool validate_edge_cases,
                                  const bool validate_input_is_in_field)
{
    C* ctx = in.context;

    field_t scalar = in.normalize();

    if (in.is_constant()) {
        const auto hash_native = crypto::pedersen::hash_single(in.get_value(), hash_index).normalize();
        return { field_t(ctx, hash_native.x), field_t(ctx, hash_native.y) };
    }

    ASSERT(ctx != nullptr);
    fr scalar_multiplier = scalar.get_value().from_montgomery_form();

    constexpr size_t num_bits = 254;
    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;

    constexpr size_t initial_exponent = num_bits; // ((num_bits & 1) == 1) ? num_bits - 1: num_bits;
    const auto gen_data = crypto::pedersen::get_generator_data(hash_index);
    const crypto::pedersen::fixed_base_ladder* ladder = gen_data.get_hash_ladder(num_bits);
    grumpkin::g1::affine_element generator = gen_data.aux_generator;

    grumpkin::g1::element origin_points[2];
    origin_points[0] = grumpkin::g1::element(ladder[0].one);
    origin_points[1] = origin_points[0] + generator;
    origin_points[1] = origin_points[1].normalize();

    fr scalar_multiplier_base = scalar_multiplier.to_montgomery_form();

    if ((scalar_multiplier.data[0] & 1) == 0) {
        fr two = fr::one() + fr::one();
        scalar_multiplier_base = scalar_multiplier_base - two;
    }
    scalar_multiplier_base = scalar_multiplier_base.from_montgomery_form();
    uint64_t wnaf_entries[num_quads + 1] = { 0 };
    bool skew = false;

    barretenberg::wnaf::fixed_wnaf<num_wnaf_bits, 1, 2>(&scalar_multiplier_base.data[0], &wnaf_entries[0], skew, 0);

    fr accumulator_offset = (fr::one() + fr::one()).pow(static_cast<uint64_t>(initial_exponent)).invert();

    fr origin_accumulators[2]{ fr::one(), accumulator_offset + fr::one() };

    std::vector<grumpkin::g1::element> multiplication_transcript;
    multiplication_transcript.resize(num_quads + 1);
    std::vector<fr> accumulator_transcript;
    accumulator_transcript.resize(num_quads + 1);

    if (skew) {
        multiplication_transcript[0] = origin_points[1];
        accumulator_transcript[0] = origin_accumulators[1];
    } else {
        multiplication_transcript[0] = origin_points[0];
        accumulator_transcript[0] = origin_accumulators[0];
    }
    constexpr fr one = fr::one();
    constexpr fr three = ((one + one) + one);

    for (size_t i = 0; i < num_quads; ++i) {
        uint64_t entry = wnaf_entries[i + 1] & 0xffffff;

        fr prev_accumulator = accumulator_transcript[i] + accumulator_transcript[i];
        prev_accumulator = prev_accumulator + prev_accumulator;

        grumpkin::g1::affine_element point_to_add = (entry == 1) ? ladder[i + 1].three : ladder[i + 1].one;

        fr scalar_to_add = (entry == 1) ? three : one;
        uint64_t predicate = (wnaf_entries[i + 1] >> 31U) & 1U;
        if (predicate) {
            point_to_add = -point_to_add;
            scalar_to_add.self_neg();
        }
        accumulator_transcript[i + 1] = prev_accumulator + scalar_to_add;
        multiplication_transcript[i + 1] = multiplication_transcript[i] + point_to_add;
    }

    grumpkin::g1::element::batch_normalize(&multiplication_transcript[0], num_quads + 1);

    waffle::fixed_group_init_quad init_quad{ origin_points[0].x,
                                             (origin_points[0].x - origin_points[1].x),
                                             origin_points[0].y,
                                             (origin_points[0].y - origin_points[1].y) };

    fr x_alpha = accumulator_offset;
    std::vector<uint32_t> accumulator_witnesses;
    for (size_t i = 0; i < num_quads; ++i) {
        waffle::fixed_group_add_quad round_quad;
        round_quad.d = ctx->add_variable(accumulator_transcript[i]);
        round_quad.a = ctx->add_variable(multiplication_transcript[i].x);
        round_quad.b = ctx->add_variable(multiplication_transcript[i].y);

        if (i == 0) {
            // we need to ensure that the first value of x_alpha is a defined constant.
            // However, repeated applications of the pedersen hash will use the same constant value.
            // `put_constant_variable` will create a gate that fixes the value of x_alpha, but only once
            round_quad.c = ctx->put_constant_variable(x_alpha);
        } else {
            round_quad.c = ctx->add_variable(x_alpha);
        }
        if ((wnaf_entries[i + 1] & 0xffffffU) == 0) {
            x_alpha = ladder[i + 1].one.x;
        } else {
            x_alpha = ladder[i + 1].three.x;
        }
        round_quad.q_x_1 = ladder[i + 1].q_x_1;
        round_quad.q_x_2 = ladder[i + 1].q_x_2;
        round_quad.q_y_1 = ladder[i + 1].q_y_1;
        round_quad.q_y_2 = ladder[i + 1].q_y_2;

        if (i > 0) {
            ctx->create_fixed_group_add_gate(round_quad);
        } else {
            ctx->create_fixed_group_add_gate_with_init(round_quad, init_quad);
        }

        accumulator_witnesses.push_back(round_quad.d);
    }

    waffle::add_quad add_quad{ ctx->add_variable(multiplication_transcript[num_quads].x),
                               ctx->add_variable(multiplication_transcript[num_quads].y),
                               ctx->add_variable(x_alpha),
                               ctx->add_variable(accumulator_transcript[num_quads]),
                               fr::zero(),
                               fr::zero(),
                               fr::zero(),
                               fr::zero(),
                               fr::zero() };
    ctx->create_big_add_gate(add_quad);
    accumulator_witnesses.push_back(add_quad.d);

    point result;
    result.x = field_t(ctx);
    result.x.witness_index = add_quad.a;
    result.y = field_t(ctx);
    result.y.witness_index = add_quad.b;

    if (validate_edge_cases) {
        field_t reconstructed_scalar(ctx);
        reconstructed_scalar.witness_index = add_quad.d;
        field_t lhs = reconstructed_scalar * in;
        field_t rhs = in * in;
        lhs.assert_equal(rhs, "pedersen lhs != rhs");
    } else {
        field_t::from_witness_index(ctx, add_quad.d).assert_equal(in, "pedersen d != in");
    }

    if (validate_input_is_in_field) {
        validate_wnaf_is_in_field(ctx, accumulator_witnesses, in, validate_edge_cases);
    }
    return result;
}

/**
 * Check the wnaf sum is smaller than the circuit modulus
 *
 * When we compute a scalar mul e.g. x * [1], we decompose `x` into an accumulating sum of 2-bit non-adjacent form
 *values. In `hash_single`, we validate that the sum of the 2-bit NAFs (`w`) equals x. But we only check that `w == x
 *mod r` where r is the circuit modulus.
 *
 * If we require the pedersen hash to be injective, we must ensure that `w < r`.
 * Typically this is required for all instances where `w` represents a field element.
 * One exception is Merkle tree membership proofs as there is only one valid output that will hash to the Merkle root
 *
 * Total cost is ~36 gates
 **/
template <typename C>
void pedersen<C>::validate_wnaf_is_in_field(C* ctx,
                                            const std::vector<uint32_t>& accumulator,
                                            const field_t& in,
                                            const bool validate_edge_cases)
{
    /**
     * To validate that `w < r`, we use schoolbook subtraction
     *
     * The wnaf entries, other than the last entry, are in the range [-3, -1, 1, 3]
     *
     *                                                                 -254
     * The last wnaf entry, wnaf[127] is taken from the range [1, 1 + 2    ]
     *
     *        127
     *        ===
     *        \                i
     *  w =   /    wnaf[i]  . 4
     *        ===
     *       i = 0
     *                                               255
     * The final value of w can range between 1 and 2
     *
     *      -254
     * The 2     term is the 'wnaf skew'. Only odd integers can be represented via a wnaf. The skew is an
     * additional value that is added into the wnaf sum to enable even integer representation.
     *
     * N.B. We do not consider the case where the input is equal to 0. This is a special edge case that must
     *      be handled separately because of affine addition formulae exceptions.
     *
     * The raw wnaf entries are not themselves represented as witnesses in the circuit.
     * The pedersen hash gate derives the wnaf entries by taking the difference between two accumulating sums.
     * We accumulate starting with the MOST significant wnaf entry
     *
     * i.e. there is a container of witnesses, `accumulators[128]`, where:
     *
     *
     *                      i
     *                     ===
     *                     \                      i - j
     *  accumulator[i] =   /    wnaf[127 - j]  . 4
     *                     ===
     *                    j = 0
     *
     * The goal is to ensure that accumulator[127] < r using as few constraints as possible
     * The following describes how we implement this check:
     *
     * 1. Use the wnaf accumulator to split `w` into two limbs w.lo and w.hi
     *
     *    w.lo is the accumulating sum of the least significant 63 wnaf entries, plus the wnaf skew (0 or 1)
     *    w.hi is the accumulating sum of the most significant 64 wnaf entries excluding the wnaf skew
     *
     *    We can extract w.hi from accumulator[64], but we need to remove the contribution from the wnaf skew
     *    We can extract w.lo by subtracting w.hi * 2^{126} from the final accumulator (the final accumulator will be
     *equal to `w`)
     *
     * 2. Compute y.lo = (r.lo - w.lo) + 2^{126} (the 2^126 constant ensures this is positive)
     *    r.lo is the least significant 126 bits of r
     *    r.hi is the most significant 128 bits of r
     *
     * 4. Compute y.overlap = y.lo.slice(126, 128) - 1
     *    (we can get this from applying a 128-bit range constraint to y.lo && extract the most significant quad)
     *    y.overlap is a 2-bit integer and *NOT* a 1-bit integer. This is because w.lo can be negative
     *    y.overlap represents the 2 bits of y.lo that overlap with y.hi
     *    We subtract 1 to counter the constant 2^{126} term we added into y.lo
     *
     * 5. Compute y.hi = r.hi - w.hi + y.overlap
     *
     * 6. Range constrain y.hi to be a 128-bit integer
     *
     * We slice the low limb to be 126 bits so that both our range checks can be over 128-bit integers (if the range is
     *a multiple of 8 we save 1 gate per range check)
     *
     * The following table describes the range of values the above terms can take, if w < r
     *
     *   ----------------------------------------------
     *   | limb               | min value | max value |
     *   ----------------------------------------------
     *   |                    |     126   |  126      |
     *   | w.lo               |  -(2  - 1)| 2         |
     *   ----------------------------------------------
     *   |                    |           |  129      |
     *   | w.hi               |         1 | 2   - 1   |
     *   ----------------------------------------------
     *   |                126 |           |  255      |
     *   | w.lo + w.hi * 2    |         1 | 2         |
     *   ----------------------------------------------
     *   |                    |    126    |    128    |
     *   | y.lo               | > 2   - 1 | < 2       |
     *   ----------------------------------------------
     *   |                    |           |    128    |
     *   | y.hi               |         0 | < 2       |
     *   ----------------------------------------------
     *
     * Possible result states and the conditions that must be satisfied:
     *
     *   ------------------------------------------------------------------------------------------------------------------------------------------------------
     *   | condition               | (r.lo - w.lo + 2^126) >> 126 | status of low limb | does p.lo overlap with p.hi? |
     *condition for w < r            |
     *   ------------------------------------------------------------------------------------------------------------------------------------------------------
     *   | w.lo > r.lo             | 0                            | negative           | yes, p.lo borrows 1 from p.hi
     *| (r.hi - w.hi - 1) must be >= 0 | | w.lo <= r.lo, w.lo >= 0 | 1                            | positive           |
     *no                                  | (r.hi - w.hi) must be >= 0     | | w.lo << 0               | 2 | positive |
     *yes, r.lo carries 1 to r.hi         | (r.hi - w.hi + 1) must be >= 0 |
     *   ------------------------------------------------------------------------------------------------------------------------------------------------------
     **/

    constexpr uint256_t modulus = fr::modulus;
    const fr r_lo = modulus.slice(0, 126);
    const fr r_hi = modulus.slice(126, 256);
    const fr shift = fr(uint256_t(1) << 126);

    // Step 1: convert accumulator into two 126/128 bit limbs
    uint32_t mid_index = accumulator[64];
    uint32_t end_index = accumulator[accumulator.size() - 1];

    /**
     * We need to extract the skew term from accumulator[0]
     *
     * We know that accumulator[0] is either 1 or (1 + 2^{-254})
     *
     * Therefore  the 2^{-254} term in accumulator[0] will translate to a value of `1` when `input` is computed
     * This corresponds to `input` being an even number (without a skew term, wnaf represenatations can only express odd
     *numbers)
     *
     * We need to factor out this skew term from w.hi as it is part of w.lo
     *
     *
     **/

    // is_even = 0 if input is odd
    // is_even = 1 if input is even
    field_t is_even = (field_t::from_witness_index(ctx, accumulator[0]) - 1) * fr(uint256_t(1) << 254);

    field_t high_limb_with_skew = field_t::from_witness_index(ctx, mid_index);

    // Reconstructed_input will equal input (this is checked in the pedersen hash function)
    // We extract term from the accumulators because input might have constant scaling factors applied to it
    field_t reconstructed_input = field_t::from_witness_index(ctx, end_index);

    /**
     *                                                         126
     *    w.lo = reconstructed_input - (high_limb_with_skew * 2  - is_even)
     *                          126
     *    y.lo = r.lo - w.lo + 2
     *                   126
     * => y.lo = r.lo + 2    - is_even - reconstructed_input + high_limb_with_skew
     *
     *  (we do not explicitly compute w.lo to save an addition gate)
     **/

    field_t y_lo = (-reconstructed_input).add_two(high_limb_with_skew * shift + (r_lo + shift), -is_even);

    /**
     * If `validate_edge_cases = true`, we need to handle the possibility the input is zero
     *
     * If the input is zero, the produced wnaf will be nonsense and may not be < r
     * (the wnaf is not used when computing a pedersen hash of 0, additional constraints are used to handle this edge
     *case). If the input is zero we must set `y.lo` and `y.hi` to 0 so the range checks do not fail
     **/
    bool_t input_not_zero;
    if (validate_edge_cases) {
        input_not_zero = !in.is_zero();
        y_lo *= input_not_zero;
    }

    // Validate y.lo is a 128-bit integer
    const auto y_lo_accumulators = ctx->decompose_into_base4_accumulators(y_lo.normalize().witness_index, 128);

    // Extract y.overlap, the 2 most significant bits of y.lo
    field_t y_overlap = field_t::from_witness_index(ctx, y_lo_accumulators[0]) - 1;

    /**
     *                                           -126
     *   w.hi = high_limb_with_skew - is_even * 2
     *
     *   y.hi = r.hi + (y.overlap - 1) - w.hi
     **/
    field_t y_hi = (is_even * fr(uint256_t(1) << 126).invert()).add_two(-high_limb_with_skew, y_overlap + (r_hi));
    if (validate_edge_cases) {
        y_hi *= input_not_zero;
    }

    // Validate y.hi is a 128-bit integer
    ctx->decompose_into_base4_accumulators(y_hi.normalize().witness_index, 128);
}

template <typename C> point<C> pedersen<C>::accumulate(const std::vector<point>& to_accumulate)
{
    if (to_accumulate.size() == 0) {
        return point{ 0, 0 };
    }

    point accumulator = to_accumulate[0];
    for (size_t i = 1; i < to_accumulate.size(); ++i) {
        accumulator = add_points(accumulator, to_accumulate[i]);
    }
    return accumulator;
}

template <typename C>
point<C> pedersen<C>::conditionally_accumulate(const std::vector<point>& to_accumulate,
                                               const std::vector<field_t>& inputs)
{
    if (to_accumulate.size() == 0) {
        return point{ 0, 0 };
    }

    point accumulator = to_accumulate[0];
    bool_t is_accumulator_zero = inputs[0].is_zero();

    for (size_t i = 1; i < to_accumulate.size(); ++i) {
        bool_t current_is_zero = inputs[i].is_zero();
        bool_t initialize_instead_of_add = (is_accumulator_zero && !current_is_zero);

        field_t lambda = (to_accumulate[i].y - accumulator.y) / (to_accumulate[i].x - accumulator.x);
        field_t x_3 = lambda * lambda - (to_accumulate[i].x + accumulator.x);
        field_t y_3 = lambda * (accumulator.x - x_3) - accumulator.y;

        x_3 = (to_accumulate[i].x - x_3).madd(initialize_instead_of_add, x_3);
        y_3 = (to_accumulate[i].y - y_3).madd(initialize_instead_of_add, y_3);
        x_3 = (accumulator.x - x_3).madd(current_is_zero, x_3);
        y_3 = (accumulator.y - y_3).madd(current_is_zero, y_3);
        accumulator.x = x_3;
        accumulator.y = y_3;
        is_accumulator_zero = is_accumulator_zero && current_is_zero;
    }

    accumulator.x = (field_t(0) - accumulator.x).madd(is_accumulator_zero, accumulator.x);
    return accumulator;
}

template <typename C>
field_t<C> pedersen<C>::compress(const field_t& in_left,
                                 const field_t& in_right,
                                 const size_t hash_index,
                                 const bool handle_edge_cases,
                                 const bool validate_input_is_in_field)
{
    if constexpr (C::type == waffle::ComposerType::PLOOKUP) {
        return pedersen_plookup<C>::compress(in_left, in_right);
    }

    std::vector<point> accumulators;
    generator_index_t index_1 = { hash_index, 0 };
    generator_index_t index_2 = { hash_index, 1 };
    accumulators.push_back(hash_single(in_left, index_1, handle_edge_cases, validate_input_is_in_field));
    accumulators.push_back(hash_single(in_right, index_2, handle_edge_cases, validate_input_is_in_field));
    if (handle_edge_cases) {
        std::vector<field_t> inputs;
        inputs.push_back(in_left);
        inputs.push_back(in_right);
        return conditionally_accumulate(accumulators, inputs).x;
    }
    return accumulate(accumulators).x;
}

template <typename C>
point<C> pedersen<C>::commit(const std::vector<field_t>& inputs, const size_t hash_index, const bool handle_edge_cases)
{
    if constexpr (C::type == waffle::ComposerType::PLOOKUP) {
        return pedersen_plookup<C>::commit(inputs);
    }

    std::vector<point> to_accumulate;
    for (size_t i = 0; i < inputs.size(); ++i) {
        generator_index_t index = { hash_index, i };
        to_accumulate.push_back(hash_single(inputs[i], index, handle_edge_cases));
    }
    if (handle_edge_cases) {
        return conditionally_accumulate(to_accumulate, inputs);
    }
    return accumulate(to_accumulate);
}

template <typename C>
field_t<C> pedersen<C>::compress(const std::vector<field_t>& inputs,
                                 const bool handle_edge_cases,
                                 const size_t hash_index)
{
    if (C::type == waffle::ComposerType::PLOOKUP) {
        // TODO handle hash index in plookup. This is a tricky problem but
        // we can defer solving it until we migrate to UltraPlonk
        return pedersen_plookup<C>::compress(inputs);
    }
    return commit(inputs, hash_index, handle_edge_cases).x;
}

// If the input values are all zero, we return the array length instead of `0`
// This is because we require the inputs to regular pedersen compression function are nonzero (we use this method to
// hash the base layer of our merkle trees)
template <typename C> field_t<C> pedersen<C>::compress(const byte_array& input)
{
    if constexpr (C::type == waffle::ComposerType::PLOOKUP) {
        return pedersen_plookup<C>::compress(packed_byte_array(input));
    }
    const size_t num_bytes = input.size();
    const size_t bytes_per_element = 31;
    size_t num_elements = (num_bytes % bytes_per_element != 0) + (num_bytes / bytes_per_element);

    std::vector<field_t> elements;
    for (size_t i = 0; i < num_elements; ++i) {
        size_t bytes_to_slice = 0;
        if (i == num_elements - 1) {
            bytes_to_slice = num_bytes - (i * bytes_per_element);
        } else {
            bytes_to_slice = bytes_per_element;
        }
        field_t element = static_cast<field_t>(input.slice(i * bytes_per_element, bytes_to_slice));
        elements.emplace_back(element);
    }
    field_t compressed = compress(elements, true, 0);

    bool_t is_zero(true);
    for (const auto& element : elements) {
        is_zero = is_zero && element.is_zero();
    }

    field_t output = field_t(is_zero).madd(field_t(num_bytes) - compressed, compressed);
    return output;
}

template <typename C>
point<C> pedersen<C>::compress_to_point(const field_t& in_left, const field_t& in_right, const size_t hash_index)
{
    if constexpr (C::type == waffle::ComposerType::PLOOKUP) {
        return pedersen_plookup<C>::compress_to_point(in_left, in_right);
    }
    generator_index_t index_1 = { hash_index, 0 };
    generator_index_t index_2 = { hash_index, 1 };
    point first = hash_single(in_left, index_1);
    point second = hash_single(in_right, index_2);
    return add_points(first, second);
}

template class pedersen<waffle::TurboComposer>;
template class pedersen<waffle::PlookupComposer>;

} // namespace stdlib
} // namespace plonk