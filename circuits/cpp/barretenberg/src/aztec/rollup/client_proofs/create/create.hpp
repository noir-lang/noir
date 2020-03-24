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

waffle::plonk_proof create_note_proof(Composer& composer, tx_note const& note, crypto::schnorr::signature const& sig);

std::vector<uint8_t> create_note_proof(tx_note const& note,
                                       crypto::schnorr::signature const& sig,
                                       std::unique_ptr<waffle::MemReferenceStringFactory>&& crs_factory);

} // namespace create
} // namespace client_proofs
} // namespace rollup