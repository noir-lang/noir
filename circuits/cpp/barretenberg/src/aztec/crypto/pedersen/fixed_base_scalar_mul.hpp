#pragma once
#include <ecc/curves/grumpkin/grumpkin.hpp>

namespace crypto {
namespace pedersen {

template <size_t num_bits>
grumpkin::g1::element fixed_base_scalar_mul(const barretenberg::fr& in, const size_t generator_index)
{
    auto gen_data = get_generator_data({ generator_index, 0 });
    barretenberg::fr scalar_multiplier = in.from_montgomery_form();

    constexpr size_t num_quads_base = (num_bits - 1) >> 1;
    constexpr size_t num_quads = ((num_quads_base << 1) + 1 < num_bits) ? num_quads_base + 1 : num_quads_base;
    constexpr size_t num_wnaf_bits = (num_quads << 1) + 1;

    const crypto::pedersen::fixed_base_ladder* ladder = gen_data.get_ladder(num_bits);

    barretenberg::fr scalar_multiplier_base = scalar_multiplier.to_montgomery_form();
    if ((scalar_multiplier.data[0] & 1) == 0) {
        barretenberg::fr two = barretenberg::fr::one() + barretenberg::fr::one();
        scalar_multiplier_base = scalar_multiplier_base - two;
    }
    scalar_multiplier_base = scalar_multiplier_base.from_montgomery_form();
    uint64_t wnaf_entries[num_quads + 2] = { 0 };
    bool skew = false;
    barretenberg::wnaf::fixed_wnaf<num_wnaf_bits, 1, 2>(&scalar_multiplier_base.data[0], &wnaf_entries[0], skew, 0);

    grumpkin::g1::element accumulator;
    accumulator = grumpkin::g1::element(ladder[0].one);
    if (skew) {
        accumulator += gen_data.generator;
    }

    for (size_t i = 0; i < num_quads; ++i) {
        uint64_t entry = wnaf_entries[i + 1];
        const grumpkin::g1::affine_element& point_to_add =
            ((entry & 0xffffff) == 1) ? ladder[i + 1].three : ladder[i + 1].one;
        uint64_t predicate = (entry >> 31U) & 1U;
        accumulator.self_mixed_add_or_sub(point_to_add, predicate);
    }

    return accumulator.normalize();
}

} // namespace pedersen
} // namespace crypto
