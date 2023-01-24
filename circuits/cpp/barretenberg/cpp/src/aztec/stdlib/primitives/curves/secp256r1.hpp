#pragma once

#include "../bigfield/bigfield.hpp"
#include "../biggroup/biggroup.hpp"
#include "../field/field.hpp"
#include <ecc/curves/secp256r1/secp256r1.hpp>

namespace plonk {
namespace stdlib {

template <typename ComposerType> struct secp256r1_ct {
    typedef ComposerType Composer;
    typedef bigfield<Composer, secp256r1::Secp256r1FqParams> fq_ct;
    typedef field_t<Composer> fr_ct;
    typedef witness_t<Composer> witness_ct;
    typedef element<Composer, fq_ct, fr_ct, typename secp256r1::g1> g1_ct;
    typedef bigfield<Composer, typename secp256r1::Secp256r1FrParams> bigfr_ct;
    typedef element<Composer, fq_ct, bigfr_ct, typename secp256r1::g1> g1_bigfr_ct;
    typedef secp256r1::g1 g1_base_t;
    typedef secp256r1::fq fq_base_t;
    typedef secp256r1::fr fr_base_t;

}; // namespace secp256r1
} // namespace stdlib
} // namespace plonk