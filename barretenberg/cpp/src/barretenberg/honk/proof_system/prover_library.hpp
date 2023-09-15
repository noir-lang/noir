#pragma once
#include "barretenberg/common/constexpr_utils.hpp"
#include "barretenberg/ecc/curves/bn254/fr.hpp"
#include "barretenberg/plonk/proof_system/proving_key/proving_key.hpp"
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/proof_system/relations/relation_parameters.hpp"

// TODO(@zac-williamson #2216). We used to include `program_settings.hpp` in this file. Needed to remove due to circular
// dependency. `program_settings.hpp` included header files that added "using namespace proof_system" and "using
// namespace barretenberg" declarations. This effects downstream code that relies on these using declarations. This is a
// big code smell (should really not have using declarations in header files!), however fixing it requires changes in a
// LOT of files. This would clutter the eccvm feature PR. Adding these following "using namespace" declarations is a
// temp workaround. Once this work is merged in we should fix the root problem (no using declarations in header files)
// See issue #2216
using namespace proof_system;
using namespace barretenberg;
namespace proof_system::honk::prover_library {

template <typename Flavor>
typename Flavor::Polynomial compute_sorted_list_accumulator(std::shared_ptr<typename Flavor::ProvingKey>& key,
                                                            typename Flavor::FF eta);

template <typename Flavor>
void add_plookup_memory_records_to_wire_4(std::shared_ptr<typename Flavor::ProvingKey>& key, typename Flavor::FF eta);

} // namespace proof_system::honk::prover_library
