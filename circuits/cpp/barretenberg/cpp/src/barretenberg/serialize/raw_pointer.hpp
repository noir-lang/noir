#pragma once
#include "msgpack_impl/schema_name.hpp"
#include <cstdint>

// Holds a raw pointer to an object of type T.
// It provides methods for packing and unpacking the pointer using MessagePack,
// a binary serialization format.
template <typename T> struct RawPointer {
    // Raw pointer to an object of type T
    T* ptr = nullptr;

    // Pack the raw pointer into a MessagePack packer.
    // The pointer is first cast to an integer type (uintptr_t) which can hold a pointer,
    // and then packed into the packer.
    void msgpack_pack(auto& packer) const { packer.pack(reinterpret_cast<uintptr_t>(ptr)); }

    // Unpack the raw pointer from a MessagePack object.
    // The object is first cast to an integer type (uintptr_t), and then to a pointer of type T.
    void msgpack_unpack(auto object) { ptr = reinterpret_cast<T*>((uintptr_t)object); }

    // help our msgpack schema compiler with this struct
    void msgpack_schema(auto& packer) const
    {
        // Manually make an ['alias', [type, 'int']] structure (without calling pack_alias as it is too restricting)
        packer.pack_array(2); // 2 elements in our outer tuple
        packer.pack("alias");
        packer.pack_array(2); // 2 elements in our inner tuple
        packer.template pack_template_type<T>("RawPointer");
        packer.pack("int");
    }

    // Overload the arrow operator to return the raw pointer.
    // This allows users to directly access the object pointed to by the raw pointer.
    T* operator->() { return ptr; }
};
