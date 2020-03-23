#include "../hashers/hashers.hpp"

#include <array>

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
} // namespace ecdsa
} // namespace crypto

#include "./ecdsa_impl.hpp"