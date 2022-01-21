#pragma once

#include "../field/field.hpp"
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/composer/composer_base.hpp>
#include <crypto/pedersen/pedersen.hpp>

#include "../../hash/pedersen/pedersen.hpp"
namespace plonk {
namespace stdlib {

using namespace barretenberg;
using namespace crypto::pedersen;

template <typename ComposerContext> class group {
  public:
    template <size_t num_bits> static auto fixed_base_scalar_mul_g1(const field_t<ComposerContext>& in);
    static auto fixed_base_scalar_mul(const field_t<ComposerContext>& lo, const field_t<ComposerContext>& hi);

    template <size_t num_bits>
    static auto fixed_base_scalar_mul(const field_t<ComposerContext>& in,
                                      const size_t generator_index,
                                      const bool forbid_zero_input = true);

  private:
    template <size_t num_bits>
    static auto fixed_base_scalar_mul_internal(const field_t<ComposerContext>& in,
                                               grumpkin::g1::affine_element const& generator,
                                               fixed_base_ladder const* ladder,
                                               const bool forbid_zero_input = true);
};

template <typename ComposerContext>
template <size_t num_bits>
auto group<ComposerContext>::fixed_base_scalar_mul_g1(const field_t<ComposerContext>& in)
{
    const auto ladder = get_g1_ladder(num_bits);
    auto generator = grumpkin::g1::one;
    return group<ComposerContext>::fixed_base_scalar_mul_internal<num_bits>(in, generator, ladder, true);
}

template <typename ComposerContext>
template <size_t num_bits>
auto group<ComposerContext>::fixed_base_scalar_mul(const field_t<ComposerContext>& in,
                                                   const size_t generator_index,
                                                   const bool forbid_zero_input)
{
    // we assume for fixed_base_scalar_mul we're interested in the gen at subindex 0
    generator_index_t index = { generator_index, 0 };
    auto gen_data = get_generator_data(index);
    return group<ComposerContext>::fixed_base_scalar_mul_internal<num_bits>(
        in, gen_data.generator, gen_data.get_ladder(num_bits), forbid_zero_input);
}

/**
 * Perform a fixed base scalar mul over a 258-bit input. Used for schnorr signature verification
 *
 * we decompose lo and hi each into a wnaf form, which validates that both `lo` and `hi` are <= 2^129
 *
 * total scalar is equal to (lo + hi << 128)
 *
 * maximum value is (2^257 + 2^129). Further range constraints are required for more precision
 **/
template <typename ComposerContext>
auto group<ComposerContext>::fixed_base_scalar_mul(const field_t<ComposerContext>& lo,
                                                   const field_t<ComposerContext>& hi)
{
    // This method does not work if lo or hi are 0. We don't apply the extra constraints to handle this edge case
    // (merely rule it out), because we can assume the scalar multipliers for schnorr are uniformly randomly distributed
    (lo * hi).assert_is_not_zero();
    const auto ladder_full = get_g1_ladder(256);
    const auto ladder_low = &ladder_full[64];
    auto generator = grumpkin::g1::one;
    grumpkin::g1::affine_element generator_high = ladder_full[64].one;
    const auto high = fixed_base_scalar_mul_internal<128>(hi, generator_high, ladder_full);
    const auto low = fixed_base_scalar_mul_internal<128>(lo, generator, ladder_low);

    // add high and low. We need to validate high != low. This can occur if `hi = 1` and `low = 2^128`
    const auto x_delta = (high.x - low.x);
    x_delta.assert_is_not_zero();
    const auto lambda = (high.y - low.y) / x_delta;
    const auto x_3 = lambda.madd(lambda, -(high.x + low.x));
    const auto y_3 = lambda.madd((low.x - x_3), -low.y);
    return point<ComposerContext>{ x_3, y_3 };
}

template <typename ComposerContext>
template <size_t num_bits>
auto group<ComposerContext>::fixed_base_scalar_mul_internal(const field_t<ComposerContext>& in,
                                                            grumpkin::g1::affine_element const& generator,
                                                            fixed_base_ladder const* ladder,
                                                            const bool forbid_zero_input)
{
    auto scalar = in.normalize();
    auto ctx = in.context;
    ASSERT(ctx != nullptr);
    uint256_t scalar_multiplier(scalar.get_value());
    if (scalar_multiplier.get_msb() >= num_bits) {
        ctx->failed = true;
        ctx->err = format(
            "fixed_base_scalar_mul scalar multiplier ", scalar_multiplier, " is larger than num_bits ", num_bits);
    }

    // constexpr size_t num_bits = 250;
    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;

    constexpr size_t initial_exponent = ((num_bits & 1) == 1) ? num_bits - 1 : num_bits;

    grumpkin::g1::element origin_points[2];
    origin_points[0] = grumpkin::g1::element(ladder[0].one);
    origin_points[1] = origin_points[0] + generator;
    origin_points[1] = origin_points[1].normalize();

    fr scalar_multiplier_base = scalar.get_value();

    if ((scalar_multiplier.data[0] & 1) == 0) {
        fr two = fr::one() + fr::one();
        scalar_multiplier_base = scalar_multiplier_base - two;
    }
    scalar_multiplier_base = scalar_multiplier_base.from_montgomery_form();
    uint64_t wnaf_entries[num_quads + 1] = { 0 };
    bool skew = false;

    wnaf::fixed_wnaf<num_wnaf_bits, 1, 2>(&scalar_multiplier_base.data[0], &wnaf_entries[0], skew, 0);

    fr accumulator_offset = (fr::one() + fr::one()).pow(static_cast<uint64_t>(initial_exponent)).invert();

    fr origin_accumulators[2]{ fr::one(), accumulator_offset + fr::one() };

    grumpkin::g1::element* multiplication_transcript =
        static_cast<grumpkin::g1::element*>(aligned_alloc(64, sizeof(grumpkin::g1::element) * (num_quads + 1)));
    fr* accumulator_transcript = static_cast<fr*>(aligned_alloc(64, sizeof(fr) * (num_quads + 1)));

    if (skew) {
        multiplication_transcript[0] = origin_points[1];
        accumulator_transcript[0] = origin_accumulators[1];
    } else {
        multiplication_transcript[0] = origin_points[0];
        accumulator_transcript[0] = origin_accumulators[0];
    }
    fr one = fr::one();
    fr three = ((one + one) + one);

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

    if (num_bits >= 254) {
        plonk::stdlib::pedersen<ComposerContext>::validate_wnaf_is_in_field(ctx, accumulator_witnesses, scalar, true);
    }
    aligned_free(multiplication_transcript);
    aligned_free(accumulator_transcript);

    auto constructed_scalar = field_t(ctx);
    constructed_scalar.witness_index = add_quad.d;

    point<ComposerContext> result;
    result.x = field_t(ctx);
    result.x.witness_index = add_quad.a;
    result.y = field_t(ctx);
    result.y.witness_index = add_quad.b;

    auto lhs = constructed_scalar;
    auto rhs = scalar;
    if (!forbid_zero_input) {
        // if forbid_zero_input == false, it is up to the calling function to correctly handle the case where the input
        // = 0
        auto is_zero = scalar.is_zero();
        // If k = 0, our scalar multiplier is going to be nonsense.
        // We need to conditionally validate that, if k != 0, the constructed scalar multiplier matches our input
        // scalar.
        lhs = constructed_scalar * (field_t<ComposerContext>(1) - field_t<ComposerContext>(is_zero));
        rhs = scalar * (field_t<ComposerContext>(1) - field_t<ComposerContext>(is_zero));
    }
    lhs.normalize();
    rhs.normalize();
    ctx->assert_equal(lhs.witness_index, rhs.witness_index, "scalars unequal");

    return result;
}
// static point fixed_base_mul(const field_t& s);
// static point add(const point& left, const point& right);
// static point variable_base_mul(const point& p, const field_t& s);

} // namespace stdlib
} // namespace plonk