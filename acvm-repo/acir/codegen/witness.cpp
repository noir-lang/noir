#pragma once

#include "serde.hpp"
#include "bincode.hpp"

namespace WitnessStack {

    struct Witness {
        uint32_t value;

        friend bool operator==(const Witness&, const Witness&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Witness bincodeDeserialize(std::vector<uint8_t>);
    };

    struct WitnessMap {
        std::map<WitnessStack::Witness, std::string> value;

        friend bool operator==(const WitnessMap&, const WitnessMap&);
        std::vector<uint8_t> bincodeSerialize() const;
        static WitnessMap bincodeDeserialize(std::vector<uint8_t>);
    };

    struct StackItem {
        uint32_t index;
        WitnessStack::WitnessMap witness;

        friend bool operator==(const StackItem&, const StackItem&);
        std::vector<uint8_t> bincodeSerialize() const;
        static StackItem bincodeDeserialize(std::vector<uint8_t>);
    };

    struct WitnessStack {
        std::vector<WitnessStack::StackItem> stack;

        friend bool operator==(const WitnessStack&, const WitnessStack&);
        std::vector<uint8_t> bincodeSerialize() const;
        static WitnessStack bincodeDeserialize(std::vector<uint8_t>);
    };

} // end of namespace WitnessStack


namespace WitnessStack {

    inline bool operator==(const StackItem &lhs, const StackItem &rhs) {
        if (!(lhs.index == rhs.index)) { return false; }
        if (!(lhs.witness == rhs.witness)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> StackItem::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<StackItem>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline StackItem StackItem::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<StackItem>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace WitnessStack

template <>
template <typename Serializer>
void serde::Serializable<WitnessStack::StackItem>::serialize(const WitnessStack::StackItem &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.index)>::serialize(obj.index, serializer);
    serde::Serializable<decltype(obj.witness)>::serialize(obj.witness, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
WitnessStack::StackItem serde::Deserializable<WitnessStack::StackItem>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    WitnessStack::StackItem obj;
    obj.index = serde::Deserializable<decltype(obj.index)>::deserialize(deserializer);
    obj.witness = serde::Deserializable<decltype(obj.witness)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace WitnessStack {

    inline bool operator==(const Witness &lhs, const Witness &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> Witness::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<Witness>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline Witness Witness::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<Witness>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace WitnessStack

template <>
template <typename Serializer>
void serde::Serializable<WitnessStack::Witness>::serialize(const WitnessStack::Witness &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
WitnessStack::Witness serde::Deserializable<WitnessStack::Witness>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    WitnessStack::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace WitnessStack {

    inline bool operator==(const WitnessMap &lhs, const WitnessMap &rhs) {
        if (!(lhs.value == rhs.value)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> WitnessMap::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<WitnessMap>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline WitnessMap WitnessMap::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<WitnessMap>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace WitnessStack

template <>
template <typename Serializer>
void serde::Serializable<WitnessStack::WitnessMap>::serialize(const WitnessStack::WitnessMap &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
WitnessStack::WitnessMap serde::Deserializable<WitnessStack::WitnessMap>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    WitnessStack::WitnessMap obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace WitnessStack {

    inline bool operator==(const WitnessStack &lhs, const WitnessStack &rhs) {
        if (!(lhs.stack == rhs.stack)) { return false; }
        return true;
    }

    inline std::vector<uint8_t> WitnessStack::bincodeSerialize() const {
        auto serializer = serde::BincodeSerializer();
        serde::Serializable<WitnessStack>::serialize(*this, serializer);
        return std::move(serializer).bytes();
    }

    inline WitnessStack WitnessStack::bincodeDeserialize(std::vector<uint8_t> input) {
        auto deserializer = serde::BincodeDeserializer(input);
        auto value = serde::Deserializable<WitnessStack>::deserialize(deserializer);
        if (deserializer.get_buffer_offset() < input.size()) {
            throw serde::deserialization_error("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace WitnessStack

template <>
template <typename Serializer>
void serde::Serializable<WitnessStack::WitnessStack>::serialize(const WitnessStack::WitnessStack &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.stack)>::serialize(obj.stack, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
WitnessStack::WitnessStack serde::Deserializable<WitnessStack::WitnessStack>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    WitnessStack::WitnessStack obj;
    obj.stack = serde::Deserializable<decltype(obj.stack)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}
