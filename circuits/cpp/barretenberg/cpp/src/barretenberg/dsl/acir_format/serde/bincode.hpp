// Copyright (c) Facebook, Inc. and its affiliates
// SPDX-License-Identifier: MIT OR Apache-2.0

#pragma once

#include <cstdint>
#include <limits>

#include "binary.hpp"
#include "serde.hpp"

// Maximum length supported in practice (e.g. Java).
constexpr size_t BINCODE_MAX_LENGTH = (1ull << 31) - 1;

namespace serde {

class BincodeSerializer : public BinarySerializer<BincodeSerializer> {
    using Parent = BinarySerializer<BincodeSerializer>;

  public:
    BincodeSerializer()
        : Parent(SIZE_MAX)
    {}

    void serialize_f32(float value);
    void serialize_f64(double value);
    void serialize_len(size_t value);
    void serialize_variant_index(uint32_t value);

    static constexpr bool enforce_strict_map_ordering = false;
};

class BincodeDeserializer : public BinaryDeserializer<BincodeDeserializer> {
    using Parent = BinaryDeserializer<BincodeDeserializer>;

  public:
    BincodeDeserializer(std::vector<uint8_t> bytes)
        : Parent(std::move(bytes), SIZE_MAX)
    {}

    float deserialize_f32();
    double deserialize_f64();
    size_t deserialize_len();
    uint32_t deserialize_variant_index();

    static constexpr bool enforce_strict_map_ordering = false;
};

// Native floats and doubles must be IEEE-754 values of the expected size.
static_assert(std::numeric_limits<float>::is_iec559);
static_assert(std::numeric_limits<double>::is_iec559);
static_assert(sizeof(float) == sizeof(uint32_t));
static_assert(sizeof(double) == sizeof(uint64_t));

inline void BincodeSerializer::serialize_f32(float value)
{
    Parent::serialize_u32(*reinterpret_cast<uint32_t*>(&value));
}

inline void BincodeSerializer::serialize_f64(double value)
{
    Parent::serialize_u64(*reinterpret_cast<uint64_t*>(&value));
}

inline void BincodeSerializer::serialize_len(size_t value)
{
    if (value > BINCODE_MAX_LENGTH) {
        throw_or_abort("Length is too large");
    }
    Parent::serialize_u64((uint64_t)value);
}

inline void BincodeSerializer::serialize_variant_index(uint32_t value)
{
    Parent::serialize_u32((uint32_t)value);
}

inline float BincodeDeserializer::deserialize_f32()
{
    auto value = Parent::deserialize_u32();
    return *reinterpret_cast<float*>(&value);
}

inline double BincodeDeserializer::deserialize_f64()
{
    auto value = Parent::deserialize_u64();
    return *reinterpret_cast<double*>(&value);
}

inline size_t BincodeDeserializer::deserialize_len()
{
    auto value = (size_t)Parent::deserialize_u64();
    if (value > BINCODE_MAX_LENGTH) {
        throw_or_abort("Length is too large");
    }
    return (size_t)value;
}

inline uint32_t BincodeDeserializer::deserialize_variant_index()
{
    return Parent::deserialize_u32();
}

} // end of namespace serde