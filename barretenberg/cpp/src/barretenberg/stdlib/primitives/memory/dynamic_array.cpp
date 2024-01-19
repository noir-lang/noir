#include "dynamic_array.hpp"

#include "../bool/bool.hpp"
#include "../circuit_builders/circuit_builders.hpp"

namespace bb::stdlib {

/**
 * @brief Construct a new Dynamic Array< Builder>:: Dynamic Array object
 *
 * @details Dynamic arrays require a maximum size when created, that cannot be exceeded.
 *          Read and write operations cost 3.25 UltraPlonk gates.
 *          Each dynamic array requires an additional 3.25 * maximum_size number of gates.
 *          If the dynamic array also requires a unique range constraint table due to its length (e.g. not a power of
 * 2), this will add an additional (maximum_size / 6) gates.
 *
 * @tparam Builder
 * @param builder
 * @param maximum_size The maximum size of the array
 */
template <typename Builder>
DynamicArray<Builder>::DynamicArray(Builder* builder, const size_t maximum_size)
    : _context(builder)
    , _max_size(maximum_size)
    , _length(0)
{
    static_assert(HasPlookup<Builder>);
    ASSERT(_context != nullptr);
    _inner_table = ram_table(_context, maximum_size);
    // Initialize the ram table with all zeroes
    for (size_t i = 0; i < maximum_size; ++i) {
        _inner_table.write(i, 0);
    }
}

/**
 * @brief Construct a new Dynamic Array< Builder>:: Dynamic Array object
 *
 * @tparam Builder
 * @param other
 */
template <typename Builder>
DynamicArray<Builder>::DynamicArray(const DynamicArray& other)
    : _context(other._context)
    , _max_size(other._max_size)
    , _length(other._length)
    , _inner_table(other._inner_table)
{}

/**
 * @brief Construct a new Dynamic Array< Builder>:: Dynamic Array object
 *
 * @tparam Builder
 * @param other
 */
template <typename Builder>
DynamicArray<Builder>::DynamicArray(DynamicArray&& other)
    : _context(other._context)
    , _max_size(other._max_size)
    , _length(other._length)
    , _inner_table(other._inner_table)
{}

/**
 * @brief Assignment Operator
 *
 * @tparam Builder
 * @param other
 * @return DynamicArray<Builder>&
 */
template <typename Builder> DynamicArray<Builder>& DynamicArray<Builder>::operator=(const DynamicArray& other)
{
    _context = other._context;
    _max_size = other._max_size;
    _length = other._length;
    _inner_table = other._inner_table;
    return *this;
}

/**
 * @brief Move Assignment Operator
 *
 * @tparam Builder
 * @param other
 * @return DynamicArray<Builder>&
 */
template <typename Builder> DynamicArray<Builder>& DynamicArray<Builder>::operator=(DynamicArray&& other)
{
    _context = other._context;
    _max_size = other._max_size;
    _length = other._length;
    _inner_table = other._inner_table;
    return *this;
}

/**
 * @brief Resize array. Current method v. inefficient!
 *
 * @tparam Builder
 * @param new_length
 */
template <typename Builder> void DynamicArray<Builder>::resize(const field_pt& new_length, const field_pt default_value)
{
    // 1: assert new_length < max_size
    field_pt max_bounds_check = (field_pt(_max_size) - new_length - 1);
    if (max_bounds_check.is_constant()) {
        ASSERT(uint256_t(new_length.get_value()) <= _max_size);
    } else {
        _context->create_new_range_constraint(max_bounds_check.normalize().get_witness_index(), _max_size);
    }

    /**
     * Iterate over max array size
     * if i is currently >= length but will be < new_length, write `default_value` into ram table
     */
    for (size_t i = 0; i < _max_size; ++i) {
        bool_pt index_valid = bool_pt(witness_pt(_context, (uint256_t)(new_length.get_value()) > i));
        {
            // index_delta will be between 0 and length - 1 if index valid
            // i.e. will pass check that index_delta < _max_size
            field_pt index_delta = (new_length - i - 1);

            // reverse_delta will be between 0 and (_max_size - length) if *invalid*
            // i.e. will pass check that reverse_delta < _max_size
            field_pt reverse_delta = (-new_length + i);

            field_pt bounds_check = field_pt::conditional_assign(index_valid, index_delta, reverse_delta);

            // this should do the same for only 2 gates, but hard to read
            // field_pt t1 = new_length - i;
            // field_pt t2 = field_pt(index_valid);
            // field_pt bounds_check = (t2 + t2).madd(t1 - 1, -t1);

            _context->create_new_range_constraint(bounds_check.normalize().get_witness_index(), _max_size);
        }

        bool_pt index_currently_invalid = bool_pt(witness_pt(_context, i >= native_size()));
        {
            // index_delta will be between 0 and length - 1 if index valid
            // i.e. will pass check that index_delta < _max_size
            field_pt index_delta = (_length - i - 1);

            // reverse_delta will be between 0 and (_max_size - length) if *invalid*
            // i.e. will pass check that reverse_delta < _max_size
            field_pt reverse_delta = (-_length + i);

            field_pt bounds_check = field_pt::conditional_assign(index_currently_invalid, reverse_delta, index_delta);

            _context->create_new_range_constraint(bounds_check.normalize().get_witness_index(), _max_size);
        }

        field_pt old_value = _inner_table.read(i);
        field_pt new_value =
            field_pt::conditional_assign(index_currently_invalid && index_valid, default_value, old_value);
        _inner_table.write(i, new_value);
    }

    _length = new_length;
}

/**
 * @brief Read a field element from the dynamic array at an index value
 *
 * @tparam Builder
 * @param index
 * @return field_t<Builder>
 */
template <typename Builder> field_t<Builder> DynamicArray<Builder>::read(const field_pt& index) const
{
    const field_pt index_delta = (_length - index - 1);

    if (index_delta.is_constant()) {
        bool valid = (uint256_t(index_delta.get_value()) < _max_size);
        if (!valid) {
            _context->failure("DynamicArray::read access out of bounds");
        }
    } else {
        _context->create_new_range_constraint(
            index_delta.normalize().get_witness_index(), _max_size, "DynamicArray::read access out of bounds");
    }

    return _inner_table.read(index);
}

/**
 * @brief Write a field element into the dynamic array at an index value
 *
 * @tparam Builder
 * @param index
 * @param value
 */
template <typename Builder> void DynamicArray<Builder>::write(const field_pt& index, const field_pt& value)
{
    const field_pt index_delta = (_length - index - 1);

    if (index_delta.is_constant()) {
        bool valid = (uint256_t(index_delta.get_value()) < _max_size);
        if (!valid) {
            _context->failure("DynamicArray::read access out of bounds");
        }
    } else {
        _context->create_new_range_constraint(
            index_delta.normalize().get_witness_index(), _max_size, "DynamicArray::read access out of bounds");
    }

    _inner_table.write(index, value);
}

/**
 * @brief Push a field element onto the dynamic array
 *
 * @tparam Builder
 * @param value
 */
template <typename Builder> void DynamicArray<Builder>::push(const field_pt& value)
{
    if (native_size() >= _max_size) {
        _context->failure("DynamicArray::push array is already at its maximum size");
    }

    _inner_table.write(_length, value);
    _length += 1;
}

/**
 * @brief Pop a field element off of the dynamic array
 *
 * @tparam Builder
 */
template <typename Builder> void DynamicArray<Builder>::pop()
{
    if (native_size() == 0) {
        _context->failure("DynamicArray::pop array is already empty");
    }

    _length.assert_is_not_zero();
    _length -= 1;
}

/**
 * @brief Conditionally push a field element onto the dynamic array
 *
 * @tparam Builder
 * @param predicate
 * @param value
 */
template <typename Builder>
void DynamicArray<Builder>::conditional_push(const bool_pt& predicate, const field_pt& value)
{
    if (native_size() >= _max_size) {
        _context->failure("DynamicArray::push array is already at its maximum size");
    }

    _inner_table.write(_length, value);
    _length += predicate;
}

/**
 * @brief Conditionallhy pop a field element off of the dynamic array
 *
 * @tparam Builder
 * @param predicate
 */
template <typename Builder> void DynamicArray<Builder>::conditional_pop(const bool_pt& predicate)
{
    if (native_size() == 0) {
        _context->failure("DynamicArray::pop array is already empty");
    }

    field_pt length_check = field_pt::conditional_assign(predicate, _length, 1);
    length_check.assert_is_not_zero();
    _length -= predicate;
}

template class DynamicArray<bb::UltraCircuitBuilder>;
template class DynamicArray<bb::GoblinUltraCircuitBuilder>;
} // namespace bb::stdlib