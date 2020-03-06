#pragma once

#include <stdint.h>

#include "../composer/composer.hpp"

namespace waffle
{
namespace mimc
{
    uint32_t mimc_round(const uint32_t input_index, Composer &composer);
}
}