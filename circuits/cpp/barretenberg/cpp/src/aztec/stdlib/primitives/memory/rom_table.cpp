#include "rom_table.hpp"

#include "../composers/composers.hpp"

using namespace barretenberg;

namespace plonk {
namespace stdlib {

template <typename Composer> rom_table<Composer>::rom_table(const std::vector<field_pt>& table_entries)
{
    static_assert(Composer::type == waffle::ComposerType::PLOOKUP);
    // get the composer context
    for (const auto& entry : table_entries) {
        if (entry.get_context() != nullptr) {
            context = entry.get_context();
            break;
        }
    }
    raw_entries = table_entries;
    length = raw_entries.size();
    // do not initialize the table yet. The input entries might all be constant,
    // if this is the case we might not have a valid pointer to a Composer
    // We get around this, by initializing the table when `operator[]` is called
    // with a non-const field element.
}

// initialize the table once we perform a read. This ensures we always have a valid
// pointer to a Composer.
// (if both the table entries and the index are constant, we don't need a composer as we
// can directly extract the desired value from `raw_entries`)
template <typename Composer> void rom_table<Composer>::initialize_table() const
{
    if (initialized) {
        return;
    }
    ASSERT(context != nullptr);
    // populate table. Table entries must be normalized and cannot be constants
    for (const auto& entry : raw_entries) {
        if (entry.is_constant()) {
            entries.emplace_back(
                field_pt::from_witness_index(context, context->put_constant_variable(entry.get_value())));
        } else {
            entries.emplace_back(entry.normalize());
        }
    }
    rom_id = context->create_ROM_array(length);

    for (size_t i = 0; i < length; ++i) {
        context->set_ROM_element(rom_id, i, entries[i].get_witness_index());
    }

    initialized = true;
}

template <typename Composer>
rom_table<Composer>::rom_table(const rom_table& other)
    : raw_entries(other.raw_entries)
    , entries(other.entries)
    , length(other.length)
    , rom_id(other.rom_id)
    , initialized(other.initialized)
    , context(other.context)
{}

template <typename Composer>
rom_table<Composer>::rom_table(rom_table&& other)
    : raw_entries(other.raw_entries)
    , entries(other.entries)
    , length(other.length)
    , rom_id(other.rom_id)
    , initialized(other.initialized)
    , context(other.context)
{}

template <typename Composer> rom_table<Composer>& rom_table<Composer>::operator=(const rom_table& other)
{
    raw_entries = other.raw_entries;
    entries = other.entries;
    length = other.length;
    rom_id = other.rom_id;
    initialized = other.initialized;
    context = other.context;
    return *this;
}

template <typename Composer> rom_table<Composer>& rom_table<Composer>::operator=(rom_table&& other)
{
    raw_entries = other.raw_entries;
    entries = other.entries;
    length = other.length;
    rom_id = other.rom_id;
    initialized = other.initialized;
    context = other.context;
    return *this;
}

template <typename Composer> field_t<Composer> rom_table<Composer>::operator[](const size_t index) const
{
    if (index >= length) {
        ASSERT(context != nullptr);
        context->failure("rom_rable: ROM array access out of bounds");
    }

    return entries[index];
}

template <typename Composer> field_t<Composer> rom_table<Composer>::operator[](const field_pt& index) const
{
    if (index.is_constant()) {
        return operator[](static_cast<size_t>(uint256_t(index.get_value()).data[0]));
    }
    if (context == nullptr) {
        context = index.get_context();
    }
    initialize_table();
    if (uint256_t(index.get_value()) >= length) {
        context->failure("rom_table: ROM array access out of bounds");
    }

    uint32_t output_idx = context->read_ROM_array(rom_id, index.normalize().get_witness_index());
    return field_pt::from_witness_index(context, output_idx);
}

INSTANTIATE_STDLIB_ULTRA_TYPE(rom_table);
} // namespace stdlib
} // namespace plonk