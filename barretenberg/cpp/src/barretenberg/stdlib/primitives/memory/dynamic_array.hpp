#pragma once
#include "../circuit_builders/circuit_builders_fwd.hpp"
#include "ram_table.hpp"
namespace bb::stdlib {

/**
 * @brief A dynamic array of field elements
 *
 * @tparam Builder (must support plookup)
 */
template <typename Builder> class DynamicArray {
  private:
    typedef field_t<Builder> field_pt;
    typedef bool_t<Builder> bool_pt;
    typedef witness_t<Builder> witness_pt;

  public:
    DynamicArray(Builder* builder, const size_t maximum_size);

    DynamicArray(const DynamicArray& other);
    DynamicArray(DynamicArray&& other);

    DynamicArray& operator=(const DynamicArray& other);
    DynamicArray& operator=(DynamicArray&& other);

    void resize(const field_pt& new_length, const field_pt default_value = 0);

    field_pt read(const field_pt& index) const;
    void write(const field_pt& index, const field_pt& value);

    void push(const field_pt& index);
    void pop();

    void conditional_push(const bool_pt& predicate, const field_pt& index);
    void conditional_pop(const bool_pt& predicate);

    field_pt size() const { return _length; }
    size_t native_size() const { return static_cast<size_t>(static_cast<uint256_t>(_length.get_value())); }
    size_t max_size() const { return _max_size; }

    Builder* get_context() const { return _context; }

  private:
    Builder* _context = nullptr;
    size_t _max_size;
    field_pt _length = 0;
    mutable ram_table<Builder> _inner_table;
};
} // namespace bb::stdlib