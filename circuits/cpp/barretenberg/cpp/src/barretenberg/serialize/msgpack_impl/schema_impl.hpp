#pragma once

#include "schema_name.hpp"
#include <array>
#include <concepts>
#include <memory>
#include <string>

struct MsgpackSchemaPacker;

// Forward declare for MsgpackSchemaPacker
template <typename T> inline void _msgpack_schema_pack(MsgpackSchemaPacker& packer, const T& obj);

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
    template <typename T> void pack_schema(const T& obj) { _msgpack_schema_pack(*this, obj); }

    // Recurse over any templated containers
    // Outputs e.g. ['vector', ['sub-type']]
    template <typename... Args> void pack_template_type(const std::string& schema_name)
    {
        // We will pack a size 2 tuple
        pack_array(2);
        pack(schema_name);
        pack_array(sizeof...(Args));

        // Note: if this fails to compile, check first in list of template Arg's
        // it may need a msgpack_schema_pack specialization (particularly if it doesn't define MSGPACK_FIELDS).
        (_msgpack_schema_pack(*this, *std::make_unique<Args>()), ...); /* pack schemas of all template Args */
    }
    /**
     * @brief Encode a type that defines msgpack based on its key value pairs.
     *
     * @tparam T the msgpack()'able type
     * @param packer Our special packer.
     * @param object The object in question.
     */
    template <msgpack_concepts::HasMsgPack T> void pack_with_name(const std::string& type, T const& object)
    {
        if (set_emitted(type)) {
            pack(type);
            return; // already emitted
        }
        msgpack::check_msgpack_usage(object);
        // Encode as map
        const_cast<T&>(object).msgpack([&](auto&... args) {
            size_t kv_size = sizeof...(args);
            // Calculate the number of entries in our map (half the size of keys + values, plus the typename)
            pack_map(uint32_t(1 + kv_size / 2));
            pack("__typename");
            pack(type);
            // Pack the map content based on the args to msgpack
            _schema_pack_map_content(*this, args...);
        });
    }
};

// Helper for packing (key, value, key, value, ...) arguments
inline void _schema_pack_map_content(MsgpackSchemaPacker&)
{
    // base case
}

namespace msgpack_concepts {
template <typename T>
concept SchemaPackable = requires(T value, MsgpackSchemaPacker packer) { msgpack_schema_pack(packer, value); };
} // namespace msgpack_concepts

// Helper for packing (key, value, key, value, ...) arguments
template <typename Value, typename... Rest>
inline void _schema_pack_map_content(MsgpackSchemaPacker& packer,
                                     std::string key,
                                     const Value& value,
                                     const Rest&... rest)
{
    static_assert(
        msgpack_concepts::SchemaPackable<Value>,
        "see the first type argument in the error trace, it might require a specialization of msgpack_schema_pack");
    packer.pack(key);
    msgpack_schema_pack(packer, value);
    _schema_pack_map_content(packer, rest...);
}

template <typename T>
    requires(!msgpack_concepts::HasMsgPackSchema<T> && !msgpack_concepts::HasMsgPack<T>)
inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, T const& obj)
{
    packer.pack(msgpack_schema_name(obj));
}

/**
 * Schema pack base case for types with no special msgpack method.
 * @tparam T the type.
 * @param packer the schema packer.
 */
template <msgpack_concepts::HasMsgPackSchema T>
inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, T const& obj)
{
    obj.msgpack_schema(packer);
}

/**
 * @brief Encode a type that defines msgpack based on its key value pairs.
 *
 * @tparam T the msgpack()'able type
 * @param packer Our special packer.
 * @param object The object in question.
 */
template <msgpack_concepts::HasMsgPack T>
    requires(!msgpack_concepts::HasMsgPackSchema<T>)
inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, T const& object)
{
    std::string type = msgpack_schema_name(object);
    packer.pack_with_name(type, object);
}

/**
 * @brief Helper method for better error reporting. Clang does not give the best errors for argument lists.
 */
template <typename T> inline void _msgpack_schema_pack(MsgpackSchemaPacker& packer, const T& obj)
{
    static_assert(msgpack_concepts::SchemaPackable<T>,
                  "see the first type argument in the error trace, it might need a msgpack_schema method!");
    msgpack_schema_pack(packer, obj);
}

template <typename... Args> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::tuple<Args...> const&)
{
    packer.pack_template_type<Args...>("tuple");
}

template <typename K, typename V> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::map<K, V> const&)
{
    packer.pack_template_type<K, V>("map");
}

template <typename T> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::optional<T> const&)
{
    packer.pack_template_type<T>("optional");
}

template <typename T> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::vector<T> const&)
{
    packer.pack_template_type<T>("vector");
}

template <typename... Args> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::variant<Args...> const&)
{
    packer.pack_template_type<Args...>("variant");
}

template <typename T> inline void msgpack_schema_pack(MsgpackSchemaPacker& packer, std::shared_ptr<T> const&)
{
    packer.pack_template_type<T>("shared_ptr");
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
    // To avoid WASM problems with large stack objects, we use a heap allocation.
    // Small note: This works because make_unique goes of scope only when the whole line is done.
    _msgpack_schema_pack(packer, *std::make_unique<T>());
    packer.pack(N);
}

/**
 * @brief Print's an object's derived msgpack schema as a string.
 *
 * @param obj The object to print schema of.
 * @return std::string The schema as a string.
 */
inline std::string msgpack_schema_to_string(const auto& obj)
{
    msgpack::sbuffer output;
    MsgpackSchemaPacker printer{ output };
    _msgpack_schema_pack(printer, obj);
    msgpack::object_handle oh = msgpack::unpack(output.data(), output.size());
    std::stringstream pretty_output;
    pretty_output << oh.get() << std::endl;
    return pretty_output.str();
}
