#pragma once
#include <ecc/curves/bn254/fr.hpp>
#include <plonk/proof_system/types/plonk_proof.hpp>

namespace rollup {
namespace rollup_proofs {

waffle::plonk_proof create_noop_join_split_proof(barretenberg::fr const& merkle_root);

}
}