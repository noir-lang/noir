#include "block_constraint.hpp"
#include "barretenberg/stdlib/primitives/memory/rom_table.hpp"
#include "barretenberg/stdlib/primitives/memory/ram_table.hpp"

using namespace proof_system::plonk;

namespace acir_format {
field_ct poly_to_field_ct(const poly_triple poly, Composer& composer)
{
    ASSERT(poly.q_m == 0);
    ASSERT(poly.q_r == 0);
    ASSERT(poly.q_o == 0);
    if (poly.q_l == 0) {
        return field_ct(poly.q_c);
    }
    field_ct x = field_ct::from_witness_index(&composer, poly.a);
    x.additive_constant = poly.q_c;
    x.multiplicative_constant = poly.q_l;
    return x;
}

void create_block_constraints(Composer& composer, const BlockConstraint constraint)
{
    std::vector<field_ct> init;
    for (auto i : constraint.init) {
        field_ct value = poly_to_field_ct(i, composer);
        init.push_back(value);
    }

    switch (constraint.type) {
    case BlockType::ROM: {

        rom_table_ct table(init);
        for (auto& op : constraint.trace) {
            ASSERT(op.access_type == 0);
            field_ct value = poly_to_field_ct(op.value, composer);
            field_ct index = poly_to_field_ct(op.index, composer);
            value.assert_equal(table[index]);
        }
    } break;
    case BlockType::RAM: {
        ram_table_ct table(init);
        for (auto& op : constraint.trace) {
            field_ct value = poly_to_field_ct(op.value, composer);
            field_ct index = poly_to_field_ct(op.index, composer);
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

} // namespace acir_format