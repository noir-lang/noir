#pragma once

#include <cstddef>
#include <iterator>

#include "ref_array.hpp"
#include "ref_vector.hpp"

namespace bb {

template <typename T> class RefSpan {
  public:
    // Default constructor
    RefSpan()
        : storage(nullptr)
        , array_size(0)
    {}

    template <std::size_t Size>
    RefSpan(const RefArray<T, Size>& ref_array)
        : storage(ref_array.get_storage())
        , array_size(Size)
    {}
    RefSpan(const RefVector<T>& ref_vector)
        : storage(&ref_vector.get_storage()[0])
        , array_size(ref_vector.size())
    {}

    // Constructor from an array of pointers and size
    RefSpan(T** ptr_array, std::size_t size)
        : storage(ptr_array)
        , array_size(size)
    {}

    // Copy constructor
    RefSpan(const RefSpan& other) = default;

    // Move constructor
    RefSpan(RefSpan&& other) noexcept = default;

    // Destructor
    ~RefSpan() = default;

    // Copy assignment operator
    RefSpan& operator=(const RefSpan& other) = default;

    // Move assignment operator
    RefSpan& operator=(RefSpan&& other) noexcept = default;

    // Access element at index
    T& operator[](std::size_t idx) const
    {
        // Assuming the caller ensures idx is within bounds.
        return *storage[idx];
    }

    // Get size of the RefSpan
    constexpr std::size_t size() const { return array_size; }

    // Iterator implementation
    class iterator {
      public:
        iterator(T* const* array, std::size_t pos)
            : array(array)
            , pos(pos)
        {}

        T& operator*() const { return *(array[pos]); }

        iterator& operator++()
        {
            ++pos;
            return *this;
        }

        iterator operator++(int)
        {
            iterator temp = *this;
            ++(*this);
            return temp;
        }

        bool operator==(const iterator& other) const { return pos == other.pos; }
        bool operator!=(const iterator& other) const { return pos != other.pos; }

      private:
        T* const* array;
        std::size_t pos;
    };

    // Begin and end for iterator support
    iterator begin() const { return iterator(storage, 0); }
    iterator end() const { return iterator(storage, array_size); }

  private:
    T* const* storage;
    std::size_t array_size;
};

} // namespace bb
