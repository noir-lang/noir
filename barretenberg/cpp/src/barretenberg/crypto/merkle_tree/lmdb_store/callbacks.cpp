#include "barretenberg/crypto/merkle_tree/lmdb_store/callbacks.hpp"
#include "barretenberg/numeric/uint256/uint256.hpp"
#include "lmdb.h"
#include <algorithm>
#include <cstdint>
#include <cstring>
#include <functional>
#include <vector>

namespace bb::crypto::merkle_tree {
void throw_error(const std::string& errorString, int error)
{
    std::stringstream ss;
    ss << errorString << ": " << error << " - " << mdb_strerror(error) << std::endl;
    throw std::runtime_error(ss.str());
}

std::vector<uint8_t> serialise_key(uint8_t key)
{
    return { key };
}

void deserialise_key(void* data, uint8_t& key)
{
    uint8_t* p = static_cast<uint8_t*>(data);
    key = *p;
}

std::vector<uint8_t> serialise_key(uint64_t key)
{
    const uint8_t* p = reinterpret_cast<uint8_t*>(&key);
    return std::vector<uint8_t>(p, p + sizeof(key));
}

void deserialise_key(void* data, uint64_t& key)
{
    std::memcpy(&key, data, sizeof(key));
}

std::vector<uint8_t> serialise_key(uint128_t key)
{
    std::vector<uint8_t> buf(16);
#ifdef __i386__
    std::memcpy(buf.data(), key.data, 16);
#else
    std::memcpy(buf.data(), &key, 16);
#endif
    return buf;
}

void deserialise_key(void* data, uint128_t& key)
{
#ifdef __i386__
    std::memcpy(key.data, data, 16);
#else
    std::memcpy(&key, data, 16);
#endif
}

std::vector<uint8_t> serialise_key(uint256_t key)
{
    std::vector<uint8_t> buf(32);
    std::memcpy(buf.data(), key.data, 32);
    return buf;
}

void deserialise_key(void* data, uint256_t& key)
{
    std::memcpy(key.data, data, 32);
}

// Nodes are stored as a heap
NodeKeyType get_key_for_node(uint32_t level, index_t index)
{
    NodeKeyType key = static_cast<NodeKeyType>(1) << level;
    key += static_cast<NodeKeyType>(index);
    return key - 1;
}

int size_cmp(const MDB_val* a, const MDB_val* b)
{
    if (a->mv_size < b->mv_size) {
        return -1;
    }
    if (a->mv_size > b->mv_size) {
        return 1;
    }
    return 0;
}

std::vector<uint8_t> mdb_val_to_vector(const MDB_val& dbVal)
{
    const uint8_t* p = static_cast<uint8_t*>(dbVal.mv_data);
    return std::vector<uint8_t>(p, p + dbVal.mv_size);
}

/**
 * Default lexicographical implementation of key comparisons used in our LMDB implementation
 */
int lexico_cmp(const MDB_val* a, const MDB_val* b)
{
    std::vector<uint8_t> a_vector = mdb_val_to_vector(*a);
    std::vector<uint8_t> b_vector = mdb_val_to_vector(*b);
    return std::lexicographical_compare(a_vector.begin(), a_vector.end(), b_vector.begin(), b_vector.end());
}

/**
 * Integer key comparison function.
 * We use this to divide the key space into discrete integer sizes
 * We check the key length in bytes to establish if it is exactly
 * 1. 1 byte
 * 2. 8 bytes
 * 3. 16 bytes
 * 4. 32 bytes
 * If it is one of the above sizes then we assume it is an integer value and we compare it as such
 */
int integer_key_cmp(const MDB_val* a, const MDB_val* b)
{
    // Id the keys sizes are different, sort by key size
    if (a->mv_size != b->mv_size) {
        return size_cmp(a, b);
    }
    uint64_t factor = a->mv_size / sizeof(uint64_t);
    uint64_t remainder = a->mv_size % sizeof(uint64_t);

    // If the size is > 32 bytes, use default comparison
    if (a->mv_size > 32) {
        return lexico_cmp(a, b);
    }
    // If the size is not a divisible by 8 then use default comparison, unless it is 1 byte
    if (a->mv_size > 1 && remainder != 0) {
        return lexico_cmp(a, b);
    }

    // Size is either 1, 8, 16 or 32 bytes, compare based on integer keys
    using f = std::function<MDB_cmp_func>;
    static std::vector<f> functions{
        value_cmp<uint8_t>, value_cmp<uint64_t>, value_cmp<uint128_t>, lexico_cmp, value_cmp<uint256_t>
    };
    return functions[factor](a, b);
}

void copy_to_vector(const MDB_val& dbVal, std::vector<uint8_t>& target)
{
    std::vector<uint8_t> temp = mdb_val_to_vector(dbVal);
    target.swap(temp);
}
} // namespace bb::crypto::merkle_tree