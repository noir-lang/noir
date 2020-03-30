#pragma once
#include "../../tx/tx_note.hpp"
#include <crypto/schnorr/schnorr.hpp>
#include <plonk/reference_string/mem_reference_string.hpp>
#include <stdlib/types/turbo.hpp>

namespace rollup {
namespace client_proofs {
namespace create {

using namespace rollup::tx;
using namespace plonk::stdlib::types::turbo;

void init_keys(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void create_note_circuit(Composer& composer, tx_note const& note, crypto::schnorr::signature const& sig);

Prover new_create_note_prover(tx_note const& note, crypto::schnorr::signature const& sig);

bool verify_proof(waffle::plonk_proof const& proof);

} // namespace create
} // namespace client_proofs
} // namespace rollup