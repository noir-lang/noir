#pragma once

#include "barretenberg/common/assert.hpp"
#include <array>
#include <cstddef>
#include <initializer_list>
#include <iterator>
#include <span>
#include <stdexcept>

namespace bb {
/**
 * @brief A template class for a reference array. Behaves as if std::array<T&, N> was possible.
 *
 * This class provides a fixed-size array of pointers to elements of type T, exposed as references.
 * It offers random access to its elements and provides an iterator class
 * for traversal.
 *
 * @tparam T The type of elements stored in the array.
 * @tparam N The size of the array.
 */
template <typename T, std::size_t N> class RefArray {
  public:
    RefArray() = default;
    RefArray(const std::array<T*, N>& ptr_array)
    {
        for (std::size_t i = 0; i < N; ++i) {
            storage[i] = ptr_array[i];
        }
    }
    template <typename... Ts> RefArray(T& ref, Ts&... rest)
    {
        storage[0] = &ref;
        int i = 1;
        ((storage[i++] = &rest), ...);
    }

    T& operator[](std::size_t idx) const
    {
        // GCC has a bug where it has trouble analyzing zip_view
        // this is likely due to this bug https://gcc.gnu.org/bugzilla/show_bug.cgi?id=104165
        // We disable this - if GCC was right, we would have caught this at runtime
#if !defined(__clang__) && defined(__GNUC__)
#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Warray-bounds"
#endif
        ASSERT(idx < N);
        return *storage[idx];
#if !defined(__clang__) && defined(__GNUC__)
#pragma GCC diagnostic pop
#endif
    }

    /**
     * @brief Nested iterator class for RefArray, based on indexing into the pointer array.
     * Provides semantics similar to what would be expected if std::array<T&, N> was possible.
     */
    class iterator {
      public:
        /**
         * @brief Constructs an iterator for a given RefArray object.
         *
         * @param array Pointer to the RefArray object.
         * @param pos The starting position in the array.
         */
        iterator(RefArray const* array, std::size_t pos)
            : array(array)
            , pos(pos)
        {}

        T& operator*() const
        {
            ASSERT(pos < N);
            return (*array)[pos];
        }

        iterator& operator++()
        {
            pos++;
            return *this;
        }

        iterator operator++(int)
        {
            iterator temp = *this;
            ++(*this);
            return temp;
        }

        bool operator==(iterator const& other) const { return pos == other.pos; }
        bool operator!=(iterator const& other) const { return pos != other.pos; }

      private:
        RefArray const* array;
        std::size_t pos;
    };

    constexpr std::size_t size() const { return N; }
    /**
     * @brief Returns an iterator to the beginning of the RefArray.
     *
     * @return An iterator to the first element.
     */
    iterator begin() const { return iterator(this, 0); }
    /**
     * @brief Returns an iterator to the end of the RefArray.
     *
     * @return An iterator to the element following the last element.
     */
    iterator end() const { return iterator(this, N); }

    T** get_storage() { return storage; }
    T* const* get_storage() const { return storage; }

  private:
    // We are making a high-level array, for simplicity having a C array as backing makes sense.
    // NOLINTNEXTLINE(cppcoreguidelines-avoid-c-arrays)
    T* storage[N];
};

/**
 * @brief Deduction guide for the RefArray class.
 * Allows for RefArray {a, b, c} without explicit template params.
 */
template <typename T, typename... Ts> RefArray(T&, Ts&...) -> RefArray<T, 1 + sizeof...(Ts)>;

/**
 * @brief Concatenates multiple RefArray objects into a single RefArray.
 *
 * This function takes multiple RefArray objects as input and concatenates them into a single
 * RefArray.

 * @tparam T The type of elements in the RefArray.
 * @tparam Ns The sizes of the input RefArrays.
 * @param ref_arrays The RefArray objects to be concatenated.
 * @return RefArray object containing all elements from the input arrays.
 */
template <typename T, std::size_t... Ns>
RefArray<T, (Ns + ...)> constexpr concatenate(const RefArray<T, Ns>&... ref_arrays)
{
    // Fold expression to calculate the total size of the new array using fold expression
    constexpr std::size_t TotalSize = (Ns + ...);
    RefArray<T, TotalSize> concatenated;

    std::size_t offset = 0;
    // Copies elements from a given RefArray to the concatenated array
    auto copy_into = [&](const auto& ref_array, std::size_t& offset) {
        for (std::size_t i = 0; i < ref_array.size(); ++i) {
            concatenated.get_storage()[offset + i] = &ref_array[i];
        }
        offset += ref_array.size();
    };

    // Fold expression to copy elements from each input RefArray to the concatenated array
    (..., copy_into(ref_arrays, offset));

    return concatenated;
}
} // namespace bb