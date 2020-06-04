#pragma once

#include <array>
#include <memory.h>
#include <string>

#include <common/serialize.hpp>
#include <common/streams.hpp>
#include <crypto/blake2s/blake2s.hpp>
#include <crypto/keccak/keccak.hpp>
#include <crypto/sha256/sha256.hpp>

#include "../hashers/hashers.hpp"

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

inline bool operator==(signature const& lhs, signature const& rhs)
{
    return lhs.s == rhs.s && lhs.e == rhs.e;
}

inline std::ostream& operator<<(std::ostream& os, signature const& sig)
{
    os << "{ " << sig.s << ", " << sig.e << " }";
    return os;
}

inline void read(uint8_t const*& it, signature& sig)
{
    read(it, sig.s);
    read(it, sig.e);
}

template <typename B> inline void write(B& buf, signature const& sig)
{
    write(buf, sig.s);
    write(buf, sig.e);
}

} // namespace schnorr
} // namespace crypto
#include "./schnorr.tcc"