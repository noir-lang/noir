#include "c_bind.h"
#include "sign_notes.hpp"
#include <common/serialize.hpp>
#include <common/streams.hpp>
#include <cstdint>
#include <ecc/curves/grumpkin/grumpkin.hpp>
#include <plonk/reference_string/pippenger_reference_string.hpp>
#include <plonk/proof_system/proving_key/serialize.hpp>
#include <sstream>

using namespace barretenberg;
using namespace rollup::proofs::notes;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void notes__sign_4_notes(uint8_t* pk_buffer,
                                     uint8_t const* output_owner_buffer,
                                     uint8_t const* note_buffer,
                                     uint8_t* output)
{
    auto private_key = grumpkin::fr::serialize_from_buffer(pk_buffer);

    auto output_owner = from_buffer<barretenberg::fr>(output_owner_buffer);
    grumpkin::g1::affine_element public_key = grumpkin::g1::one * private_key;
    auto notes = from_buffer<std::array<tx_note, 4>>(note_buffer);
    auto signature = sign_notes(notes, output_owner, { private_key, public_key });
    write(output, signature);
}

WASM_EXPORT void notes__encrypt_note(uint8_t const* note_buffer, uint8_t* output)
{
    tx_note note = from_buffer<tx_note>(note_buffer);
    auto encrypted = encrypt_note(note);
    write(output, encrypted);
}

WASM_EXPORT bool notes__decrypt_note(uint8_t const* encrypted_note_buf,
                                     uint8_t const* private_key_buf,
                                     uint8_t const* viewing_key_buf,
                                     uint8_t const* asset_id_buf,
                                     uint8_t* output)
{
    grumpkin::g1::affine_element encrypted_note;
    read(encrypted_note_buf, encrypted_note.x);
    read(encrypted_note_buf, encrypted_note.y);
    grumpkin::fr private_key;
    read(private_key_buf, private_key);
    fr viewing_key;
    read(viewing_key_buf, viewing_key);
    uint32_t asset_id;
    serialize::read(asset_id_buf, asset_id);
    uint256_t result;
    bool success = decrypt_note(encrypted_note, private_key, viewing_key, asset_id, result);
    write(output, result);
    return success;
}
}
