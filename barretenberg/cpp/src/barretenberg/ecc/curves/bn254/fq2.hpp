#pragma once

#include "../../fields/field2.hpp"
#include "./fq.hpp"

namespace bb {
struct Bn254Fq2Params {
#if defined(__SIZEOF_INT128__) && !defined(__wasm__)
    static constexpr fq twist_coeff_b_0{
        0x3bf938e377b802a8UL, 0x020b1b273633535dUL, 0x26b7edf049755260UL, 0x2514c6324384a86dUL
    };
    static constexpr fq twist_coeff_b_1{
        0x38e7ecccd1dcff67UL, 0x65f0b37d93ce0d3eUL, 0xd749d0dd22ac00aaUL, 0x0141b9ce4a688d4dUL
    };
    static constexpr fq twist_mul_by_q_x_0{
        0xb5773b104563ab30UL, 0x347f91c8a9aa6454UL, 0x7a007127242e0991UL, 0x1956bcd8118214ecUL
    };
    static constexpr fq twist_mul_by_q_x_1{
        0x6e849f1ea0aa4757UL, 0xaa1c7b6d89f89141UL, 0xb6e713cdfae0ca3aUL, 0x26694fbb4e82ebc3UL
    };
    static constexpr fq twist_mul_by_q_y_0{
        0xe4bbdd0c2936b629UL, 0xbb30f162e133bacbUL, 0x31a9d1b6f9645366UL, 0x253570bea500f8ddUL
    };
    static constexpr fq twist_mul_by_q_y_1{
        0xa1d77ce45ffe77c7UL, 0x07affd117826d1dbUL, 0x6d16bd27bb7edc6bUL, 0x2c87200285defeccUL
    };
    static constexpr fq twist_cube_root_0{
        0x505ecc6f0dff1ac2UL, 0x2071416db35ec465UL, 0xf2b53469fa43ea78UL, 0x18545552044c99aaUL
    };
    static constexpr fq twist_cube_root_1{
        0xad607f911cfe17a8UL, 0xb6bb78aa154154c4UL, 0xb53dd351736b20dbUL, 0x1d8ed57c5cc33d41UL
    };
#else
    static constexpr fq twist_coeff_b_0{
        0xdc19fa4aab489658UL, 0xd416744fbbf6e69UL, 0x8f7734ed0a8a033aUL, 0x19316b8353ee09bbUL
    };
    static constexpr fq twist_coeff_b_1{
        0x1cfd999a3b9fece0UL, 0xbe166fb279c1a7c7UL, 0xe93a1ba45580154cUL, 0x283739c94d11a9baUL
    };
    static constexpr fq twist_mul_by_q_x_0{
        0xecdea09b24a59190UL, 0x17db8ffeae2fe1c2UL, 0xbb09c97c6dabac4dUL, 0x2492b3d41d289af3UL
    };
    static constexpr fq twist_mul_by_q_x_1{
        0xf1663598f1142ef1UL, 0x77ec057e0bf56062UL, 0xdd0baaecb677a631UL, 0x135e4e31d284d463UL
    };
    static constexpr fq twist_mul_by_q_y_0{
        0xf46e7f60db1f0678UL, 0x31fc2eba5bcc5c3eUL, 0xedb3adc3086a2411UL, 0x1d46bd0f837817bcUL
    };
    static constexpr fq twist_mul_by_q_y_1{
        0x6b3fbdf579a647d5UL, 0xcc568fb62ff64974UL, 0xc1bfbf4ac4348ac6UL, 0x15871d4d3940b4d3UL
    };
    static constexpr fq twist_cube_root_0{
        0x49d0cc74381383d0UL, 0x9611849fe4bbe3d6UL, 0xd1a231d73067c92aUL, 0x445c312767932c2UL
    };
    static constexpr fq twist_cube_root_1{
        0x35a58c718e7c28bbUL, 0x98d42c77e7b8901aUL, 0xf9c53da2d0ca8c84UL, 0x1a68dd04e1b8c51dUL
    };
#endif
};

using fq2 = field2<fq, Bn254Fq2Params>;
} // namespace bb