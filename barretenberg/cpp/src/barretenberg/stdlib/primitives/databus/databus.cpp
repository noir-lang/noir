#include "databus.hpp"
#include "../circuit_builders/circuit_builders.hpp"

namespace bb::stdlib {

template <typename Builder> void databus<Builder>::bus_vector::set_values(const std::vector<field_pt>& entries_in)
{
    // Set the context from the input entries
    for (const auto& entry : entries_in) {
        if (entry.get_context() != nullptr) {
            context = entry.get_context();
            break;
        }
    }
    // Enforce that builder context is known at this stage. Otherwise first read will fail if the index is a constant.
    ASSERT(context != nullptr);

    // Initialize the bus vector entries from the input entries which are un-normalized and possibly constants
    for (const auto& entry : entries_in) {
        if (entry.is_constant()) { // create a constant witness from the constant
            auto const_var_idx = context->put_constant_variable(entry.get_value());
            entries.emplace_back(field_pt::from_witness_index(context, const_var_idx));
        } else { // normalize the raw entry
            entries.emplace_back(entry.normalize());
        }
        // Add the entry to the bus vector data
        context->append_to_bus_vector(bus_idx, entries.back().get_witness_index());
    }
    length = entries.size();
}

template <typename Builder> field_t<Builder> databus<Builder>::bus_vector::operator[](const field_pt& index) const
{
    // Ensure the read is valid
    auto raw_index = static_cast<size_t>(uint256_t(index.get_value()).data[0]);
    if (raw_index >= length) {
        context->failure("bus_vector: access out of bounds");
    }

    // The read index must be a witness; if constant, add it as a constant variable
    uint32_t index_witness_idx = 0;
    if (index.is_constant()) {
        index_witness_idx = context->put_constant_variable(index.get_value());
    } else {
        index_witness_idx = index.normalize().get_witness_index();
    }

    // Read from the bus vector at the specified index. Creates a single read gate
    uint32_t output_idx = context->read_bus_vector(bus_idx, index_witness_idx);
    return field_pt::from_witness_index(context, output_idx);
}

template class databus<bb::GoblinUltraCircuitBuilder>;
} // namespace bb::stdlib