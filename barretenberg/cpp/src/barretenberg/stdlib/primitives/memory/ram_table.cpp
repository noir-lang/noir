#include "ram_table.hpp"

#include "../circuit_builders/circuit_builders.hpp"

namespace bb::stdlib {

/**
 * @brief Construct a new ram table<Builder>::ram table object. It's dynamic memory!
 *
 * @tparam Builder
 * @param table_entries vector of field elements that will initialize the RAM table
 */
template <typename Builder> ram_table<Builder>::ram_table(Builder* builder, const size_t table_size)
{
    static_assert(HasPlookup<Builder>);
    _context = builder;
    _length = table_size;
    _index_initialized.resize(table_size);
    for (size_t i = 0; i < _index_initialized.size(); ++i) {
        _index_initialized[i] = false;
    }

    // do not initialize the table yet. The input entries might all be constant,
    // if this is the case we might not have a valid pointer to a Builder
    // We get around this, by initializing the table when `read` or `write` operator is called
    // with a non-const field element.
}

/**
 * @brief Construct a new ram table<Builder>::ram table object. It's dynamic memory!
 *
 * @tparam Builder
 * @param table_entries vector of field elements that will initialize the RAM table
 */
template <typename Builder> ram_table<Builder>::ram_table(const std::vector<field_pt>& table_entries)
{
    static_assert(HasPlookup<Builder>);
    // get the builder _context
    for (const auto& entry : table_entries) {
        if (entry.get_context() != nullptr) {
            _context = entry.get_context();
            break;
        }
    }
    _raw_entries = table_entries;
    _length = _raw_entries.size();
    _index_initialized.resize(_length);
    for (size_t i = 0; i < _index_initialized.size(); ++i) {
        _index_initialized[i] = false;
    }
    // do not initialize the table yet. The input entries might all be constant,
    // if this is the case we might not have a valid pointer to a Builder
    // We get around this, by initializing the table when `read` or `write` operator is called
    // with a non-const field element.
}

/**
 * @brief internal method, is used to call Builder methods that will generate RAM table.
 *
 * @details initialize the table once we perform a read. This ensures we always have a pointer to a Builder.
 * (if both the table entries and the index are constant, we don't need a builder as we
 * can directly extract the desired value fram `_raw_entries`)
 *
 * @tparam Builder
 */
template <typename Builder> void ram_table<Builder>::initialize_table() const
{
    if (_ram_table_generated_in_builder) {
        return;
    }
    ASSERT(_context != nullptr);

    _ram_id = _context->create_RAM_array(_length);

    if (_raw_entries.size() > 0) {
        for (size_t i = 0; i < _length; ++i) {
            if (!_index_initialized[i]) {
                field_pt entry;
                if (_raw_entries[i].is_constant()) {
                    entry = field_pt::from_witness_index(_context,
                                                         _context->put_constant_variable(_raw_entries[i].get_value()));
                } else {
                    entry = _raw_entries[i].normalize();
                }
                _context->init_RAM_element(_ram_id, i, entry.get_witness_index());
                _index_initialized[i] = true;
            }
        }
    }

    _ram_table_generated_in_builder = true;
}

/**
 * @brief Construct a new ram table<Builder>::ram table object
 *
 * @tparam Builder
 * @param other
 */
template <typename Builder>
ram_table<Builder>::ram_table(const ram_table& other)
    : _raw_entries(other._raw_entries)
    , _index_initialized(other._index_initialized)
    , _length(other._length)
    , _ram_id(other._ram_id)
    , _ram_table_generated_in_builder(other._ram_table_generated_in_builder)
    , _all_entries_written_to_with_constant_index(other._all_entries_written_to_with_constant_index)
    , _context(other._context)
{}

/**
 * @brief Construct a new ram table<Builder>::ram table object
 *
 * @tparam Builder
 * @param other
 */
template <typename Builder>
ram_table<Builder>::ram_table(ram_table&& other)
    : _raw_entries(other._raw_entries)
    , _index_initialized(other._index_initialized)
    , _length(other._length)
    , _ram_id(other._ram_id)
    , _ram_table_generated_in_builder(other._ram_table_generated_in_builder)
    , _all_entries_written_to_with_constant_index(other._all_entries_written_to_with_constant_index)
    , _context(other._context)
{}

/**
 * @brief Copy assignment operator
 *
 * @tparam Builder
 * @param other
 * @return ram_table<Builder>&
 */
template <typename Builder> ram_table<Builder>& ram_table<Builder>::operator=(const ram_table& other)
{
    _raw_entries = other._raw_entries;
    _length = other._length;
    _ram_id = other._ram_id;
    _index_initialized = other._index_initialized;
    _ram_table_generated_in_builder = other._ram_table_generated_in_builder;
    _all_entries_written_to_with_constant_index = other._all_entries_written_to_with_constant_index;

    _context = other._context;
    return *this;
}

/**
 * @brief Move assignment operator
 *
 * @tparam Builder
 * @param other
 * @return ram_table<Builder>&
 */
template <typename Builder> ram_table<Builder>& ram_table<Builder>::operator=(ram_table&& other)
{
    _raw_entries = other._raw_entries;
    _length = other._length;
    _ram_id = other._ram_id;
    _index_initialized = other._index_initialized;
    _ram_table_generated_in_builder = other._ram_table_generated_in_builder;
    _all_entries_written_to_with_constant_index = other._all_entries_written_to_with_constant_index;
    _context = other._context;
    return *this;
}

/**
 * @brief Read a field element from the RAM table at an index value
 *
 * @tparam Builder
 * @param index
 * @return field_t<Builder>
 */
template <typename Builder> field_t<Builder> ram_table<Builder>::read(const field_pt& index) const
{
    if (_context == nullptr) {
        _context = index.get_context();
    }

    if (uint256_t(index.get_value()) >= _length) {
        // TODO: what's best practise here? We are assuming that this action will generate failing constraints,
        // and we set failure message here so that it better describes the point of failure.
        // However, we are not *ensuring* that failing constraints are generated at the point that `failure()` is
        // called. Is this ok?
        _context->failure("ram_table: RAM array access out of bounds");
    }

    initialize_table();

    if (!check_indices_initialized()) {
        _context->failure("ram_table: must write to every RAM entry at least once (with constant index value) before "
                          "table can be read");
    }

    field_pt index_wire = index;
    if (index.is_constant()) {
        index_wire = field_pt::from_witness_index(_context, _context->put_constant_variable(index.get_value()));
    }

    uint32_t output_idx = _context->read_RAM_array(_ram_id, index_wire.normalize().get_witness_index());
    return field_pt::from_witness_index(_context, output_idx);
}

/**
 * @brief Write a field element from the RAM table at an index value
 *
 * @tparam Builder
 * @param index
 * @param value
 */
template <typename Builder> void ram_table<Builder>::write(const field_pt& index, const field_pt& value)
{
    if (_context == nullptr) {
        _context = index.get_context();
    }

    if (uint256_t(index.get_value()) >= _length) {
        // TODO: what's best practise here? We are assuming that this action will generate failing constraints,
        // and we set failure message here so that it better describes the point of failure.
        // However, we are not *ensuring* that failing constraints are generated at the point that `failure()` is
        // called. Is this ok?
        _context->failure("ram_table: RAM array access out of bounds");
    }

    initialize_table();
    field_pt index_wire = index;
    auto native_index = index.get_value();
    if (index.is_constant()) {
        // need to write every array element at a constant index before doing reads/writes at prover-defined indices
        index_wire = field_pt::from_witness_index(_context, _context->put_constant_variable(native_index));
    } else {
        if (!check_indices_initialized()) {
            _context->failure("ram_table: must write to every RAM entry at least once (with constant index value) "
                              "before table can be written to at an unknown index");
        }
    }

    field_pt value_wire = value;
    auto native_value = value.get_value();
    if (value.is_constant()) {
        value_wire = field_pt::from_witness_index(_context, _context->put_constant_variable(native_value));
    }

    const size_t cast_index = static_cast<size_t>(static_cast<uint64_t>(native_index));
    if (index.is_constant() && _index_initialized[cast_index] == false) {
        _context->init_RAM_element(_ram_id, cast_index, value_wire.get_witness_index());

        _index_initialized[cast_index] = true;
    } else {
        _context->write_RAM_array(_ram_id, index_wire.normalize().get_witness_index(), value_wire.get_witness_index());
    }
}

template class ram_table<bb::UltraCircuitBuilder>;
template class ram_table<bb::GoblinUltraCircuitBuilder>;
} // namespace bb::stdlib