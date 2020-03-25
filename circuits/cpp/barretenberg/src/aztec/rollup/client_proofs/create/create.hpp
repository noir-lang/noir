#pragma once
#include <crypto/schnorr/schnorr.hpp>
#include <stdlib/types/turbo.hpp>
#include <plonk/reference_string/mem_reference_string.hpp>
#include "../../tx/tx_note.hpp"

namespace rollup {
namespace client_proofs {
namespace create {

using namespace rollup::tx;
using namespace plonk::stdlib::types::turbo;

void init_keys(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory);

void create_note_proof(Composer& composer, tx_note const& note, crypto::schnorr::signature const& sig);

std::vector<uint8_t> create_note_proof(tx_note const& note, crypto::schnorr::signature const& sig);

bool verify_proof(waffle::plonk_proof const& proof);

} // namespace create
} // namespace client_proofs
} // namespace rollup