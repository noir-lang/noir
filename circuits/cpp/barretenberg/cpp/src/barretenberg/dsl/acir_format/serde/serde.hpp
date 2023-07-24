// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#pragma once

#include "barretenberg/common/throw_or_abort.hpp"
#include <array>
#include <cstdint>
#include <functional>
#include <map>
#include <memory>
#include <optional>
#include <stdexcept>
#include <string>
#include <tuple>
#include <type_traits>
#include <variant>
#include <vector>

namespace serde {

class serialization_error : public std::invalid_argument {
  public:
    explicit serialization_error(const std::string& what_arg)
        : std::invalid_argument(what_arg)
    {}
    explicit serialization_error(const char* what_arg)
        : std::invalid_argument(what_arg)
    {}
};

class deserialization_error : public std::invalid_argument {
  public:
    explicit deserialization_error(const std::string& what_arg)
        : std::invalid_argument(what_arg)
    {}
    explicit deserialization_error(const char* what_arg)
        : std::invalid_argument(what_arg)
    {}
};

// Basic implementation for 128-bit unsigned integers.
struct uint128_t {
    uint64_t high;
    uint64_t low;

    friend bool operator==(const uint128_t&, const uint128_t&);
};

inline bool operator==(const uint128_t& lhs, const uint128_t& rhs)
{
    return lhs.high == rhs.high && lhs.low == rhs.low;
}

// 128-bit signed integers.
struct int128_t {
    int64_t high;
    uint64_t low;

