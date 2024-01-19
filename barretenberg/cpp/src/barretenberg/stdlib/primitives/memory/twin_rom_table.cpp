#include "twin_rom_table.hpp"

#include "../circuit_builders/circuit_builders.hpp"

using namespace bb;

namespace bb::stdlib {

template <typename Builder>
twin_rom_table<Builder>::twin_rom_table(const std::vector<std::array<field_pt, 2>>& table_entries)
{
    static_assert(HasPlookup<Builder>);
    // get the builder context
    for (const auto& entry : table_entries) {
        if (entry[0].get_context() != nullptr) {
            context = entry[0].get_context();
            break;
        }
        if (entry[1].get_context() != nullptr) {
            context = entry[1].get_context();
            break;
        }
    }
    raw_entries = table_entries;
    length = raw_entries.size();
    // do not initialize the table yet. The input entries might all be constant,
    // if this is the case we might not have a valid pointer to a Builder
    // We get around this, by initializing the table when `operator[]` is called
    // with a non-const field element.
}

// initialize the table once we perform a read. This ensures we always have a valid
// pointer to a Builder.
// (if both the table entries and the index are constant, we don't need a builder as we
// can directly extract the desired value from `raw_entries`)
template <typename Builder> void twin_rom_table<Builder>::initialize_table() const
{
    if (initialized) {
        return;
    }
    ASSERT(context != nullptr);
    // populate table. Table entries must be normalized and cannot be constants
    for (const auto& entry : raw_entries) {
        field_pt first;
        field_pt second;
        if (entry[0].is_constant()) {
            first = field_pt::from_witness_index(context, context->put_constant_variable(entry[0].get_value()));
        } else {
            first = entry[0].normalize();
        }
        if (entry[1].is_constant()) {
            second = field_pt::from_witness_index(context, context->put_constant_variable(entry[1].get_value()));
        } else {
            second = entry[1].normalize();
        }
        entries.emplace_back(field_pair_pt{ first, second });
    }
    rom_id = context->create_ROM_array(length);

    for (size_t i = 0; i < length; ++i) {
        context->set_ROM_element_pair(
            rom_id, i, std::array<uint32_t, 2>{ entries[i][0].get_witness_index(), entries[i][1].get_witness_index() });
    }
    initialized = true;
}

template <typename Builder>
twin_rom_table<Builder>::twin_rom_table(const twin_rom_table& other)
    : raw_entries(other.raw_entries)
    , entries(other.entries)
    , length(other.length)
    , rom_id(other.rom_id)
    , initialized(other.initialized)
    , context(other.context)
{}

template <typename Builder>
twin_rom_table<Builder>::twin_rom_table(twin_rom_table&& other)
    : raw_entries(other.raw_entries)
    , entries(other.entries)
    , length(other.length)
    , rom_id(other.rom_id)
    , initialized(other.initialized)
    , context(other.context)
{}

template <typename Builder> twin_rom_table<Builder>& twin_rom_table<Builder>::operator=(const twin_rom_table& other)
{
    raw_entries = other.raw_entries;
    entries = other.entries;
    length = other.length;
    rom_id = other.rom_id;
    initialized = other.initialized;
    context = other.context;
    return *this;
}

template <typename Builder> twin_rom_table<Builder>& twin_rom_table<Builder>::operator=(twin_rom_table&& other)
{
    raw_entries = other.raw_entries;
    entries = other.entries;
    length = other.length;
    rom_id = other.rom_id;
    initialized = other.initialized;
    context = other.context;
    return *this;
}

template <typename Builder>
std::array<field_t<Builder>, 2> twin_rom_table<Builder>::operator[](const size_t index) const
{
    if (index >= length) {
        ASSERT(context != nullptr);
        context->failure("twin_rom_table: ROM array access out of bounds");
    }

    return entries[index];
}

template <typename Builder>
std::array<field_t<Builder>, 2> twin_rom_table<Builder>::operator[](const field_pt& index) const
{
    if (index.is_constant()) {
        return operator[](static_cast<size_t>(uint256_t(index.get_value()).data[0]));
    }
    if (context == nullptr) {
        context = index.get_context();
    }
    initialize_table();
    if (uint256_t(index.get_value()) >= length) {
        context->failure("twin_rom_table: ROM array access out of bounds");
    }

    auto output_indices = context->read_ROM_array_pair(rom_id, index.normalize().get_witness_index());
    return field_pair_pt{
        field_pt::from_witness_index(context, output_indices[0]),
        field_pt::from_witness_index(context, output_indices[1]),
    };
}

template class twin_rom_table<bb::UltraCircuitBuilder>;
template class twin_rom_table<bb::GoblinUltraCircuitBuilder>;
} // namespace bb::stdlib