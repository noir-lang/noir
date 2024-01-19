#pragma once

#include "../types.hpp"
#include "./fixed_base_params.hpp"
#include "barretenberg/crypto/generators/generator_data.hpp"
#include "barretenberg/crypto/pedersen_hash/pedersen.hpp"
#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

namespace bb::plookup::fixed_base {

/**
 * @brief Generates plookup tables required to perform fixed-base scalar multiplication over a fixed number of points.
 *
 */
class table : public FixedBaseParams {
  public:
    using affine_element = grumpkin::g1::affine_element;
    using element = grumpkin::g1::element;
    using single_lookup_table = std::vector<affine_element>;
    using fixed_base_scalar_mul_tables = std::vector<single_lookup_table>;
    using all_multi_tables = std::array<fixed_base_scalar_mul_tables, NUM_FIXED_BASE_MULTI_TABLES>;

    static constexpr affine_element LHS_GENERATOR_POINT =
        crypto::generator_data<curve::Grumpkin>::precomputed_generators[0];

    static constexpr affine_element RHS_GENERATOR_POINT =
        crypto::generator_data<curve::Grumpkin>::precomputed_generators[1];

    static inline single_lookup_table generate_single_lookup_table(const affine_element& base_point,
                                                                   const affine_element& offset_generator);
    template <size_t num_bits> static fixed_base_scalar_mul_tables generate_tables(const affine_element& input);

    template <size_t num_table_bits> static affine_element generate_generator_offset(const affine_element& input);

    static constexpr uint256_t MAX_LO_SCALAR = uint256_t(1) << BITS_PER_LO_SCALAR;
    // We split each scalar mulitplier into BITS_PER_LO_SCALAR, BITS_PER_HI_SCALAR chunks and perform 2 scalar muls of
    // size BITS_PER_LO_SCALAR, BITS_PER_HI_SCALAR (see fixed_base_params.hpp for more details)
    // i.e. we treat 1 scalar mul as two independent scalar muls over (roughly) half-width input scalars.
    // The base_point members describe the fixed-base points that correspond to the two independent scalar muls,
    // for our two supported points
    inline static const affine_element lhs_base_point_lo = LHS_GENERATOR_POINT;
    inline static const affine_element lhs_base_point_hi = element(lhs_base_point_lo) * MAX_LO_SCALAR;
    inline static const affine_element rhs_base_point_lo = RHS_GENERATOR_POINT;
    inline static const affine_element rhs_base_point_hi = element(rhs_base_point_lo) * MAX_LO_SCALAR;

    // fixed_base_tables = lookup tables of precomputed base points required for our lookup arguments.
    // N.B. these "tables" are not plookup tables, just regular ol' software lookup tables.
    // Used to build the proper plookup table and in the `BasicTable::get_values_from_key` method
    static const all_multi_tables fixed_base_tables;

    /**
     * @brief offset generators!
     *
     * We add a unique "offset generator" into each lookup table to ensure that we never trigger
     * incomplete addition formulae for short Weierstrass curves.
     * The offset generators are linearly independent from the fixed-base points we're multiplying, ensuring that a
     * collision is as likely as solving the discrete logarithm problem.
     * For example, imagine a 2-bit lookup table of a point [P]. The table would normally contain {
     * 0.[P], 1.[P], 2.[P], 3.[P]}. But, we dont want to have to handle points at infinity and we also don't want to
     * deal with windowed-non-adjacent-form complexities. Instead, we derive offset generator [G] and make the table
     * equal to { [G] + 0.[P], [G] + 1.[P], [G] + 2.[P], [G] + 3.[P]}. Each table uses a unique offset generator to
     * prevent collisions.
     * The final scalar multiplication output will have a precisely-known contribution from the offset generators,
     * which can then be subtracted off with a single point subtraction.
     **/
    static const std::array<affine_element, table::NUM_FIXED_BASE_MULTI_TABLES> fixed_base_table_offset_generators;

    static bool lookup_table_exists_for_point(const affine_element& input);
    static std::optional<std::array<MultiTableId, 2>> get_lookup_table_ids_for_point(const affine_element& input);
    static std::optional<affine_element> get_generator_offset_for_table_id(MultiTableId table_id);

    template <size_t multitable_index>
    static BasicTable generate_basic_fixed_base_table(BasicTableId id, size_t basic_table_index, size_t table_index);
    template <size_t multitable_index, size_t num_bits> static MultiTable get_fixed_base_table(MultiTableId id);

    template <size_t multitable_index, size_t table_index>
    static std::array<bb::fr, 2> get_basic_fixed_base_table_values(const std::array<uint64_t, 2> key)
    {
        static_assert(multitable_index < NUM_FIXED_BASE_MULTI_TABLES);
        static_assert(table_index < get_num_bits_of_multi_table(multitable_index));
        const auto& basic_table = fixed_base_tables[multitable_index][table_index];
        const auto index = static_cast<size_t>(key[0]);
        return { basic_table[index].x, basic_table[index].y };
    }
};

} // namespace bb::plookup::fixed_base
