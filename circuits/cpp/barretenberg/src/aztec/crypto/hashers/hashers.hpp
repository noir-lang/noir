#include "../blake2s/blake2s.hpp"
#include "../keccak/keccak.hpp"
#include "../sha256/sha256.hpp"

#include "memory.h"
#include <vector>
struct KeccakHasher {
    static std::vector<uint8_t> hash(const std::vector<uint8_t>& message)
    {
        keccak256 hash_result = ethash_keccak256(&message[0], message.size());

        std::vector<uint8_t> output;
        output.resize(32);

        memcpy((void*)&output[0], (void*)&hash_result.word64s[0], 32);
        return output;
    }
};

struct Sha256Hasher {
    static std::vector<uint8_t> hash(const std::vector<uint8_t>& message) { return sha256::sha256(message); }
};

struct Blake2sHasher {
    static std::vector<uint8_t> hash(const std::vector<uint8_t>& message) { return blake2::blake2s(message); }
};
