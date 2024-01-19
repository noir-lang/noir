#include "block_constraint.hpp"
#include "barretenberg/stdlib/primitives/memory/ram_table.hpp"
#include "barretenberg/stdlib/primitives/memory/rom_table.hpp"

using namespace bb::plonk;

namespace acir_format {

template <typename Builder> bb::stdlib::field_t<Builder> poly_to_field_ct(const poly_triple poly, Builder& builder)
{
    using field_ct = bb::stdlib::field_t<Builder>;

    ASSERT(poly.q_m == 0);
    ASSERT(poly.q_r == 0);
    ASSERT(poly.q_o == 0);
    if (poly.q_l == 0) {
        return field_ct(poly.q_c);
    }
    field_ct x = field_ct::from_witness_index(&builder, poly.a);
    x.additive_constant = poly.q_c;
    x.multiplicative_constant = poly.q_l;
    return x;
}

template <typename Builder>
void create_block_constraints(Builder& builder, const BlockConstraint constraint, bool has_valid_witness_assignments)
{
    using field_ct = bb::stdlib::field_t<Builder>;
    using rom_table_ct = bb::stdlib::rom_table<Builder>;
    using ram_table_ct = bb::stdlib::ram_table<Builder>;

    std::vector<field_ct> init;
    for (auto i : constraint.init) {
        field_ct value = poly_to_field_ct(i, builder);
        init.push_back(value);
    }

    switch (constraint.type) {
    case BlockType::ROM: {
        rom_table_ct table(init);
        for (auto& op : constraint.trace) {
            ASSERT(op.access_type == 0);
            field_ct value = poly_to_field_ct(op.value, builder);
            field_ct index = poly_to_field_ct(op.index, builder);
            // For a ROM table, constant read should be optimized out:
            // The rom_table won't work with a constant read because the table may not be initialized
            ASSERT(op.index.q_l != 0);
            // We create a new witness w to avoid issues with non-valid witness assignements:
            // if witness are not assigned, then w will be zero and table[w] will work
            fr w_value = 0;
            if (has_valid_witness_assignments) {
                // If witness are assigned, we use the correct value for w
                w_value = index.get_value();
            }
            field_ct w = field_ct::from_witness(&builder, w_value);
            value.assert_equal(table[w]);
            w.assert_equal(index);
        }
    } break;
    case BlockType::RAM: {
        ram_table_ct table(init);
        for (auto& op : constraint.trace) {
            field_ct value = poly_to_field_ct(op.value, builder);
            field_ct index = poly_to_field_ct(op.index, builder);

            // We create a new witness w to avoid issues with non-valid witness assignements.
            // If witness are not assigned, then index will be zero and table[index] won't hit bounds check.
            fr index_value = has_valid_witness_assignments ? index.get_value() : 0;
            // Create new witness and ensure equal to index.
            field_ct::from_witness(&builder, index_value).assert_equal(index);

            if (op.access_type == 0) {
                value.assert_equal(table.read(index));
            } else {
                ASSERT(op.access_type == 1);
                table.write(index, value);
            }
        }
    } break;
    default:
        ASSERT(false);
        break;
    }
}

template void create_block_constraints<UltraCircuitBuilder>(UltraCircuitBuilder& builder,
                                                            const BlockConstraint constraint,
                                                            bool has_valid_witness_assignments);
template void create_block_constraints<GoblinUltraCircuitBuilder>(GoblinUltraCircuitBuilder& builder,
                                                                  const BlockConstraint constraint,
                                                                  bool has_valid_witness_assignments);

} // namespace acir_format