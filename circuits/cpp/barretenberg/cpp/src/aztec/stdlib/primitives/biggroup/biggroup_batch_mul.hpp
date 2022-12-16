#pragma once

namespace plonk {
namespace stdlib {

/**
 * only works for Plookup (otherwise falls back on batch_mul)! Multiscalar multiplication that utilizes 4-bit wNAF
 * lookup tables is more efficient than points-as-linear-combinations lookup tables, if the number of points is 3 or
 * fewer
 */
template <typename C, class Fq, class Fr, class G>
template <size_t max_num_bits>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::wnaf_batch_mul(const std::vector<element>& points,
                                                            const std::vector<Fr>& scalars)
{
    constexpr size_t WNAF_SIZE = 4;
    ASSERT(points.size() == scalars.size());
    if constexpr (C::type != waffle::ComposerType::PLOOKUP) {
        return batch_mul(points, scalars, max_num_bits);
    }

    std::vector<four_bit_table_plookup<>> point_tables;
    for (const auto& point : points) {
        point_tables.emplace_back(four_bit_table_plookup<>(point));
    }

    std::vector<std::vector<field_t<C>>> wnaf_entries;
    for (const auto& scalar : scalars) {
        wnaf_entries.emplace_back(compute_wnaf<max_num_bits>(scalar));
    }

    constexpr size_t num_bits = (max_num_bits == 0) ? (Fr::modulus.get_msb() + 1) : (max_num_bits);
    constexpr size_t num_rounds = ((num_bits + WNAF_SIZE - 1) / WNAF_SIZE);
    const auto offset_generators = compute_offset_generators(num_rounds * 4 - 3);

    element accumulator = offset_generators.first + point_tables[0][wnaf_entries[0][0]];
    for (size_t i = 1; i < points.size(); ++i) {
        accumulator += point_tables[i][wnaf_entries[i][0]];
    }

    for (size_t i = 1; i < num_rounds; ++i) {
        accumulator = accumulator.dbl();
        accumulator = accumulator.dbl();

        element to_add = point_tables[0][wnaf_entries[0][i]];
        for (size_t j = 1; j < points.size(); ++j) {
            to_add += point_tables[j][wnaf_entries[j][i]];
        }
        // accumulator = accumulator.dbl();
        // accumulator = accumulator.montgomery_ladder(to_add);
        accumulator = accumulator.double_into_montgomery_ladder(to_add);
    }

    for (size_t i = 0; i < points.size(); ++i) {
        element skew = accumulator - points[i];
        Fq out_x = accumulator.x.conditional_select(skew.x, bool_t<C>(wnaf_entries[i][num_rounds]));
        Fq out_y = accumulator.y.conditional_select(skew.y, bool_t<C>(wnaf_entries[i][num_rounds]));
        accumulator = element(out_x, out_y);
    }
    accumulator -= offset_generators.second;
    return accumulator;
}
} // namespace stdlib
} // namespace plonk