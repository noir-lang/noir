#include "block_constraint.hpp"
#include "barretenberg/stdlib/primitives/databus/databus.hpp"
#include "barretenberg/stdlib/primitives/memory/ram_table.hpp"
#include "barretenberg/stdlib/primitives/memory/rom_table.hpp"

namespace acir_format {

using namespace bb::plonk;
using namespace bb;

template <typename Builder> stdlib::field_t<Builder> poly_to_field_ct(const poly_triple poly, Builder& builder)
{
    using field_ct = stdlib::field_t<Builder>;

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

/**
 * @brief Create block constraints; Specialization for Ultra arithmetization
 * @details Ultra does not support DataBus operations so calldata/returndata are treated as ROM ops
 *
 */
template <>
void create_block_constraints(UltraCircuitBuilder& builder,
                              const BlockConstraint& constraint,
                              bool has_valid_witness_assignments)
{
    using field_ct = bb::stdlib::field_t<UltraCircuitBuilder>;

    std::vector<field_ct> init;
    for (auto i : constraint.init) {
        field_ct value = poly_to_field_ct(i, builder);
        init.push_back(value);
    }

    switch (constraint.type) {
    // Note: CallData/ReturnData not supported by Ultra; interpreted as ROM ops instead
    case BlockType::CallData:
    case BlockType::ReturnData:
    case BlockType::ROM: {
        process_ROM_operations(builder, constraint, has_valid_witness_assignments, init);
    } break;
    case BlockType::RAM: {
        process_RAM_operations(builder, constraint, has_valid_witness_assignments, init);
    } break;
    default:
        ASSERT(false);
        break;
    }
}

/**
 * @brief Create block constraints; Specialization for Mega arithmetization
 *
 */
template <>
void create_block_constraints(MegaCircuitBuilder& builder,
                              const BlockConstraint& constraint,
                              bool has_valid_witness_assignments)
{
    using field_ct = stdlib::field_t<MegaCircuitBuilder>;

    std::vector<field_ct> init;
    for (auto i : constraint.init) {
        field_ct value = poly_to_field_ct(i, builder);
        init.push_back(value);
    }

    switch (constraint.type) {
    case BlockType::ROM: {
        process_ROM_operations(builder, constraint, has_valid_witness_assignments, init);
    } break;
    case BlockType::RAM: {
        process_RAM_operations(builder, constraint, has_valid_witness_assignments, init);
    } break;
    case BlockType::CallData: {
        process_call_data_operations(builder, constraint, has_valid_witness_assignments, init);
    } break;
    case BlockType::ReturnData: {
        process_return_data_operations(constraint, init);
    } break;
    default:
        ASSERT(false);
        break;
    }
}

template <typename Builder>
void process_ROM_operations(Builder& builder,
                            const BlockConstraint& constraint,
                            bool has_valid_witness_assignments,
                            std::vector<bb::stdlib::field_t<Builder>>& init)
{
    using field_ct = stdlib::field_t<Builder>;
    using rom_table_ct = stdlib::rom_table<Builder>;

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
}

template <typename Builder>
void process_RAM_operations(Builder& builder,
                            const BlockConstraint& constraint,
                            bool has_valid_witness_assignments,
                            std::vector<bb::stdlib::field_t<Builder>>& init)
{
    using field_ct = stdlib::field_t<Builder>;
    using ram_table_ct = stdlib::ram_table<Builder>;

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
}

template <typename Builder>
void process_call_data_operations(Builder& builder,
                                  const BlockConstraint& constraint,
                                  bool has_valid_witness_assignments,
                                  std::vector<bb::stdlib::field_t<Builder>>& init)
{
    using field_ct = stdlib::field_t<Builder>;
    using databus_ct = stdlib::databus<Builder>;

    databus_ct databus;
    // Populate the calldata in the databus
    databus.calldata.set_values(init);
    for (const auto& op : constraint.trace) {
        ASSERT(op.access_type == 0);
        field_ct value = poly_to_field_ct(op.value, builder);
        field_ct index = poly_to_field_ct(op.index, builder);
        fr w_value = 0;
        if (has_valid_witness_assignments) {
            // If witness are assigned, we use the correct value for w
            w_value = index.get_value();
        }
        field_ct w = field_ct::from_witness(&builder, w_value);
        value.assert_equal(databus.calldata[w]);
        w.assert_equal(index);
    }
}

template <typename Builder>
void process_return_data_operations(const BlockConstraint& constraint, std::vector<bb::stdlib::field_t<Builder>>& init)
{
    using databus_ct = stdlib::databus<Builder>;

    databus_ct databus;
    // Populate the returndata in the databus
    databus.return_data.set_values(init);
    // For each entry of the return data, explicitly assert equality with the initialization value. This implicitly
    // creates the return data read gates that are required to connect witness values in the main wires to witness
    // values in the databus return data column.
    size_t c = 0;
    for (const auto& value : init) {
        value.assert_equal(databus.return_data[c]);
        c++;
    }
    ASSERT(constraint.trace.size() == 0);
}

} // namespace acir_format