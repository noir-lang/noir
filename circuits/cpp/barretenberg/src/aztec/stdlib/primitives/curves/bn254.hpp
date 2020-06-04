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
    typedef element<Composer, fq_ct, fr_ct, barretenberg::g1> g1_ct;
    typedef barretenberg::g1 g1_base_t;
    typedef barretenberg::fq fq_base_t;
    typedef barretenberg::fr fr_base_t;
}; // namespace bn254
} // namespace stdlib
} // namespace plonk