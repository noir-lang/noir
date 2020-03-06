#pragma once

#include "../../fields/field2.hpp"
#include "./fq.hpp"

namespace barretenberg {
struct Bn254Fq2Params {
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
};

typedef field2<fq, Bn254Fq2Params> fq2;
} // namespace barretenberg