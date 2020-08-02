#pragma once
#include <stdlib/types/turbo.hpp>
#include "note_types.hpp"

namespace rollup {
namespace client_proofs {
namespace notes {

using namespace plonk::stdlib::types::turbo;

struct account_note {
    account_note(point_ct const& owner_pub_key,
                 grumpkin::g1::affine_element const& signing_pub_key,
                 bool_ct const& is_real)
        : composer_(*owner_pub_key.x.context)
        , owner_pub_key_(owner_pub_key)
        , signing_pub_key_({ witness_ct(&composer_, signing_pub_key.x), witness_ct(&composer_, signing_pub_key.y) })
        , is_real_(is_real)
        , leaf_data_(&composer_)
        , nullifier_(&composer_)
    {}

    point_ct const& owner_pub_key() const { return owner_pub_key_; }

    point_ct const& signing_pub_key() const { return signing_pub_key_; }

    byte_array_ct const& leaf_data() const
    {
        if (leaf_data_.size()) {
            return leaf_data_;
        }
        // TODO: Not sure about this prefix stuff. Is it needed? We need to fit note data into 2 fields...
        // field_ct account_prefix(&composer_, ACCOUNT);
        // field_ct gibberish_prefix(&composer_, GIBBERISH);
        // byte_array_ct prefix = (account_prefix * is_real) + (gibberish_prefix * !is_real);
        return leaf_data_ /*.write(prefix)*/.write(owner_pub_key_.x).write(signing_pub_key_.x);
    }

    field_ct nullifier() const
    {
        if (!nullifier_.get_value().is_zero()) {
            return nullifier_;
        }
        return nullifier_ = field_ct(stdlib::blake2s(leaf_data()));
    }

    void set_public() const
    {
        // Owners public key is shared between notes, so just calling set_public_input only makes it public once.
        // We need it to be exposed as a public input twice. So, create a new public input and assert equality.
        auto owner_x = public_witness_ct(&composer_, owner_pub_key_.x.get_value());
        composer_.assert_equal(owner_pub_key_.x.witness_index, owner_x.witness_index);
        composer_.set_public_input(signing_pub_key_.x.witness_index);
    }

  private:
    Composer& composer_;
    const point_ct owner_pub_key_;
    const point_ct signing_pub_key_;
    bool_ct is_real_;
    mutable byte_array_ct leaf_data_;
    mutable field_ct nullifier_;
};

} // namespace notes
} // namespace client_proofs
} // namespace rollup