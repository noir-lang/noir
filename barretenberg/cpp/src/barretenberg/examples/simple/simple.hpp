#pragma once
#include "barretenberg/plonk/proof_system/types/proof.hpp"
#include "barretenberg/stdlib/types/ultra.hpp"

namespace examples::simple {

using namespace proof_system::plonk;
using namespace stdlib::types;

struct BuilderComposerPtrs {
    Builder* builder;
    Composer* composer;
};

BuilderComposerPtrs create_builder_and_composer(
    std::shared_ptr<barretenberg::srs::factories::CrsFactory<curve::BN254>> const& crs_factory);

proof create_proof(BuilderComposerPtrs pair);

bool verify_proof(BuilderComposerPtrs pair, proof_system::plonk::proof const& proof);

void delete_builder_and_composer(BuilderComposerPtrs pair);

} // namespace examples::simple
