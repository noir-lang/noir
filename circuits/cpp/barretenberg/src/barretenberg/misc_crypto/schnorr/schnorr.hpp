#pragma once

#include <memory.h>
#include <string>
#include <array>

#include "../../keccak/keccak.h"
#include "../blake2s/blake2s.hpp"
#include "../sha256/sha256.hpp"

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

namespace crypto {
namespace schnorr {
template <typename Fr, typename G1> struct key_pair {
    Fr private_key;
    typename G1::affine_element public_key;
};

struct signature {
    std::array<uint8_t, 32> s;
    std::array<uint8_t, 32> e;
};

struct signature_b {
    std::array<uint8_t, 32> s;
    std::array<uint8_t, 32> r;
};

template <typename Hash, typename Fq, typename Fr, typename G1>
bool verify_signature(const std::string& message, const typename G1::affine_element& public_key, const signature& sig);

template <typename Hash, typename Fq, typename Fr, typename G1>
signature construct_signature(const std::string& message, const key_pair<Fr, G1>& account);

template <typename Hash, typename Fq, typename Fr, typename G1>
signature_b construct_signature_b(const std::string& message, const key_pair<Fr, G1>& account);

template <typename Hash, typename Fq, typename Fr, typename G1>
typename G1::affine_element ecrecover(const std::string& message, const signature_b& sig);

} // namespace schnorr
} // namespace crypto
#include "./schnorr.tcc"