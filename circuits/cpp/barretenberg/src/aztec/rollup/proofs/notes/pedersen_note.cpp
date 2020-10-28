#include "pedersen_note.hpp"
#include <crypto/pedersen/pedersen.hpp>
#include <stdlib/hash/pedersen/pedersen.hpp>
#include "./note_generator_indices.hpp"

namespace rollup {
namespace proofs {
namespace notes {

using namespace barretenberg;
using namespace plonk::stdlib::types::turbo;

struct note_triple {
    point_ct base;
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

    if (scalar_multiplier.get_msb() >= num_bits) {
        ctx->failed = true;
        ctx->err = format(
            "fixed_base_scalar_mul scalar multiplier ", scalar_multiplier, " is larger than num_bits ", num_bits);
    }

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

// compute a pedersen hash of `scalar` and add the resulting point into `accumulator`, iff scalar != 0
template <size_t num_scalar_mul_bits>
note_triple conditionally_hash_and_accumulate(Composer* context, const note_triple& accumulator, const field_ct& scalar, const size_t generator_index)
{
    note_triple p_1 = fixed_base_scalar_mul<num_scalar_mul_bits>(scalar, generator_index);

    bool_ct is_zero = scalar.is_zero();
    // If k = 0, our scalar multiplier is going to be nonsense.
    // We need to conditionally validate that, if k != 0, the constructed scalar multiplier matches our input scalar.
    field_ct lhs = p_1.scalar * (field_ct(1) - field_ct(is_zero));
    field_ct rhs = scalar * (field_ct(1) - field_ct(is_zero));
    lhs.normalize();
    rhs.normalize();
    context->assert_equal(lhs.witness_index, rhs.witness_index, "conditional hash and accumulate assert equal failure");

    // If scalar = 0 we want to return accumulator, as g^{0} = 1
    // If scalar != 0, we want to return accumulator + p_1
    field_ct lambda = (accumulator.base.y - p_1.base.y) / (accumulator.base.x - p_1.base.x);
    field_ct x_2 = (lambda * lambda) - (accumulator.base.x + p_1.base.x);
    field_ct y_2 = lambda * (p_1.base.x - x_2) - p_1.base.y;

    x_2 = (accumulator.base.x - x_2) * field_ct(is_zero) + x_2;
    y_2 = (accumulator.base.y - y_2) * field_ct(is_zero) + y_2;
    return { { x_2, y_2 }, scalar };
}

note_triple accumulate(const note_triple& accumulator, const point_ct& p_1)
{
    field_ct lambda = (p_1.y - accumulator.base.y) / (p_1.x - accumulator.base.x);
    field_ct x_2 = (lambda * lambda) - (p_1.x + accumulator.base.x);
    field_ct y_2 = lambda * (accumulator.base.x - x_2) - accumulator.base.y;
    return {{ x_2, y_2 }, accumulator.scalar };
}

/**
 * Compute a pedersen hash of the plaintext:
 * [output] = plaintext.value * [g0] + plaintext.secret * [g1] + plaintext.asset_id * [g2] + plaintext.owner.x * [g3] + plaintext.owner.y * [g4]
 **/ 
public_note encrypt_note(const private_note& plaintext)
{
    Composer* context = plaintext.value.get_context();

    note_triple accumulator = fixed_base_scalar_mul<250>(plaintext.secret, TX_NOTE_HASH_INDEX + 1);
    context->assert_equal(accumulator.scalar.witness_index, plaintext.secret.witness_index, "pedersen_note::encrypt_note assert equal fail");
    accumulator = conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(context, accumulator, plaintext.value, TX_NOTE_HASH_INDEX);
    accumulator = conditionally_hash_and_accumulate<32>(context, accumulator, plaintext.asset_id, TX_NOTE_HASH_INDEX + 2);
    accumulator = accumulate(accumulator, pedersen::compress_to_point(plaintext.owner.x, plaintext.owner.y, TX_NOTE_HASH_INDEX + 3));

    public_note ciphertext{ accumulator.base };
    return ciphertext;
}

field_ct compute_nullifier(const private_note& plaintext, const public_note& ciphertext, const field_ct& tree_index, const bool_ct& is_real_note)
{
    // modified_index = tree_index plus a modifier to indicate whether the note is a real note or a virtual note (i.e. value 0 and not a member of the tree)
    // For virtual notes, we set the 65'th bit of modified_index to be true (this cannot overlap with tree index, which we range constrain to be 32 bits)
    barretenberg::fr shift = uint256_t(1) << 64;
    field_ct modified_index = (tree_index + (static_cast<field_ct>(is_real_note) * shift)).normalize();
    std::vector<field_ct> hash_inputs{
        ciphertext.ciphertext.x,
        plaintext.secret,
        modified_index,
    };

    const auto result = pedersen::compress(hash_inputs, true, TX_NOTE_NULLIFIER_INDEX);
    return result;
}

template note_triple fixed_base_scalar_mul<32>(const field_ct& in, const size_t generator_index);
template note_triple fixed_base_scalar_mul<NOTE_VALUE_BIT_LENGTH>(const field_ct& in, const size_t generator_index);
template note_triple fixed_base_scalar_mul<250>(const field_ct& in, const size_t generator_index);

template note_triple conditionally_hash_and_accumulate<32>(Composer* context, const note_triple& accumulator, const field_ct& scalar, const size_t generator_index);
template note_triple conditionally_hash_and_accumulate<NOTE_VALUE_BIT_LENGTH>(Composer* context, const note_triple& accumulator, const field_ct& scalar, const size_t generator_index);
template note_triple conditionally_hash_and_accumulate<250>(Composer* context, const note_triple& accumulator, const field_ct& scalar, const size_t generator_index);

} // namespace notes
} // namespace proofs
} // namespace rollup
