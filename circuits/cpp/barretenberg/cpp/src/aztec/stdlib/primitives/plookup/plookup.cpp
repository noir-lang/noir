#include "./plookup.hpp"
#include <plonk/composer/ultra_composer.hpp>
#include <plonk/composer/plookup_tables/plookup_tables.hpp>
#include <plonk/composer/plookup_tables/types.hpp>

namespace waffle {
class UltraComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {

using namespace barretenberg;

template <typename Composer>
plookup::ReadData<field_t<Composer>> plookup_<Composer>::get_lookup_accumulators(const MultiTableId id,
                                                                                 const field_t<Composer>& key_a_in,
                                                                                 const field_t<Composer>& key_b_in,
                                                                                 const bool is_2_to_1_lookup)
{
    auto key_a = key_a_in.normalize();
    auto key_b = key_b_in.normalize();
    Composer* ctx = key_a.get_context() ? key_a.get_context() : key_b.get_context();
    const plookup::ReadData<barretenberg::fr> lookup_data =
        plookup::get_lookup_accumulators(id, key_a.get_value(), key_b.get_value(), is_2_to_1_lookup);

    const bool is_key_a_constant = key_a.is_constant();
    plookup::ReadData<field_t<Composer>> lookup;
    if (is_key_a_constant && (key_b.is_constant() || !is_2_to_1_lookup)) {
        for (size_t i = 0; i < lookup_data[ColumnIdx::C1].size(); ++i) {
            lookup[ColumnIdx::C1].emplace_back(field_t<Composer>(ctx, lookup_data[ColumnIdx::C1][i]));
            lookup[ColumnIdx::C2].emplace_back(field_t<Composer>(ctx, lookup_data[ColumnIdx::C2][i]));
            lookup[ColumnIdx::C3].emplace_back(field_t<Composer>(ctx, lookup_data[ColumnIdx::C3][i]));
        }
    } else {
        uint32_t lhs_index = key_a.witness_index;
        uint32_t rhs_index = key_b.witness_index;
        // If only one lookup key is constant, we need to instantiate it as a real witness
        if (is_key_a_constant) {
            lhs_index = ctx->put_constant_variable(key_a.get_value());
        }
        if (key_b.is_constant() && is_2_to_1_lookup) {
            rhs_index = ctx->put_constant_variable(key_b.get_value());
        }

        auto key_b_witness = std::make_optional(rhs_index);
        if (rhs_index == IS_CONSTANT) {
            key_b_witness = std::nullopt;
        }
        const auto accumulator_witnesses =
            ctx->create_gates_from_plookup_accumulators(id, lookup_data, lhs_index, key_b_witness);

        for (size_t i = 0; i < lookup_data[ColumnIdx::C1].size(); ++i) {
            lookup[ColumnIdx::C1].emplace_back(
                field_t<Composer>::from_witness_index(ctx, accumulator_witnesses[ColumnIdx::C1][i]));
            lookup[ColumnIdx::C2].emplace_back(
                field_t<Composer>::from_witness_index(ctx, accumulator_witnesses[ColumnIdx::C2][i]));
            lookup[ColumnIdx::C3].emplace_back(
                field_t<Composer>::from_witness_index(ctx, accumulator_witnesses[ColumnIdx::C3][i]));
        }
    }
    return lookup;
}

template <typename Composer>
std::pair<field_t<Composer>, field_t<Composer>> plookup_<Composer>::read_pair_from_table(const MultiTableId id,
                                                                                         const field_t<Composer>& key)
{
    const auto lookup = get_lookup_accumulators(id, key);

    return { lookup[ColumnIdx::C2][0], lookup[ColumnIdx::C3][0] };
}

template <typename Composer>
field_t<Composer> plookup_<Composer>::read_from_2_to_1_table(const MultiTableId id,
                                                             const field_t<Composer>& key_a,
                                                             const field_t<Composer>& key_b)
{
    const auto lookup = get_lookup_accumulators(id, key_a, key_b, true);

    return lookup[ColumnIdx::C2][0];
}

template <typename Composer>
field_t<Composer> plookup_<Composer>::read_from_1_to_2_table(const MultiTableId id, const field_t<Composer>& key_a)
{
    const auto lookup = get_lookup_accumulators(id, key_a);

    return lookup[ColumnIdx::C2][0];
}

template class plookup_<waffle::UltraComposer>;
} // namespace stdlib
} // namespace plonk
