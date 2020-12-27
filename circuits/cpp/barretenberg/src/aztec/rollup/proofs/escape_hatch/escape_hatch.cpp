#include "escape_hatch.hpp"
#include "escape_hatch_circuit.hpp"
#include "compute_circuit_data.hpp"
#include "../join_split/join_split.hpp"
#include "../rollup/rollup_circuit.hpp"
#include <common/log.hpp>
#include <plonk/composer/turbo/compute_verification_key.hpp>
#include <stdlib/encryption/schnorr/schnorr.hpp>
#include <stdlib/merkle_tree/membership.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <plonk/proof_system/commitment_scheme/kate_commitment_scheme.hpp>

namespace rollup {
namespace proofs {
namespace escape_hatch {

using namespace plonk::stdlib::types::turbo;
using namespace plonk::stdlib::merkle_tree;

static std::shared_ptr<waffle::proving_key> proving_key;
static std::shared_ptr<waffle::verification_key> verification_key;

void init_proving_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    // Junk data required just to create proving key.
    escape_hatch_tx tx = dummy_tx();

    Composer composer(std::move(crs_factory), 512 * 1024);

    escape_hatch_circuit(composer, tx);

    info("proving key num gates: ", composer.get_num_gates());
    proving_key = composer.compute_proving_key();
}

void init_proving_key(std::shared_ptr<waffle::ProverReferenceString> const& crs, waffle::proving_key_data&& pk_data)
{
    proving_key = std::make_shared<waffle::proving_key>(std::move(pk_data), crs);
}

void init_verification_key(std::unique_ptr<waffle::ReferenceStringFactory>&& crs_factory)
{
    if (!proving_key) {
        std::abort();
    }
    // Patch the 'nothing' reference string fed to init_proving_key.
    proving_key->reference_string = crs_factory->get_prover_crs(proving_key->n);
    verification_key = waffle::turbo_composer::compute_verification_key(proving_key, crs_factory->get_verifier_crs());
}

void init_verification_key(std::shared_ptr<waffle::VerifierMemReferenceString> const& crs,
                           waffle::verification_key_data&& vk_data)
{
    verification_key = std::make_shared<waffle::verification_key>(std::move(vk_data), crs);
}

Prover new_escape_hatch_prover(escape_hatch_tx const& tx)
{
    Composer composer(proving_key, nullptr);
    escape_hatch_circuit(composer, tx);

    if (composer.failed) {
        error("composer logic failed: ", composer.err);
    }

    info("composer gates: ", composer.get_num_gates());
    info("public inputs: ", composer.public_inputs.size());

    return composer.create_prover();
}

std::vector<uint8_t> create_escape_hatch_proof(escape_hatch_tx const& tx)
{
    Composer composer(proving_key, nullptr);
    escape_hatch_circuit(composer, tx);

    if (composer.failed) {
        error("composer logic failed: ", composer.err);
    }

    info("composer gates: ", composer.get_num_gates());
    info("public inputs: ", composer.public_inputs.size());

    auto prover = composer.create_prover();
    auto proof = prover.construct_proof();
    auto proof_data = proof.proof_data;
    return proof_data;
}

bool verify_proof(waffle::plonk_proof const& proof)
{
    Verifier verifier(verification_key, Composer::create_manifest(verification_key->num_public_inputs));

    std::unique_ptr<waffle::KateCommitmentScheme<waffle::turbo_settings>> kate_commitment_scheme =
        std::make_unique<waffle::KateCommitmentScheme<waffle::turbo_settings>>();
    verifier.commitment_scheme = std::move(kate_commitment_scheme);

    return verifier.verify_proof(proof);
}

std::shared_ptr<waffle::proving_key> get_proving_key()
{
    return proving_key;
}

std::shared_ptr<waffle::verification_key> get_verification_key()
{
    return verification_key;
}

} // namespace escape_hatch
} // namespace proofs
} // namespace rollup
