#pragma once
#include <cstdint>
#include "msgpack_impl/schema_name.hpp"

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

    // Overload the arrow operator to return the raw pointer.
    // This allows users to directly access the object pointed to by the raw pointer.
    T* operator->() { return ptr; }
};

// help our msgpack schema compiler with this struct
template <typename T> inline void msgpack_schema_pack(auto& packer, RawPointer<T> const&)
{
    packer.pack_alias(msgpack_schema_name(T{}) + "Ptr", "int");
}
