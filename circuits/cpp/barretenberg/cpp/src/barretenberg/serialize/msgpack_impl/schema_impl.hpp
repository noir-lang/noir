#pragma once

#include <memory>
#include <concepts>
#include <string>
#include <array>
#include "schema_name.hpp"

/**
 * Define a serialization schema based on compile-time information about a type being serialized.
 * This is then consumed by typescript to make bindings.
 */
struct MsgpackSchemaPacker : msgpack::packer<msgpack::sbuffer> {
    MsgpackSchemaPacker(msgpack::sbuffer& stream)
        : packer<msgpack::sbuffer>(stream)
    {}
    // For tracking emitted types
    std::set<std::string> emitted_types;
    // Returns if already was emitted
    bool set_emitted(const std::string& type)
    {
        if (emitted_types.find(type) == emitted_types.end()) {
            emitted_types.insert(type);
            return false;
        }
        return true;
    }

    /**
     * Pack a type indicating it is an alias of a certain msgpack type
     * Packs in the form ["alias", [schema_name, msgpack_name]]
     * @param schema_name The CPP type.
     * @param msgpack_name The msgpack type.
     */
    void pack_alias(const std::string& schema_name, const std::string& msgpack_name)
    {
        // We will pack a size 2 tuple
        pack_array(2);
        pack("alias");
        // That has a size 2 tuple as its 2nd arg
        pack_array(2);
        pack(schema_name);
        pack(msgpack_name);
    }

    /**
     * Pack the schema of a given object.
     * @tparam T the object's type.
     * @param obj the object.
     */
    template <typename T> void pack_schema(const T& obj) { msgpack_schema_pack(*this, obj); }
};

// Helper for packing (key, value, key, value, ...) arguments
inline void _schema_pack_map_content(MsgpackSchemaPacker&)
{
    // base case
}

namespace msgpack_concepts {
template <typename T> concept SchemaPackable = requires(T value, MsgpackSchemaPacker packer)
{
    msgpack_schema_pack(packer, value);
};
} // namespace msgpack_concepts

// Helper for packing (key, value, key, value, ...) arguments
template <typename Value, typename... Rest>
inline void _schema_pack_map_content(MsgpackSchemaPacker& packer, std::string key, Value value, Rest... rest)
{
    static_assert(
        msgpack_concepts::SchemaPackable<Value>,
        "see the first type argument in the error trace, it might require a specialization of msgpack_schema_pack");
    packer.pack(key);
    msgpack_schema_pack(packer, value);
    _schema_pack_map_content(packer, rest...);
}

/**
 * Schema pack base case for types with no special msgpack method.
 * @tparam T the type.
 * @param packer the schema packer.
 */
template <typename T>
requires(!msgpack_concepts::HasMsgPack<T> &&
         !msgpack_concepts::HasMsgPackPack<T>) inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, T const&)
{
    packer.pack(msgpack_schema_name(T{}));
}

/**
 * @brief Encode a type that defines msgpack based on its key value pairs.
 *
 * @tparam T the msgpack()'able type
 * @param packer Our special packer.
 * @param object The object in question.
 */
template <msgpack_concepts::HasMsgPack T> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, T const& object)
{
    std::string type = msgpack_schema_name(object);
    if (packer.set_emitted(type)) {
        packer.pack(type);
        return; // already emitted
    }
    msgpack::check_msgpack_usage(object);
    // Encode as map
    const_cast<T&>(object).msgpack([&](auto&... args) {
        size_t kv_size = sizeof...(args);
        // Calculate the number of entries in our map (half the size of keys + values, plus the typename)
        packer.pack_map(uint32_t(1 + kv_size / 2));
        packer.pack("__typename");
        packer.pack(type);
        // Pack the map content based on the args to msgpack
        _schema_pack_map_content(packer, args...);
    });
}

// Recurse over any templated containers
// Outputs e.g. ['vector', ['sub-type']]
template <typename... Args>
inline void _msgpack_schema_pack(MsgpackSchemaPacker& packer, const std::string& schema_name)
{
    // We will pack a size 2 tuple
    packer.pack_array(2);
    packer.pack(schema_name);
    packer.pack_array(sizeof...(Args));
    // helper for better errors
    auto pack = [&](auto arg) {
        static_assert(msgpack_concepts::SchemaPackable<decltype(arg)>,
                      "see the type argument of this lambda in the error trace, it might require a specialization of "
                      "msgpack_schema_pack");
        msgpack_schema_pack(packer, arg);
    };

    // Note: if this fails to compile, check first in list of template Arg's
    // it may need a msgpack_schema_pack specialization (particularly if it doesn't define MSGPACK_FIELDS).
    (pack(Args{}), ...); /* pack schemas of all template Args */
}
template <typename... Args> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::tuple<Args...> const&)
{
    _msgpack_schema_pack<Args...>(packer, "tuple");
}
template <typename K, typename V> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::map<K, V> const&)
{
    _msgpack_schema_pack<K, V>(packer, "map");
}
template <typename T> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::optional<T> const&)
{
    _msgpack_schema_pack<T>(packer, "optional");
}
template <typename T> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::vector<T> const&)
{
    _msgpack_schema_pack<T>(packer, "vector");
}
template <typename... Args> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::variant<Args...> const&)
{
    _msgpack_schema_pack<Args...>(packer, "variant");
}
template <typename T> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::shared_ptr<T> const&)
{
    _msgpack_schema_pack<T>(packer, "shared_ptr");
}

// Outputs e.g. ['array', ['array-type', 'N']]
template <typename T, std::size_t N>
inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::array<T, N> const&)
{
    // We will pack a size 2 tuple
    packer.pack_array(2);
    packer.pack("array");
    // That has a size 2 tuple as its 2nd arg
    packer.pack_array(2); /* param list format for consistency*/
    msgpack_schema_pack(packer, T{});
    packer.pack(N);
}

/**
 * @brief Print's an object's derived msgpack schema as a string.
 *
 * @param obj The object to print schema of.
 * @return std::string The schema as a string.
 */
inline std::string msgpack_schema_to_string(auto obj)
{
    msgpack::sbuffer output;
    MsgpackSchemaPacker printer{ output };
    msgpack_schema_pack(printer, obj);
    msgpack::object_handle oh = msgpack::unpack(output.data(), output.size());
    std::stringstream pretty_output;
    pretty_output << oh.get() << std::endl;
    return pretty_output.str();
}
