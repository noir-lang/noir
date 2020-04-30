#include "pedersen_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>

namespace rollup {
namespace pedersen_note {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

struct note_triple {
    point base;
    field_ct scalar;
};

template <size_t num_bits> note_triple fixed_base_scalar_mul(const field_ct& in, const size_t generator_index)
{
    field_ct scalar = in;
    if (!(in.additive_constant == fr::zero()) || !(in.multiplicative_constant == fr::one())) {
        scalar = scalar.normalize();
    }
    Composer* ctx = in.context;
    ASSERT(ctx != nullptr);
    fr scalar_multiplier = scalar.get_value().from_montgomery_form();

    // constexpr size_t num_bits = 250;
    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;

    size_t initial_exponent = ((num_bits & 1) == 1) ? num_bits - 1 : num_bits;
    const crypto::pedersen::fixed_base_ladder* ladder = crypto::pedersen::get_ladder(generator_index, num_bits);
    grumpkin::g1::affine_element generator = crypto::pedersen::get_generator(generator_index);

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

    note_triple result;
    result.base.x = field_ct(ctx);
    result.base.x.witness_index = add_quad.a;
    result.base.y = field_ct(ctx);
    result.base.y.witness_index = add_quad.b;
    result.scalar = field_ct(ctx);
    result.scalar.witness_index = add_quad.d;

    aligned_free(multiplication_transcript);
    aligned_free(accumulator_transcript);

    return result;
}

public_note encrypt_note(const private_note& plaintext)
{
    Composer* context = plaintext.value.get_context();

    field_ct k = static_cast<field_ct>(plaintext.value);

    note_triple p_1 = fixed_base_scalar_mul<32>(k, 0);
    note_triple p_2 = fixed_base_scalar_mul<250>(plaintext.secret, 1);

    context->assert_equal(p_2.scalar.witness_index, plaintext.secret.witness_index);

    // if k = 0, then k * inv - 1 != 0
    // k * inv - (1 - is_zero)
    field_ct one(context, fr::one());
    bool_ct is_zero = k.is_zero();

    // If k = 0, our scalar multiplier is going to be nonsense.
    // We need to conditionally validate that, if k != 0, the constructed scalar multiplier matches our input scalar.
    field_ct lhs = p_1.scalar * (one - field_ct(is_zero));
    field_ct rhs = k * (one - field_ct(is_zero));
    lhs.normalize();
    rhs.normalize();
    context->assert_equal(lhs.witness_index, rhs.witness_index);

    // If k = 0 we want to return p_2.base, as g^{0} = 1
    // If k != 0, we want to return p_1.base + p_2.base
    field_ct lambda = (p_2.base.y - p_1.base.y) / (p_2.base.x - p_1.base.x);
    field_ct x_3 = (lambda * lambda) - (p_2.base.x + p_1.base.x);
    field_ct y_3 = lambda * (p_1.base.x - x_3) - p_1.base.y;

    field_ct x_4 = (p_2.base.x - x_3) * field_ct(is_zero) + x_3;
    field_ct y_4 = (p_2.base.y - y_3) * field_ct(is_zero) + y_3;

    point p_3 = plonk::stdlib::pedersen::compress_to_point(plaintext.owner.x, plaintext.owner.y);

    field_ct lambda_out = (p_3.y - y_4) / (p_3.x - x_4);
    field_ct x_out = (lambda_out * lambda_out) - (p_3.x + x_4);
    field_ct y_out = lambda_out * (x_4 - x_out) - y_4;
    x_out = x_out.normalize();
    y_out = y_out.normalize();

    public_note ciphertext{ { x_out, y_out } };
    return ciphertext;
}

template note_triple fixed_base_scalar_mul<32>(const field_ct& in, const size_t generator_index);
template note_triple fixed_base_scalar_mul<250>(const field_ct& in, const size_t generator_index);

} // namespace pedersen_note
} // namespace rollup