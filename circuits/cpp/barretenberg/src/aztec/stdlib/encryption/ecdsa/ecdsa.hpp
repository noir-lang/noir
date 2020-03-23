#pragma once

#include "../../primitives/byte_array/byte_array.hpp"
#include "../../primitives/composers/composers_fwd.hpp"

namespace plonk {
namespace stdlib {
namespace ecdsa {

template <typename Composer> struct signature {
    stdlib::byte_array<Composer> r;
    stdlib::byte_array<Composer> s;
};

// (ノಠ益ಠ)ノ彡┻━┻
template <typename Composer, typename Fq, typename Fr, typename G1>
bool_t<Composer> verify_signature(const stdlib::byte_array<Composer>& message,
                                  const G1& public_key,
                                  const signature<Composer>& sig);
} // namespace ecdsa
} // namespace stdlib
} // namespace plonk

#include "./ecdsa_impl.hpp"