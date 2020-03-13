#pragma once
#include <plonk/composer/standard_composer.hpp>
#include "../byte_array/byte_array.hpp"

namespace stdlib {
namespace types {
namespace standard {

using namespace plonk;

typedef waffle::StandardComposer Composer;
typedef stdlib::bool_t<Composer> bool_t;
typedef stdlib::byte_array<Composer> byte_array;

} // namespace standard
} // namespace types
} // namespace stdlib