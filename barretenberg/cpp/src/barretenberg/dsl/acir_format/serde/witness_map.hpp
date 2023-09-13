#pragma once

#include "bincode.hpp"
#include "serde.hpp"

namespace WitnessMap {

struct Witness {
    uint32_t value;

    friend bool operator==(const Witness&, const Witness&);
    bool operator<(Witness const& rhs) const { return value < rhs.value; }
    std::vector<uint8_t> bincodeSerialize() const;
    static Witness bincodeDeserialize(std::vector<uint8_t>);
};

struct WitnessMap {
    std::map<Witness, std::string> value;

    friend bool operator==(const WitnessMap&, const WitnessMap&);
    std::vector<uint8_t> bincodeSerialize() const;
    static WitnessMap bincodeDeserialize(std::vector<uint8_t>);
};

} // end of namespace WitnessMap

namespace WitnessMap {

inline bool operator==(const Witness& lhs, const Witness& rhs)
{
    if (!(lhs.value == rhs.value)) {
        return false;
    }
    return true;
}

inline std::vector<uint8_t> Witness::bincodeSerialize() const
{
    auto serializer = serde::BincodeSerializer();
    serde::Serializable<Witness>::serialize(*this, serializer);
    return std::move(serializer).bytes();
}

inline Witness Witness::bincodeDeserialize(std::vector<uint8_t> input)
{
    auto deserializer = serde::BincodeDeserializer(input);
    auto value = serde::Deserializable<Witness>::deserialize(deserializer);
    if (deserializer.get_buffer_offset() < input.size()) {
        throw_or_abort("Some input bytes were not read");
    }
    return value;
}

} // end of namespace WitnessMap

template <>
template <typename Serializer>
void serde::Serializable<WitnessMap::Witness>::serialize(const WitnessMap::Witness& obj, Serializer& serializer)
{
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
WitnessMap::Witness serde::Deserializable<WitnessMap::Witness>::deserialize(Deserializer& deserializer)
{
    deserializer.increase_container_depth();
    WitnessMap::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace WitnessMap {

inline bool operator==(const WitnessMap& lhs, const WitnessMap& rhs)
{
    if (!(lhs.value == rhs.value)) {
        return false;
    }
    return true;
}

inline std::vector<uint8_t> WitnessMap::bincodeSerialize() const
{
    auto serializer = serde::BincodeSerializer();
    serde::Serializable<WitnessMap>::serialize(*this, serializer);
    return std::move(serializer).bytes();
}

inline WitnessMap WitnessMap::bincodeDeserialize(std::vector<uint8_t> input)
{
    auto deserializer = serde::BincodeDeserializer(input);
    auto value = serde::Deserializable<WitnessMap>::deserialize(deserializer);
    if (deserializer.get_buffer_offset() < input.size()) {
        throw_or_abort("Some input bytes were not read");
    }
    return value;
}

} // end of namespace WitnessMap

template <>
template <typename Serializer>
void serde::Serializable<WitnessMap::WitnessMap>::serialize(const WitnessMap::WitnessMap& obj, Serializer& serializer)
{
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
WitnessMap::WitnessMap serde::Deserializable<WitnessMap::WitnessMap>::deserialize(Deserializer& deserializer)
{
    deserializer.increase_container_depth();
    WitnessMap::WitnessMap obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}