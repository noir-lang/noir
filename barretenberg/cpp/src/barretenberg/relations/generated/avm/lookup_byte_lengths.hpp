#pragma once

#include "barretenberg/relations/generic_lookup/generic_lookup_relation.hpp"

#include <cstddef>
#include <tuple>

namespace bb {

class lookup_byte_lengths_lookup_settings {
  public:
    static constexpr size_t READ_TERMS = 1;
    static constexpr size_t WRITE_TERMS = 1;
    static constexpr size_t READ_TERM_TYPES[READ_TERMS] = { 0 };
    static constexpr size_t WRITE_TERM_TYPES[WRITE_TERMS] = { 0 };
    static constexpr size_t LOOKUP_TUPLE_SIZE = 2;
    static constexpr size_t INVERSE_EXISTS_POLYNOMIAL_DEGREE = 4;
    static constexpr size_t READ_TERM_DEGREE = 0;
    static constexpr size_t WRITE_TERM_DEGREE = 0;

    template <typename AllEntities> static inline auto inverse_polynomial_is_computed_at_row(const AllEntities& in)
    {
        return (in.binary_start == 1 || in.byte_lookup_sel_bin == 1);
    }

    template <typename Accumulator, typename AllEntities>
    static inline auto compute_inverse_exists(const AllEntities& in)
    {
        using View = typename Accumulator::View;
        const auto is_operation = View(in.binary_start);
        const auto is_table_entry = View(in.byte_lookup_sel_bin);
        return (is_operation + is_table_entry - is_operation * is_table_entry);
    }

    template <typename AllEntities> static inline auto get_const_entities(const AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_byte_lengths,
                                     in.lookup_byte_lengths_counts,
                                     in.binary_start,
                                     in.byte_lookup_sel_bin,
                                     in.binary_in_tag,
                                     in.binary_mem_tag_ctr,
                                     in.byte_lookup_table_in_tags,
                                     in.byte_lookup_table_byte_lengths);
    }

    template <typename AllEntities> static inline auto get_nonconst_entities(AllEntities& in)
    {
        return std::forward_as_tuple(in.lookup_byte_lengths,
                                     in.lookup_byte_lengths_counts,
                                     in.binary_start,
                                     in.byte_lookup_sel_bin,
                                     in.binary_in_tag,
                                     in.binary_mem_tag_ctr,
                                     in.byte_lookup_table_in_tags,
                                     in.byte_lookup_table_byte_lengths);
    }
};

template <typename FF_>
class lookup_byte_lengths_relation : public GenericLookupRelation<lookup_byte_lengths_lookup_settings, FF_> {
  public:
    static constexpr const char* NAME = "LOOKUP_BYTE_LENGTHS";
};
template <typename FF_> using lookup_byte_lengths = GenericLookup<lookup_byte_lengths_lookup_settings, FF_>;

} // namespace bb