    friend bool operator==(const int128_t&, const int128_t&);
};

inline bool operator==(const int128_t& lhs, const int128_t& rhs)
{
    return lhs.high == rhs.high && lhs.low == rhs.low;
}

// A copyable unique_ptr with value semantics.
// Freely inspired by the following discussion:
// https://codereview.stackexchange.com/questions/103744/deepptr-a-deep-copying-unique-ptr-wrapper-in-c
template <typename T> class value_ptr {
  public:
    value_ptr()
        : ptr_(nullptr)
    {}

    value_ptr(const T& value)
        : ptr_(new T{ value })
    {}

    value_ptr(const value_ptr& other)
        : ptr_(nullptr)
    {
        if (other) {
            ptr_ = std::unique_ptr<T>{ new T{ *other } };
        }
    }

    value_ptr& operator=(const value_ptr& other)
    {
        value_ptr temp{ other };
        std::swap(ptr_, temp.ptr_);
        return *this;
    }

    value_ptr(value_ptr&& other) = default;

    value_ptr& operator=(value_ptr&& other) = default;

    T& operator*() { return *ptr_; }

    const T& operator*() const { return *ptr_; }

    T* const operator->() { return ptr_.operator->(); }

    const T* const operator->() const { return ptr_.operator->(); }

    const T* const get() const { return ptr_.get(); }

    operator bool() const { return (bool)ptr_; }

    template <typename U> friend bool operator==(const value_ptr<U>&, const value_ptr<U>&);

  private:
    std::unique_ptr<T> ptr_;
};

template <typename T> bool operator==(const value_ptr<T>& lhs, const value_ptr<T>& rhs)
{
    return *lhs == *rhs;
}

// Trait to enable serialization of values of type T.
// This is similar to the `serde::Serialize` trait in Rust.
template <typename T> struct Serializable {
    template <typename Serializer> static void serialize(const T& value, Serializer& serializer);
};

// Trait to enable deserialization of values of type T.
// This is similar to the `serde::Deserialize` trait in Rust.
template <typename T> struct Deserializable {
    template <typename Deserializer> static T deserialize(Deserializer& deserializer);
};

// --- Implementation of Serializable for base types ---

// string
template <> struct Serializable<std::string> {
    template <typename Serializer> static void serialize(const std::string& value, Serializer& serializer)
    {
        serializer.serialize_str(value);
    }
};

// unit
template <> struct Serializable<std::monostate> {
    template <typename Serializer> static void serialize(const std::monostate&, Serializer& serializer)
    {
        serializer.serialize_unit();
    }
};

// bool
template <> struct Serializable<bool> {
    template <typename Serializer> static void serialize(const bool& value, Serializer& serializer)
    {
        serializer.serialize_bool(value);
    }
};

// UTF-8 char
template <> struct Serializable<char32_t> {
    template <typename Serializer> static void serialize(const char32_t& value, Serializer& serializer)
    {
        serializer.serialize_char(value);
    }
};

// f32
template <> struct Serializable<float> {
    template <typename Serializer> static void serialize(const float& value, Serializer& serializer)
    {
        serializer.serialize_f32(value);
    }
};

// f64
template <> struct Serializable<double> {
    template <typename Serializer> static void serialize(const double& value, Serializer& serializer)
    {
        serializer.serialize_f64(value);
    }
};

// u8
template <> struct Serializable<uint8_t> {
    template <typename Serializer> static void serialize(const uint8_t& value, Serializer& serializer)
    {
        serializer.serialize_u8(value);
    }
};

// u16
template <> struct Serializable<uint16_t> {
    template <typename Serializer> static void serialize(const uint16_t& value, Serializer& serializer)
    {
        serializer.serialize_u16(value);
    }
};

// u32
template <> struct Serializable<uint32_t> {
    template <typename Serializer> static void serialize(const uint32_t& value, Serializer& serializer)
    {
        serializer.serialize_u32(value);
    }
};

// u64
template <> struct Serializable<uint64_t> {
    template <typename Serializer> static void serialize(const uint64_t& value, Serializer& serializer)
    {
        serializer.serialize_u64(value);
    }
};

// u128
template <> struct Serializable<uint128_t> {
    template <typename Serializer> static void serialize(const uint128_t& value, Serializer& serializer)
    {
        serializer.serialize_u128(value);
    }
};

// i8
template <> struct Serializable<int8_t> {
    template <typename Serializer> static void serialize(const int8_t& value, Serializer& serializer)
    {
        serializer.serialize_i8(value);
    }
};

// i16
template <> struct Serializable<int16_t> {
    template <typename Serializer> static void serialize(const int16_t& value, Serializer& serializer)
    {
        serializer.serialize_i16(value);
    }
};

// i32
template <> struct Serializable<int32_t> {
    template <typename Serializer> static void serialize(const int32_t& value, Serializer& serializer)
    {
        serializer.serialize_i32(value);
    }
};

// i64
template <> struct Serializable<int64_t> {
    template <typename Serializer> static void serialize(const int64_t& value, Serializer& serializer)
    {
        serializer.serialize_i64(value);
    }
};

// i128
template <> struct Serializable<int128_t> {
    template <typename Serializer> static void serialize(const int128_t& value, Serializer& serializer)
    {
        serializer.serialize_i128(value);
    }
};

// --- Derivation of Serializable for composite types ---

// Value pointers (non-nullable)
template <typename T> struct Serializable<value_ptr<T>> {
    template <typename Serializer> static void serialize(const value_ptr<T>& value, Serializer& serializer)
    {
        Serializable<T>::serialize(*value, serializer);
    }
};

// Options
template <typename T> struct Serializable<std::optional<T>> {
    template <typename Serializer> static void serialize(const std::optional<T>& option, Serializer& serializer)
    {
        if (option.has_value()) {
            serializer.serialize_option_tag(true);
            Serializable<T>::serialize(option.value(), serializer);
        } else {
            serializer.serialize_option_tag(false);
        }
    }
};

// Vectors (sequences)
template <typename T, typename Allocator> struct Serializable<std::vector<T, Allocator>> {
    template <typename Serializer> static void serialize(const std::vector<T, Allocator>& value, Serializer& serializer)
    {
        serializer.serialize_len(value.size());
        for (const T& item : value) {
            Serializable<T>::serialize(item, serializer);
        }
    }
};

// Fixed-size arrays
template <typename T, std::size_t N> struct Serializable<std::array<T, N>> {
    template <typename Serializer> static void serialize(const std::array<T, N>& value, Serializer& serializer)
    {
        for (const T& item : value) {
            Serializable<T>::serialize(item, serializer);
        }
    }
};

// Maps
template <typename K, typename V, typename Allocator> struct Serializable<std::map<K, V, Allocator>> {
    template <typename Serializer> static void serialize(const std::map<K, V, Allocator>& value, Serializer& serializer)
    {
        serializer.serialize_len(value.size());
        std::vector<size_t> offsets;
        for (const auto& item : value) {
            if constexpr (Serializer::enforce_strict_map_ordering) {
                offsets.push_back(serializer.get_buffer_offset());
            }
            Serializable<K>::serialize(item.first, serializer);
            Serializable<V>::serialize(item.second, serializer);
        }
        if constexpr (Serializer::enforce_strict_map_ordering) {
            serializer.sort_last_entries(std::move(offsets));
        }
    }
};

// Tuples
template <class... Types> struct Serializable<std::tuple<Types...>> {
    template <typename Serializer> static void serialize(const std::tuple<Types...>& value, Serializer& serializer)
    {
        // Visit each of the type components.
        std::apply([&serializer](Types const&... args) { (Serializable<Types>::serialize(args, serializer), ...); },
                   value);
    }
};

// Enums
template <class... Types> struct Serializable<std::variant<Types...>> {
    template <typename Serializer> static void serialize(const std::variant<Types...>& value, Serializer& serializer)
    {
        // Write the variant index.
        serializer.serialize_variant_index(value.index());
        // Visit the inner type.
        std::visit(
            [&serializer](const auto& arg) {
                using T = typename std::decay<decltype(arg)>::type;
                Serializable<T>::serialize(arg, serializer);
            },
            value);
    }
};

// --- Implementation of Deserializable for base types ---

// string
template <> struct Deserializable<std::string> {
    template <typename Deserializer> static std::string deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_str();
    }
};

