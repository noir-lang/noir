//! `default_on_reserved` is a sum-level decode-policy flag — it controls
//! what the runtime does when an *unknown variant tag* is encountered. It
//! has no analog at the variant level (variant payloads are products with
//! their own field-tag space) and is rejected here.

use msgpack_tagged::MsgpackTagged;

#[derive(MsgpackTagged)]
enum Choice {
    #[tag(0)]
    #[tagged(default_on_reserved)]
    First,
}

fn main() {}
