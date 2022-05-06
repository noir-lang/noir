#include "../blake2s/blake2s.hpp"
#include "../keccak/keccak.hpp"
#include "../sha256/sha256.hpp"

#include "memory.h"
#include <vector>
struct KeccakHasher {
    static constexpr size_t BLOCK_SIZE = 64;
    static constexpr size_t OUTPUT_SIZE = 32;
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
    static constexpr size_t BLOCK_SIZE = 64;
    static constexpr size_t OUTPUT_SIZE = 32;

    template <typename B = std::vector<uint8_t>> static auto hash(const B& message) { return sha256::sha256(message); }
};

struct Blake2sHasher {
    static constexpr size_t BLOCK_SIZE = 64;
    static constexpr size_t OUTPUT_SIZE = 32;
    static std::vector<uint8_t> hash(const std::vector<uint8_t>& message) { return blake2::blake2s(message); }
};
