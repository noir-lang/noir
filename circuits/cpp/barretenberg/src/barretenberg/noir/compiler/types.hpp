#pragma once
#include "../../waffle/composer/turbo_composer.hpp"
#include "../../waffle/stdlib/bool/bool.hpp"
#include "../../waffle/stdlib/uint/noir_uint.hpp"

namespace noir {
namespace code_gen {

//#define throw std::abort(); auto __ex__ =

typedef waffle::TurboComposer Composer;
typedef plonk::stdlib::field_t<Composer> field_t;
typedef plonk::stdlib::bool_t<Composer> bool_t;
typedef plonk::stdlib::witness_t<Composer> witness_t;
typedef plonk::stdlib::uintNoir<Composer> uint;

} // namespace code_gen
} // namespace noir