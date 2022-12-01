#pragma once

#include <array>
#include <memory.h>
#include <string>

#include <ecc/curves/grumpkin/grumpkin.hpp>

#include <crypto/hashers/hashers.hpp>

#include <common/serialize.hpp>
#include <common/streams.hpp>

namespace crypto {
namespace schnorr {
template <typename Fr, typename G1> struct key_pair {
    Fr private_key;
    typename G1::affine_element public_key;
};

struct signature {
    std::array<uint8_t, 32> s; // Fr
    std::array<uint8_t, 32> e; // Fr
};

template <typename Hash, typename Fq, typename Fr, typename G1>
bool verify_signature(const std::string& message, const typename G1::affine_element& public_key, const signature& sig);

template <typename Hash, typename Fq, typename Fr, typename G1>
signature construct_signature(const std::string& message, const key_pair<Fr, G1>& account);

inline bool operator==(signature const& lhs, signature const& rhs)
{
    return lhs.s == rhs.s && lhs.e == rhs.e;
}

inline std::ostream& operator<<(std::ostream& os, signature const& sig)
{
    os << "{ " << sig.s << ", " << sig.e << " }";
    return os;
}

template <typename B> inline void read(B& it, signature& sig)
{
    read(it, sig.s);
    read(it, sig.e);
}

template <typename B> inline void write(B& buf, signature const& sig)
{
    write(buf, sig.s);
    write(buf, sig.e);
}

template <typename B> inline void read(B& it, key_pair<grumpkin::fr, grumpkin::g1>& keypair)
{
    read(it, keypair.private_key);
    read(it, keypair.public_key);
}

template <typename B> inline void write(B& buf, key_pair<grumpkin::fr, grumpkin::g1> const& keypair)
{
    write(buf, keypair.private_key);
    write(buf, keypair.public_key);
}
} // namespace schnorr
} // namespace crypto
#include "./schnorr.tcc"