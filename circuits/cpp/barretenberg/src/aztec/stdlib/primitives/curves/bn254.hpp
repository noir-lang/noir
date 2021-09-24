#pragma once

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"

namespace plonk {
namespace stdlib {

template <typename ComposerType> struct bn254 {
    typedef ComposerType Composer;
    typedef bigfield<Composer, barretenberg::Bn254FqParams> fq_ct;
    typedef field_t<Composer> fr_ct;
    typedef bool_t<Composer> bool_ct;
    typedef byte_array<Composer> byte_array_ct;
    typedef stdlib::uint32<Composer> uint32_ct;
    typedef witness_t<Composer> witness_ct;
    typedef public_witness_t<Composer> public_witness_ct;
    typedef element<Composer, fq_ct, fr_ct, barretenberg::g1> g1_ct;
    typedef bigfield<Composer, barretenberg::Bn254FrParams> bigfr_ct;
    typedef element<Composer, fq_ct, bigfr_ct, barretenberg::g1> g1_bigfr_ct;
    typedef barretenberg::g1 g1_base_t;
    typedef barretenberg::fq fq_base_t;
    typedef barretenberg::fr fr_base_t;

}; // namespace bn254
} // namespace stdlib
} // namespace plonk