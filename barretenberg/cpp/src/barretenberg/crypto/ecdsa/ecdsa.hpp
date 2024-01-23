#pragma once
#include "../hashers/hashers.hpp"
#include "barretenberg/ecc/curves/secp256k1/secp256k1.hpp"

#include "barretenberg/ecc/curves/secp256r1/secp256r1.hpp"

#include "barretenberg/serialize/msgpack.hpp"
#include <array>
#include <string>

namespace bb::crypto {
template <typename Fr, typename G1> struct ecdsa_key_pair {
    Fr private_key;
    typename G1::affine_element public_key;
    // For serialization, update with any new fields
    MSGPACK_FIELDS(private_key, public_key);
};

struct ecdsa_signature {
    std::array<uint8_t, 32> r;
    std::array<uint8_t, 32> s;
    uint8_t v;
    // For serialization, update with any new fields
    MSGPACK_FIELDS(r, s, v);
};

template <typename Hash, typename Fq, typename Fr, typename G1>
ecdsa_signature ecdsa_construct_signature(const std::string& message, const ecdsa_key_pair<Fr, G1>& account);

template <typename Hash, typename Fq, typename Fr, typename G1>
typename G1::affine_element ecdsa_recover_public_key(const std::string& message, const ecdsa_signature& sig);

template <typename Hash, typename Fq, typename Fr, typename G1>
bool ecdsa_verify_signature(const std::string& message,
                            const typename G1::affine_element& public_key,
                            const ecdsa_signature& signature);

inline bool operator==(ecdsa_signature const& lhs, ecdsa_signature const& rhs)
{
    return lhs.r == rhs.r && lhs.s == rhs.s && lhs.v == rhs.v;
}

inline std::ostream& operator<<(std::ostream& os, ecdsa_signature const& sig)
{
    os << "{ " << sig.r << ", " << sig.s << ", " << static_cast<uint32_t>(sig.v) << " }";
    return os;
}

} // namespace bb::crypto

#include "./ecdsa_impl.hpp"
