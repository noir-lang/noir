#pragma once
#include "../hashers/hashers.hpp"
#include <array>
#include <string>
#include "barretenberg/ecc/curves/secp256k1/secp256k1.hpp"

namespace crypto {
namespace ecdsa {
template <typename Fr, typename G1> struct key_pair {
    Fr private_key;
    typename G1::affine_element public_key;
};

struct signature {
    std::array<uint8_t, 32> r;
    std::array<uint8_t, 32> s;
};

template <typename Hash, typename Fq, typename Fr, typename G1>
signature construct_signature(const std::string& message, const key_pair<Fr, G1>& account);

template <typename Hash, typename Fq, typename Fr, typename G1>
bool verify_signature(const std::string& message,
                      const typename G1::affine_element& public_key,
                      const signature& signature);

inline bool operator==(signature const& lhs, signature const& rhs)
{
    return lhs.r == rhs.r && lhs.s == rhs.s;
}

inline std::ostream& operator<<(std::ostream& os, signature const& sig)
{
    os << "{ " << sig.r << ", " << sig.s << " }";
    return os;
}

template <typename B> inline void read(B& it, signature& sig)
{
    read(it, sig.r);
    read(it, sig.s);
}

template <typename B> inline void write(B& buf, signature const& sig)
{
    write(buf, sig.r);
    write(buf, sig.s);
}

template <typename B> inline void read(B& it, key_pair<secp256k1::fr, secp256k1::g1>& keypair)
{
    read(it, keypair.private_key);
    read(it, keypair.public_key);
}

template <typename B> inline void write(B& buf, key_pair<secp256k1::fr, secp256k1::g1> const& keypair)
{
    write(buf, keypair.private_key);
    write(buf, keypair.public_key);
}

} // namespace ecdsa
} // namespace crypto

#include "./ecdsa_impl.hpp"