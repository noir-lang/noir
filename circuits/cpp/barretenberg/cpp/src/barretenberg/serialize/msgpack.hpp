#pragma once
/* Minimal header for declaring msgpack fields.
This should be included as "barretenberg/serialize/msgpack.hpp" unless a translation wants
to use msgpack for bindings, then "barretenberg/serialize/cbind.hpp" should be included.

## Overview

The Msgpack module allows for efficient serialization and deserialization of data structures. It can be applied to
map-like objects, array-like objects, and custom serialization/deserialization logic.

## Binding objects

Marking structs/classes with their fields for msgpack allows you to pack and unpack the class.

1. All objects bound should have a default constructor
2. Objects can be tightly packed as binary (see field_impl.hpp), array-like, or map-like. See below
3. You should list all fields of a class in the below methods, or use the custom method.

### Typical Objects

To make objects serializable as a map-like format, define the `msgpack` method in your class as follows:

```cpp
void msgpack(auto ar) {
    ar(NVP(circuit_type, circuit_size, num_public_inputs, commitments, contains_recursive_proof,
recursive_proof_public_input_indices));
}
or
MSGPACK_FIELDS(circuit_type, circuit_size, num_public_inputs, commitments, contains_recursive_proof,
recursive_proof_public_input_indices);
```

This approach assumes 1. all members are default constructible 2. you give it all members 3. all members are writable
references

This method maps the object's properties (e.g., `circuit_type`, `circuit_size`, etc.) to their respective keys in the
serialized data.


### Custom Serialization and Deserialization

For custom serialization and deserialization, define `msgpack_pack` and `msgpack_unpack` methods in your class:

```cpp
// For serialization
template <class Params> void field<Params>::msgpack_pack(auto& packer) const
{
    auto adjusted = from_montgomery_form();
    uint64_t bin_data[4] = {
        htonll(adjusted.data[3]), htonll(adjusted.data[2]), htonll(adjusted.data[1]), htonll(adjusted.data[0])
    };
    packer.pack_bin(sizeof(bin_data));
    packer.pack_bin_body((const char*)bin_data, sizeof(bin_data));
}

// For deserialization
template <class Params> void field<Params>::msgpack_unpack(auto o)
{
    msgpack::read_bin64(o, data, 4);
    uint64_t reversed[] = {data[3], data[2], data[1], data[0]};
    for (int i = 0; i < 4; i++) {
        data[i] = reversed[i];
    }
    *this = to_montgomery_form();
}
```

These methods allow you to implement custom logic for the serialization and deserialization processes.


## Packing/Unpacking

Only when actually using msgpack to write or read data, include "barretenberg/serialize/cbind.hpp".
You can then use msgpack library features to serialize and deserialize C++ objects.

e.g. packing
```
    // Create a buffer to store the encoded data
    msgpack::sbuffer buffer;
    msgpack::pack(buffer, obj);

    uint8_t* output = (uint8_t*)aligned_alloc(64, buffer.size());
    memcpy(output, buffer.data(), buffer.size());
    // Convert the buffer data to a string and return it
    return { output, buffer.size() };
```

e.g. unpacking

```
    msgpack::unpack((const char*)encoded_data, encoded_data_size).get().convert(*value);
```
*/
#include "msgpack_impl/concepts.hpp"
#include "msgpack_impl/name_value_pair_macro.hpp"
#include <type_traits>

// Helper for above documented syntax
// Define a macro that takes any amount of parameters and expands to a msgpack method definition
// __VA__ARGS__ expands to the parmeters, comma separated.
#define MSGPACK_FIELDS(...)                                                                                            \
    void msgpack(auto pack_fn) { pack_fn(NVP(__VA_ARGS__)); }
