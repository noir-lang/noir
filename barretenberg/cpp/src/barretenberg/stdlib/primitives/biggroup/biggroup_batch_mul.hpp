#pragma once

#include "barretenberg/stdlib/primitives/biggroup/biggroup_edgecase_handling.hpp"
#include <cstddef>
namespace bb::stdlib {

/**
 * @brief Multiscalar multiplication that utilizes 4-bit wNAF lookup tables.
 * @details This is more efficient than points-as-linear-combinations lookup tables, if the number of points is 3 or
 * fewer. Only works for Plookup (otherwise falls back on batch_mul)!
 * @todo : TODO(https://github.com/AztecProtocol/barretenberg/issues/1001) when we nuke standard and turbo plonk we
 * should remove the fallback batch mul method!
 */
template <typename C, class Fq, class Fr, class G>
template <size_t max_num_bits>
element<C, Fq, Fr, G> element<C, Fq, Fr, G>::wnaf_batch_mul(const std::vector<element>& _points,
                                                            const std::vector<Fr>& _scalars)
{
    constexpr size_t WNAF_SIZE = 4;
    ASSERT(_points.size() == _scalars.size());
    if constexpr (!HasPlookup<C>) {
        return batch_mul(_points, _scalars, max_num_bits);
    }

    const auto [points, scalars] = handle_points_at_infinity(_points, _scalars);

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
        std::vector<element> to_add;
        for (size_t j = 0; j < points.size(); ++j) {
            to_add.emplace_back(point_tables[j][wnaf_entries[j][i]]);
        }
        accumulator = accumulator.quadruple_and_add(to_add);
    }

    for (size_t i = 0; i < points.size(); ++i) {
        element skew = accumulator - points[i];
        Fq out_x = accumulator.x.conditional_select(skew.x, bool_ct(wnaf_entries[i][num_rounds]));
        Fq out_y = accumulator.y.conditional_select(skew.y, bool_ct(wnaf_entries[i][num_rounds]));
        accumulator = element(out_x, out_y);
    }
    accumulator -= offset_generators.second;
    return accumulator;
}
} // namespace bb::stdlib