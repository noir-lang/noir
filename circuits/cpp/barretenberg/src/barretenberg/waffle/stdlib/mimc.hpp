#pragma once

#include <vector>

namespace waffle {
class StandardComposer;
class MiMCComposer;
} // namespace waffle

namespace plonk {
namespace stdlib {

template <typename ComposerContext> class field_t;

field_t<waffle::MiMCComposer> mimc_block_cipher(field_t<waffle::MiMCComposer> input,
                                                field_t<waffle::MiMCComposer> k_in);

field_t<waffle::StandardComposer> mimc_block_cipher(field_t<waffle::StandardComposer> input,
                                                    field_t<waffle::StandardComposer> k_in);

template <typename Composer> field_t<Composer> mimc7(std::vector<field_t<Composer>> const& inputs);

extern template field_t<waffle::StandardComposer> mimc7(std::vector<field_t<waffle::StandardComposer>> const& inputs);
extern template field_t<waffle::MiMCComposer> mimc7(std::vector<field_t<waffle::MiMCComposer>> const& inputs);

} // namespace stdlib
} // namespace plonk