// unit
template <> struct Deserializable<std::monostate> {
    template <typename Deserializer> static std::monostate deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_unit();
    }
};

// bool
template <> struct Deserializable<bool> {
    template <typename Deserializer> static bool deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_bool();
    }
};

// f32
template <> struct Deserializable<float> {
    template <typename Deserializer> static float deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_f32();
    }
};

// f64
template <> struct Deserializable<double> {
    template <typename Deserializer> static double deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_f64();
    }
};

// UTF-8 char
template <> struct Deserializable<char32_t> {
    template <typename Deserializer> static char32_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_char();
    }
};

// u8
template <> struct Deserializable<uint8_t> {
    template <typename Deserializer> static uint8_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_u8();
    }
};

// u16
template <> struct Deserializable<uint16_t> {
    template <typename Deserializer> static uint16_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_u16();
    }
};

// u32
template <> struct Deserializable<uint32_t> {
    template <typename Deserializer> static uint32_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_u32();
    }
};

// u64
template <> struct Deserializable<uint64_t> {
    template <typename Deserializer> static uint64_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_u64();
    }
};

// u128
template <> struct Deserializable<uint128_t> {
    template <typename Deserializer> static uint128_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_u128();
    }
};

// i8
template <> struct Deserializable<int8_t> {
    template <typename Deserializer> static int8_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_i8();
    }
};

// i16
template <> struct Deserializable<int16_t> {
    template <typename Deserializer> static int16_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_i16();
    }
};

// i32
template <> struct Deserializable<int32_t> {
    template <typename Deserializer> static int32_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_i32();
    }
};

// i64
template <> struct Deserializable<int64_t> {
    template <typename Deserializer> static int64_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_i64();
    }
};

// i128
template <> struct Deserializable<int128_t> {
    template <typename Deserializer> static int128_t deserialize(Deserializer& deserializer)
    {
        return deserializer.deserialize_i128();
    }
};

// --- Derivation of Deserializable for composite types ---

