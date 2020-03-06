#pragma once

#include "../field/field.hpp"

namespace waffle {
class TurboComposer;
}

namespace plonk {
namespace stdlib {
struct point {
    field_t<waffle::TurboComposer> x;
    field_t<waffle::TurboComposer> y;
};

} // namespace stdlib
} // namespace plonk