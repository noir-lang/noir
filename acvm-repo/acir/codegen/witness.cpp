#pragma once

#include "serde.hpp"
#include "msgpack.hpp"
#include "bincode.hpp"

namespace Witnesses {
    struct Helpers {
        static std::map<std::string, msgpack::object const*> make_kvmap(
            msgpack::object const& o,
            std::string const& name
        ) {
            if(o.type != msgpack::type::MAP) {
                std::cerr << o << std::endl;
                throw_or_abort("expected MAP for " + name);
            }
            std::map<std::string, msgpack::object const*> kvmap;
            for (uint32_t i = 0; i < o.via.map.size; ++i) {
                if (o.via.map.ptr[i].key.type != msgpack::type::STR) {
                    std::cerr << o << std::endl;
                    throw_or_abort("expected STR for keys of " + name);
                }
                kvmap.emplace(
                    std::string(
                        o.via.map.ptr[i].key.via.str.ptr,
                        o.via.map.ptr[i].key.via.str.size),
                    &o.via.map.ptr[i].val);
            }
            return kvmap;
        }
        template<typename T>
        static void conv_fld_from_kvmap(
            std::map<std::string, msgpack::object const*> const& kvmap,
            std::string const& struct_name,
            std::string const& field_name,
            T& field,
            bool is_optional
        ) {
            auto it = kvmap.find(field_name);
            if (it != kvmap.end()) {
                try {
                    it->second->convert(field);
                } catch (const msgpack::type_error&) {
                    std::cerr << *it->second << std::endl;
                    throw_or_abort("error converting into field " + struct_name + "::" + field_name);
                }
            } else if (!is_optional) {
                throw_or_abort("missing field: " + struct_name + "::" + field_name);
            }
        }
    };
    }

namespace Witnesses {

    struct Witness {
        uint32_t value;

        friend bool operator==(const Witness&, const Witness&);
        std::vector<uint8_t> bincodeSerialize() const;
        static Witness bincodeDeserialize(std::vector<uint8_t>);

        bool operator<(Witness const& rhs) const { return value < rhs.value; }void msgpack_pack(auto& packer) const { packer.pack(value); }

        void msgpack_unpack(msgpack::object const& o) {
            try {
                o.convert(value);
            } catch (const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting into newtype 'Witness'");
            }
        }
    };

    struct WitnessMap {
        std::map<Witnesses::Witness, std::string> value;

        friend bool operator==(const WitnessMap&, const WitnessMap&);
        std::vector<uint8_t> bincodeSerialize() const;
        static WitnessMap bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const { packer.pack(value); }

        void msgpack_unpack(msgpack::object const& o) {
            try {
                o.convert(value);
            } catch (const msgpack::type_error&) {
                std::cerr << o << std::endl;
                throw_or_abort("error converting into newtype 'WitnessMap'");
            }
        }
    };

    struct StackItem {
        uint32_t index;
        Witnesses::WitnessMap witness;

        friend bool operator==(const StackItem&, const StackItem&);
        std::vector<uint8_t> bincodeSerialize() const;
        static StackItem bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(2);
            packer.pack(std::make_pair("index", index));
            packer.pack(std::make_pair("witness", witness));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "StackItem";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "index", index, false);
            Helpers::conv_fld_from_kvmap(kvmap, name, "witness", witness, false);
        }
    };

    struct WitnessStack {
        std::vector<Witnesses::StackItem> stack;

        friend bool operator==(const WitnessStack&, const WitnessStack&);
        std::vector<uint8_t> bincodeSerialize() const;
        static WitnessStack bincodeDeserialize(std::vector<uint8_t>);

        void msgpack_pack(auto& packer) const {
            packer.pack_map(1);
            packer.pack(std::make_pair("stack", stack));
        }

        void msgpack_unpack(msgpack::object const& o) {
            auto name = "WitnessStack";
            auto kvmap = Helpers::make_kvmap(o, name);
            Helpers::conv_fld_from_kvmap(kvmap, name, "stack", stack, false);
        }
    };

} // end of namespace Witnesses


namespace Witnesses {

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
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Witnesses

template <>
template <typename Serializer>
void serde::Serializable<Witnesses::StackItem>::serialize(const Witnesses::StackItem &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.index)>::serialize(obj.index, serializer);
    serde::Serializable<decltype(obj.witness)>::serialize(obj.witness, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Witnesses::StackItem serde::Deserializable<Witnesses::StackItem>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Witnesses::StackItem obj;
    obj.index = serde::Deserializable<decltype(obj.index)>::deserialize(deserializer);
    obj.witness = serde::Deserializable<decltype(obj.witness)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Witnesses {

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
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Witnesses

template <>
template <typename Serializer>
void serde::Serializable<Witnesses::Witness>::serialize(const Witnesses::Witness &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Witnesses::Witness serde::Deserializable<Witnesses::Witness>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Witnesses::Witness obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Witnesses {

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
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Witnesses

template <>
template <typename Serializer>
void serde::Serializable<Witnesses::WitnessMap>::serialize(const Witnesses::WitnessMap &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.value)>::serialize(obj.value, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Witnesses::WitnessMap serde::Deserializable<Witnesses::WitnessMap>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Witnesses::WitnessMap obj;
    obj.value = serde::Deserializable<decltype(obj.value)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}

namespace Witnesses {

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
            throw_or_abort("Some input bytes were not read");
        }
        return value;
    }

} // end of namespace Witnesses

template <>
template <typename Serializer>
void serde::Serializable<Witnesses::WitnessStack>::serialize(const Witnesses::WitnessStack &obj, Serializer &serializer) {
    serializer.increase_container_depth();
    serde::Serializable<decltype(obj.stack)>::serialize(obj.stack, serializer);
    serializer.decrease_container_depth();
}

template <>
template <typename Deserializer>
Witnesses::WitnessStack serde::Deserializable<Witnesses::WitnessStack>::deserialize(Deserializer &deserializer) {
    deserializer.increase_container_depth();
    Witnesses::WitnessStack obj;
    obj.stack = serde::Deserializable<decltype(obj.stack)>::deserialize(deserializer);
    deserializer.decrease_container_depth();
    return obj;
}
