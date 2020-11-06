#include "pedersen.hpp"
#include "pedersen_plookup.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <ecc/curves/grumpkin/grumpkin.hpp>

#include "../../primitives/composers/composers.hpp"
#include "../../primitives/packed_byte_array/packed_byte_array.hpp"

namespace plonk {
namespace stdlib {

using namespace barretenberg;

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
 **/
template <typename C>
point<C> pedersen<C>::hash_single(const field_t& in, const size_t hash_index, const bool validate_edge_cases)
{
    C* ctx = in.context;

    field_t scalar = in;

    if (in.is_constant()) {
        const auto hash_native = crypto::pedersen::hash_single(in.get_value(), hash_index).normalize();
        return { field_t(ctx, hash_native.x), field_t(ctx, hash_native.y) };
    }

    if (!(in.additive_constant == fr::zero()) || !(in.multiplicative_constant == fr::one())) {
        scalar = scalar.normalize();
    }
    ASSERT(ctx != nullptr);
    fr scalar_multiplier = scalar.get_value().from_montgomery_form();

    constexpr size_t num_bits = 254;
    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;

    constexpr size_t initial_exponent = num_bits; // ((num_bits & 1) == 1) ? num_bits - 1: num_bits;
    const crypto::pedersen::fixed_base_ladder* ladder = crypto::pedersen::get_hash_ladder(hash_index, num_bits);
    grumpkin::g1::affine_element generator = crypto::pedersen::get_generator(hash_index * 2 + 1);

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
        ctx->assert_equal(lhs.witness_index, rhs.witness_index);
    } else {
        ctx->assert_equal(add_quad.d, in.witness_index);
    }
    return result;
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
                                 const bool handle_edge_cases)
{
    if constexpr (C::type == waffle::ComposerType::PLOOKUP) {
        return pedersen_plookup<C>::compress(in_left, in_right);
    }

    std::vector<point> accumulators;
    accumulators.push_back(hash_single(in_left, hash_index, handle_edge_cases));
    accumulators.push_back(hash_single(in_right, hash_index + 1, handle_edge_cases));
    if (handle_edge_cases) {
        std::vector<field_t> inputs;
        inputs.push_back(in_left);
        inputs.push_back(in_right);
        return conditionally_accumulate(accumulators, inputs).x;
    }
    return accumulate(accumulators).x;
}

template <typename C>
point<C> pedersen<C>::encrypt(const std::vector<field_t>& inputs, const size_t hash_index, const bool handle_edge_cases)
{
    if constexpr (C::type == waffle::ComposerType::PLOOKUP) {
        return pedersen_plookup<C>::encrypt(inputs);
    }

    std::vector<point> to_accumulate;
    for (size_t i = 0; i < inputs.size(); ++i) {
        to_accumulate.push_back(hash_single(inputs[i].normalize(), hash_index + i, handle_edge_cases));
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
    return encrypt(inputs, hash_index, handle_edge_cases).x;
}

// If the input values are all zero, we return the array length instead of `0`
// This is because we require the inputs to regular pedersen compression function are nonzero (we use this method to
// hash the base layer of our merkle trees)
template <typename C> byte_array<C> pedersen<C>::compress(const byte_array& input)
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
        field_t element = static_cast<field_t>(input.slice(i * bytes_per_element, bytes_to_slice)).normalize();
        elements.emplace_back(element);
    }
    field_t compressed = compress(elements, true);

    bool_t is_zero(true);
    for (const auto& element : elements) {
        is_zero = is_zero && element.is_zero();
    }

    field_t output = field_t(is_zero).madd(field_t(num_bytes) - compressed, compressed);
    return byte_array(output);
}

template <typename C>
point<C> pedersen<C>::compress_to_point(const field_t& in_left, const field_t& in_right, const size_t hash_index)
{
    if constexpr (C::type == waffle::ComposerType::PLOOKUP) {
        return pedersen_plookup<C>::compress_to_point(in_left, in_right);
    }
    point first = hash_single(in_left, hash_index);
    point second = hash_single(in_right, hash_index + 1);
    return add_points(first, second);
}

template class pedersen<waffle::TurboComposer>;
template class pedersen<waffle::PlookupComposer>;

} // namespace stdlib
} // namespace plonk