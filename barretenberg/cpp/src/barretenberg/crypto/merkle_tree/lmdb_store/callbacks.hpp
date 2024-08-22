#pragma once

#include "barretenberg/crypto/merkle_tree/types.hpp"
#include "barretenberg/numeric/uint128/uint128.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include <cstdint>
#include <lmdb.h>
#include <vector>

namespace bb::crypto::merkle_tree {
using NodeKeyType = uint128_t;
using LeafIndexKeyType = uint64_t;
using FrKeyType = uint256_t;
using MetaKeyType = uint8_t;

void throw_error(const std::string& errorString, int error);

int size_cmp(const MDB_val* a, const MDB_val* b);

int lexico_cmp(const MDB_val*, const MDB_val*);

NodeKeyType get_key_for_node(uint32_t level, index_t index);

std::vector<uint8_t> serialise_key(uint8_t key);
std::vector<uint8_t> serialise_key(uint64_t key);
std::vector<uint8_t> serialise_key(uint128_t key);
std::vector<uint8_t> serialise_key(uint256_t key);

void deserialise_key(void* data, uint8_t& key);
void deserialise_key(void* data, uint64_t& key);
void deserialise_key(void* data, uint128_t& key);
void deserialise_key(void* data, uint256_t& key);

template <typename T> int value_cmp(const MDB_val* a, const MDB_val* b)
{
    T lhs;
    T rhs;
    deserialise_key(a->mv_data, lhs);
    deserialise_key(b->mv_data, rhs);
    if (lhs < rhs) {
        return -1;
    }
    if (lhs > rhs) {
        return 1;
    }
    return 0;
}

int integer_key_cmp(const MDB_val* a, const MDB_val* b);
std::vector<uint8_t> mdb_val_to_vector(const MDB_val& dbVal);
void copy_to_vector(const MDB_val& dbVal, std::vector<uint8_t>& target);

template <typename... TArgs> bool call_lmdb_func(int (*f)(TArgs...), TArgs... args)
{
    int error = f(args...);
    return error == 0;
}

template <typename... TArgs> void call_lmdb_func(const std::string& errorString, int (*f)(TArgs...), TArgs... args)
{
    int error = f(args...);
    if (error != 0) {
        throw_error(errorString, error);
    }
}

template <typename... TArgs> void call_lmdb_func(void (*f)(TArgs...), TArgs... args)
{
    f(args...);
}
} // namespace bb::crypto::merkle_tree