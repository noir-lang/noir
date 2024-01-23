#pragma once

#include <array>
#include <memory.h>
#include <string>

#include "barretenberg/ecc/curves/grumpkin/grumpkin.hpp"

#include "barretenberg/crypto/hashers/hashers.hpp"

#include "barretenberg/common/serialize.hpp"
#include "barretenberg/common/streams.hpp"
#include "barretenberg/serialize/msgpack.hpp"

namespace bb::crypto {
template <typename Fr, typename G1> struct schnorr_key_pair {
    Fr private_key;
    typename G1::affine_element public_key;
};

// Raw representation of a Schnorr signature (e,s).  We use the short variant of Schnorr
// where we include the challenge hash `e` instead of the group element R representing
// the provers initial message.
struct schnorr_signature {

    // `s` is a serialized field element (also 32 bytes), representing the prover's response to
    // to the verifier challenge `e`.
    // We do not enforce that `s` is canonical since signatures are verified inside a circuit,
    // and are provided as private inputs. Malleability is not an issue in this case.
    std::array<uint8_t, 32> s;
    // `e` represents the verifier's challenge in the protocol. It is encoded as the 32-byte
    // output of a hash function modeling a random oracle in the Fiat-Shamir transform.
    std::array<uint8_t, 32> e;
    MSGPACK_FIELDS(s, e);
};

template <typename Hash, typename Fq, typename Fr, typename G1>
bool schnorr_verify_signature(const std::string& message,
                              const typename G1::affine_element& public_key,
                              const schnorr_signature& sig);

template <typename Hash, typename Fq, typename Fr, typename G1>
schnorr_signature schnorr_construct_signature(const std::string& message, const schnorr_key_pair<Fr, G1>& account);

inline bool operator==(schnorr_signature const& lhs, schnorr_signature const& rhs)
{
    return lhs.s == rhs.s && lhs.e == rhs.e;
}

inline std::ostream& operator<<(std::ostream& os, schnorr_signature const& sig)
{
    os << "{ " << sig.s << ", " << sig.e << " }";
    return os;
}

template <typename B> inline void read(B& it, schnorr_key_pair<grumpkin::fr, grumpkin::g1>& keypair)
{
    read(it, keypair.private_key);
    read(it, keypair.public_key);
}

template <typename B> inline void write(B& buf, schnorr_key_pair<grumpkin::fr, grumpkin::g1> const& keypair)
{
    write(buf, keypair.private_key);
    write(buf, keypair.public_key);
}
} // namespace bb::crypto
#include "./schnorr.tcc"
