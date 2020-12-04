#include "c_bind.h"
#include "native/sign_notes.hpp"
#include "native/encrypt_note.hpp"
#include "native/compute_nullifier.hpp"

using namespace barretenberg;
using namespace rollup::proofs::notes::native;

#define WASM_EXPORT __attribute__((visibility("default")))

extern "C" {

WASM_EXPORT void notes__encrypt_note(uint8_t const* note_buffer, uint8_t* output)
{
    auto note = from_buffer<value_note>(note_buffer);
    auto encrypted = encrypt_note(note);
    write(output, encrypted);
}

WASM_EXPORT void notes__compute_nullifier(
    uint8_t const* enc_note_buffer, uint8_t* acc_pk_buffer, uint32_t index, bool is_real, uint8_t* output)
{
    auto enc_note = from_buffer<grumpkin::g1::affine_element>(enc_note_buffer);
    auto acc_pk = from_buffer<uint256_t>(acc_pk_buffer);
    auto nullifier = compute_nullifier(enc_note, index, acc_pk, is_real);
    write(output, nullifier);
}
}