// Value pointers
template <typename T> struct Deserializable<value_ptr<T>> {
    template <typename Deserializer> static value_ptr<T> deserialize(Deserializer& deserializer)
    {
        return value_ptr<T>(Deserializable<T>::deserialize(deserializer));
    }
};

// Options
template <typename T> struct Deserializable<std::optional<T>> {
    template <typename Deserializer> static std::optional<T> deserialize(Deserializer& deserializer)
    {
        auto tag = deserializer.deserialize_option_tag();
        if (!tag) {
            return {};
        } else {
            return { Deserializable<T>::deserialize(deserializer) };
        }
    }
};

// Vectors
template <typename T, typename Allocator> struct Deserializable<std::vector<T, Allocator>> {
    template <typename Deserializer> static std::vector<T> deserialize(Deserializer& deserializer)
    {
        std::vector<T> result;
        size_t len = deserializer.deserialize_len();
        for (size_t i = 0; i < len; i++) {
            result.push_back(Deserializable<T>::deserialize(deserializer));
        }
        return result;
    }
};

// Maps
template <typename K, typename V> struct Deserializable<std::map<K, V>> {
    template <typename Deserializer> static std::map<K, V> deserialize(Deserializer& deserializer)
    {
        std::map<K, V> result;
        size_t len = deserializer.deserialize_len();
        std::optional<std::tuple<size_t, size_t>> previous_key_slice;
        for (size_t i = 0; i < len; i++) {
            if constexpr (Deserializer::enforce_strict_map_ordering) {
                auto start = deserializer.get_buffer_offset();
                auto key = Deserializable<K>::deserialize(deserializer);
                auto end = deserializer.get_buffer_offset();
                if (previous_key_slice.has_value()) {
                    deserializer.check_that_key_slices_are_increasing(previous_key_slice.value(), { start, end });
                }
                previous_key_slice = { start, end };
                auto value = Deserializable<V>::deserialize(deserializer);
                result.insert({ key, value });
            } else {
                auto key = Deserializable<K>::deserialize(deserializer);
                auto value = Deserializable<V>::deserialize(deserializer);
                result.insert({ key, value });
            }
        }
        return result;
    }
};

// Fixed-size arrays
template <typename T, std::size_t N> struct Deserializable<std::array<T, N>> {
    template <typename Deserializer> static std::array<T, N> deserialize(Deserializer& deserializer)
    {
        std::array<T, N> result;
        for (T& item : result) {
            item = Deserializable<T>::deserialize(deserializer);
        }
        return result;
    }
};

// Tuples
template <class... Types> struct Deserializable<std::tuple<Types...>> {
    template <typename Deserializer> static std::tuple<Types...> deserialize(Deserializer& deserializer)
    {
        // Visit each of the type components. We use the constructor of `std::tuple` so
        // that the evaluation order of arguments is specified by the C++ standard.
        return std::tuple<Types...>{ Deserializable<Types>::deserialize(deserializer)... };
    }
};

// Enums
template <class... Types> struct Deserializable<std::variant<Types...>> {
    template <typename Deserializer> static std::variant<Types...> deserialize(Deserializer& deserializer)
    {
        // A "case" is analog to a particular branch in switch-case over the
        // index. Given the variant type `T` known statically, we create a
        // closure that will deserialize a value `T` and return it as a variant.
        using Case = std::function<std::variant<Types...>(Deserializer&)>;
        auto make_case = [](auto tag) -> Case {
            // Obtain the type `T` encoded in the type of `tag ==
            // std::common_type<T>{}`.
            using T = typename decltype(tag)::type;
            auto f = [](Deserializer& de) { return std::variant<Types...>(Deserializable<T>::deserialize(de)); };
            return f;
        };

        // The static array of all the cases for this variant.
        static const std::array<Case, sizeof...(Types)> cases = { make_case(std::common_type<Types>{})... };

        // Read the variant index and execute the corresponding case.
        auto index = deserializer.deserialize_variant_index();
        if (index > cases.size()) {
            throw_or_abort("Unknown variant index for enum");
        }
        return cases.at(index)(deserializer);
    }
};

} // end of namespace serde